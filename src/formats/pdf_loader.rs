use pdfium_render::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RenderRequest {
    page_index: usize,
    zoom_bucket: u32,
}

enum WorkerReply {
    Rendered {
        page_index: usize,
        zoom_bucket: u32,
        image: Result<egui::ColorImage, String>,
    },
    Skipped {
        page_index: usize,
        zoom_bucket: u32,
    },
}

struct InitMeta {
    title: String,
    author: String,
    page_count: usize,
    backend_name: &'static str,
    cover_image: Option<egui::ColorImage>,
}

pub enum PageTextureState {
    Ready(egui::TextureHandle),
    Loading,
    Error(String),
}

/// Gerenciador assíncrono de PDF:
/// - Mantém o documento aberto em uma thread dedicada (sem reabrir o arquivo a cada página)
/// - Nunca bloqueia a thread da interface (`egui`), mantendo 60 FPS mesmo em PDFs de 300+ páginas
/// - Usa fila LIFO (prioriza páginas visíveis agora e descarta requisições antigas ao rolar rápido)
pub struct PdfDocumentHandle {
    pub path: PathBuf,
    pub title: String,
    pub author: String,
    pub page_count: usize,
    backend_name: &'static str,
    cover_image: Option<egui::ColorImage>,
    req_tx: Sender<RenderRequest>,
    reply_rx: Receiver<WorkerReply>,
    /// Cache LRU de texturas: chave = (page_index, zoom_bucket)
    texture_cache: HashMap<(usize, u32), egui::TextureHandle>,
    lru_order: VecDeque<(usize, u32)>,
    /// Requisições atualmente em andamento na thread worker
    in_flight: HashSet<(usize, u32)>,
    /// Últimos erros por página
    page_errors: HashMap<usize, String>,
}

impl PdfDocumentHandle {
    const MAX_CACHED_PAGES: usize = 24;

    pub fn open(path: &Path, ctx: egui::Context) -> Result<Self, String> {
        let path_buf = path.to_path_buf();
        let fallback_title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Documento PDF".to_string());

        let (init_tx, init_rx) = mpsc::channel::<Result<InitMeta, String>>();
        let (req_tx, req_rx) = mpsc::channel::<RenderRequest>();
        let (reply_tx, reply_rx) = mpsc::channel::<WorkerReply>();

        let worker_path = path_buf.clone();
        thread::spawn(move || {
            pdf_worker_loop(worker_path, fallback_title, ctx, init_tx, req_rx, reply_tx);
        });

        let meta = init_rx
            .recv()
            .map_err(|_| "Falha ao inicializar thread do leitor PDF.".to_string())??;

        Ok(Self {
            path: path_buf,
            title: meta.title,
            author: meta.author,
            page_count: meta.page_count.max(1),
            backend_name: meta.backend_name,
            cover_image: meta.cover_image,
            req_tx,
            reply_rx,
            texture_cache: HashMap::new(),
            lru_order: VecDeque::new(),
            in_flight: HashSet::new(),
            page_errors: HashMap::new(),
        })
    }

    pub fn backend_name(&self) -> &'static str {
        self.backend_name
    }

    pub fn take_cover_image(&mut self) -> Option<egui::ColorImage> {
        self.cover_image.take()
    }

    /// Recebe páginas finalizadas pela thread em segundo plano e sobe para a GPU (não bloqueante)
    pub fn poll_results(&mut self, ctx: &egui::Context) {
        while let Ok(reply) = self.reply_rx.try_recv() {
            match reply {
                WorkerReply::Rendered {
                    page_index,
                    zoom_bucket,
                    image,
                } => {
                    let key = (page_index, zoom_bucket);
                    self.in_flight.remove(&key);
                    match image {
                        Ok(color_img) => {
                            self.page_errors.remove(&page_index);
                            let tex_name = format!("pdf_p{}_z{}", page_index, zoom_bucket);
                            let handle =
                                ctx.load_texture(tex_name, color_img, egui::TextureOptions::LINEAR);
                            self.insert_cache(key, handle);
                        }
                        Err(err) => {
                            self.page_errors.insert(page_index, err);
                        }
                    }
                }
                WorkerReply::Skipped {
                    page_index,
                    zoom_bucket,
                } => {
                    self.in_flight.remove(&(page_index, zoom_bucket));
                }
            }
        }
    }

    /// Obtém a textura da página instantaneamente se estiver em cache,
    /// ou agenda a renderização em segundo plano sem travar a interface.
    pub fn get_or_request_page(&mut self, page_index: usize, zoom: f32) -> PageTextureState {
        let zoom_bucket = quantize_zoom(zoom);
        let key = (page_index, zoom_bucket);

        if let Some(tex) = self.texture_cache.get(&key).cloned() {
            self.touch_lru(key);
            return PageTextureState::Ready(tex);
        }

        // Agenda a renderização na resolução exata
        self.enqueue_request(page_index, zoom_bucket);

        // Enquanto a nova resolução renderiza, reutiliza a mesma página em outro zoom (evita piscar)
        if let Some(fallback_tex) = self
            .texture_cache
            .iter()
            .find(|((p, _), _)| *p == page_index)
            .map(|(_, tex)| tex.clone())
        {
            return PageTextureState::Ready(fallback_tex);
        }

        if let Some(err) = self.page_errors.get(&page_index) {
            return PageTextureState::Error(err.clone());
        }

        PageTextureState::Loading
    }

    /// Pré-carrega páginas vizinhas em segundo plano (ex.: próxima página no modo paginado)
    pub fn prefetch(&mut self, page_index: usize, zoom: f32) {
        if page_index >= self.page_count {
            return;
        }
        let zoom_bucket = quantize_zoom(zoom);
        let key = (page_index, zoom_bucket);
        if !self.texture_cache.contains_key(&key) {
            self.enqueue_request(page_index, zoom_bucket);
        }
    }

    fn enqueue_request(&mut self, page_index: usize, zoom_bucket: u32) {
        let key = (page_index, zoom_bucket);
        if self.in_flight.insert(key) {
            let _ = self.req_tx.send(RenderRequest {
                page_index,
                zoom_bucket,
            });
        }
    }

    fn insert_cache(&mut self, key: (usize, u32), handle: egui::TextureHandle) {
        self.lru_order.retain(|k| k != &key);
        self.lru_order.push_back(key);
        self.texture_cache.insert(key, handle);

        while self.lru_order.len() > Self::MAX_CACHED_PAGES {
            if let Some(oldest) = self.lru_order.pop_front() {
                self.texture_cache.remove(&oldest);
            }
        }
    }

    fn touch_lru(&mut self, key: (usize, u32)) {
        if self.lru_order.back() != Some(&key) {
            self.lru_order.retain(|k| k != &key);
            self.lru_order.push_back(key);
        }
    }
}

fn quantize_zoom(zoom: f32) -> u32 {
    (((zoom * 100.0) / 20.0).round() as u32 * 20).clamp(60, 220)
}

fn pdf_worker_loop(
    path: PathBuf,
    fallback_title: String,
    ctx: egui::Context,
    init_tx: Sender<Result<InitMeta, String>>,
    req_rx: Receiver<RenderRequest>,
    reply_tx: Sender<WorkerReply>,
) {
    // 1. Tenta inicializar com Pdfium (mantém o documento aberto na memória desta thread!)
    if let Some(pdfium) = try_bind_pdfium() {
        let doc = match pdfium.load_pdf_from_file(&path, None) {
            Ok(d) => d,
            Err(e) => {
                let _ = init_tx.send(Err(format!("Falha ao abrir PDF com Pdfium: {}", e)));
                return;
            }
        };

        let page_count = doc.pages().len() as usize;
        let title = doc
            .metadata()
            .get(PdfDocumentMetadataTagType::Title)
            .map(|t| t.value().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback_title);

        let author = doc
            .metadata()
            .get(PdfDocumentMetadataTagType::Author)
            .map(|t| t.value().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Autor desconhecido".to_string());

        let cover_image = render_pdfium_page(&doc, 0, 0.55).ok();

        if init_tx
            .send(Ok(InitMeta {
                title,
                author,
                page_count: page_count.max(1),
                backend_name: "Pdfium (assíncrono)",
                cover_image,
            }))
            .is_err()
        {
            return;
        }

        // Loop de renderização com documento já aberto — sem reabrir o PDF!
        while let Ok(first) = req_rx.recv() {
            let batch = drain_latest_requests(first, &req_rx, &reply_tx, 6);
            for req in batch {
                let zoom = req.zoom_bucket as f32 / 100.0;
                let image = render_pdfium_page(&doc, req.page_index, zoom);
                if reply_tx
                    .send(WorkerReply::Rendered {
                        page_index: req.page_index,
                        zoom_bucket: req.zoom_bucket,
                        image,
                    })
                    .is_err()
                {
                    return;
                }
                ctx.request_repaint();
            }
        }
        return;
    }

    // 2. Fallback automático: Poppler CLI (pdftoppm) em segundo plano
    if command_exists("pdftoppm") {
        let (page_count, meta_title, meta_author) = read_poppler_info(&path);
        let cover_image = render_poppler_page(&path, 0, 0.55).ok();

        if init_tx
            .send(Ok(InitMeta {
                title: meta_title.unwrap_or(fallback_title),
                author: meta_author.unwrap_or_else(|| "Autor desconhecido".to_string()),
                page_count: page_count.max(1),
                backend_name: "Poppler (assíncrono)",
                cover_image,
            }))
            .is_err()
        {
            return;
        }

        while let Ok(first) = req_rx.recv() {
            let batch = drain_latest_requests(first, &req_rx, &reply_tx, 4);
            for req in batch {
                let zoom = req.zoom_bucket as f32 / 100.0;
                let image = render_poppler_page(&path, req.page_index, zoom);
                if reply_tx
                    .send(WorkerReply::Rendered {
                        page_index: req.page_index,
                        zoom_bucket: req.zoom_bucket,
                        image,
                    })
                    .is_err()
                {
                    return;
                }
                ctx.request_repaint();
            }
        }
        return;
    }

    let _ = init_tx.send(Err(
        "Motor PDF não encontrado. Coloque `libpdfium.so` / `pdfium.dll` / `libpdfium.dylib` na pasta do aplicativo (veja README.md) ou instale o `poppler-utils` (`pdftoppm`)."
            .to_string(),
    ));
}

/// Coleta todas as requisições acumuladas na fila, prioriza as mais recentes (LIFO)
/// e marca as antigas como `Skipped` para não desperdiçar tempo ao rolar rápido por 300+ páginas.
fn drain_latest_requests(
    first: RenderRequest,
    req_rx: &Receiver<RenderRequest>,
    reply_tx: &Sender<WorkerReply>,
    max_keep: usize,
) -> Vec<RenderRequest> {
    let mut queue = vec![first];
    while let Ok(req) = req_rx.try_recv() {
        if let Some(pos) = queue.iter().position(|r| r == &req) {
            queue.remove(pos);
        }
        queue.push(req);
    }

    // Mais recentes primeiro
    queue.reverse();
    if queue.len() > max_keep {
        for skipped in queue.drain(max_keep..) {
            let _ = reply_tx.send(WorkerReply::Skipped {
                page_index: skipped.page_index,
                zoom_bucket: skipped.zoom_bucket,
            });
        }
    }
    queue
}

fn render_pdfium_page(
    doc: &PdfDocument<'_>,
    page_index: usize,
    zoom: f32,
) -> Result<egui::ColorImage, String> {
    let page = doc
        .pages()
        .get(page_index as u16)
        .map_err(|e| format!("Página {} inválida: {}", page_index + 1, e))?;

    let target_width = (980.0 * zoom).clamp(420.0, 2200.0) as i32;
    let config = PdfRenderConfig::new()
        .set_target_width(target_width)
        .set_maximum_height(3000);

    let bitmap = page
        .render_with_config(&config)
        .map_err(|e| format!("Erro ao renderizar página: {}", e))?;

    let w = bitmap.width() as usize;
    let h = bitmap.height() as usize;
    let rgba = bitmap.as_rgba_bytes();

    Ok(egui::ColorImage::from_rgba_unmultiplied([w, h], &rgba))
}

fn render_poppler_page(
    path: &Path,
    page_index: usize,
    zoom: f32,
) -> Result<egui::ColorImage, String> {
    let dpi = (120.0 * zoom).clamp(72.0, 240.0) as u32;
    let page_num = (page_index + 1).to_string();
    let output = Command::new("pdftoppm")
        .args([
            "-png",
            "-r",
            &dpi.to_string(),
            "-f",
            &page_num,
            "-l",
            &page_num,
            "-singlefile",
        ])
        .arg(path)
        .output()
        .map_err(|e| format!("Erro ao executar pdftoppm: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "pdftoppm falhou: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let img = image::load_from_memory(&output.stdout)
        .map_err(|e| format!("Falha ao decodificar PNG do PDF: {}", e))?
        .to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Ok(egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
}

fn try_bind_pdfium() -> Option<Pdfium> {
    if let Ok(bindings) =
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
    {
        return Some(Pdfium::new(bindings));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if let Ok(bindings) =
                Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(dir))
            {
                return Some(Pdfium::new(bindings));
            }
        }
    }

    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Some(Pdfium::new(bindings));
    }

    None
}

fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("-v")
        .output()
        .map(|_| true)
        .unwrap_or(false)
}

fn read_poppler_info(path: &Path) -> (usize, Option<String>, Option<String>) {
    let output = match Command::new("pdfinfo").arg(path).output() {
        Ok(out) if out.status.success() => out,
        _ => return (1, None, None),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut pages = 1usize;
    let mut title = None;
    let mut author = None;

    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("Pages:") {
            if let Ok(n) = rest.trim().parse::<usize>() {
                pages = n.max(1);
            }
        } else if let Some(rest) = line.strip_prefix("Title:") {
            let val = rest.trim();
            if !val.is_empty() {
                title = Some(val.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("Author:") {
            let val = rest.trim();
            if !val.is_empty() {
                author = Some(val.to_string());
            }
        }
    }

    (pages, title, author)
}

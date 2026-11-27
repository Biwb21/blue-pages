mod formats;
mod state;
mod theme;
mod ui;

use chrono::Utc;
use eframe::egui;
use formats::epub_loader::EpubDocument;
use formats::pdf_loader::PdfDocumentHandle;
use state::{BookFormat, LibraryState, RecentBook};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use theme::BluePagesTheme;
use ui::home::{HomeAction, HomeView};
use ui::reader::{ActiveDocument, ReaderAction, ReaderView};

const APP_ICON_PNG: &[u8] = include_bytes!("../assets/blue_pages_icon.png");

enum Screen {
    Home,
    Reader(ReaderView),
}

struct BluePagesApp {
    library: LibraryState,
    home: HomeView,
    screen: Screen,
    app_icon: Option<egui::TextureHandle>,
    cover_textures: HashMap<PathBuf, egui::TextureHandle>,
    error_banner: Option<String>,
}

impl BluePagesApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        BluePagesTheme::install_fonts(&cc.egui_ctx);
        BluePagesTheme::apply(&cc.egui_ctx);
        let library = LibraryState::load();

        let app_icon = formats::load_color_image_from_bytes(APP_ICON_PNG).map(|img| {
            cc.egui_ctx
                .load_texture("blue_pages_app_icon", img, egui::TextureOptions::LINEAR)
        });

        let mut app = Self {
            library,
            home: HomeView::default(),
            screen: Screen::Home,
            app_icon,
            cover_textures: HashMap::new(),
            error_banner: None,
        };
        app.preload_cached_covers(&cc.egui_ctx);
        app
    }

    fn preload_cached_covers(&mut self, ctx: &egui::Context) {
        for book in &self.library.recent_books {
            if let Some(cover_path) = &book.cover_cache_path {
                if !self.cover_textures.contains_key(&book.path) {
                    if let Some(color_img) = formats::load_color_image_from_path(cover_path) {
                        let tex = ctx.load_texture(
                            format!("cover_{}", book.path.display()),
                            color_img,
                            egui::TextureOptions::LINEAR,
                        );
                        self.cover_textures.insert(book.path.clone(), tex);
                    }
                }
            }
        }
    }

    fn open_file_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Blue Pages — Abrir livro (.epub ou .pdf)")
            .add_filter("Livros e Documentos", &["epub", "pdf"])
            .pick_file()
        {
            self.open_book(ctx, &path);
        }
    }

    fn open_book(&mut self, ctx: &egui::Context, path: &Path) {
        self.error_banner = None;

        let format = match BookFormat::from_path(path) {
            Some(f) => f,
            None => {
                self.error_banner = Some(
                    "Formato não suportado. Selecione um arquivo .epub ou .pdf.".to_string(),
                );
                return;
            }
        };

        let existing_index = self
            .library
            .recent_books
            .iter()
            .find(|b| b.path == path)
            .map(|b| b.current_index)
            .unwrap_or(0);

        let existing_cover = self
            .library
            .recent_books
            .iter()
            .find(|b| b.path == path)
            .and_then(|b| b.cover_cache_path.clone());

        match format {
            BookFormat::Epub => match EpubDocument::open(path) {
                Ok(doc) => {
                    let cover_cache_path = existing_cover.or_else(|| {
                        doc.cover_bytes
                            .as_deref()
                            .and_then(|bytes| formats::save_cover_to_cache(path, bytes))
                    });

                    if let Some(cover_path) = &cover_cache_path {
                        if let Some(img) = formats::load_color_image_from_path(cover_path) {
                            let tex = ctx.load_texture(
                                format!("cover_{}", path.display()),
                                img,
                                egui::TextureOptions::LINEAR,
                            );
                            self.cover_textures.insert(path.to_path_buf(), tex);
                        }
                    }

                    let recent = RecentBook {
                        path: path.to_path_buf(),
                        title: doc.title.clone(),
                        author: doc.author.clone(),
                        description: doc.description.clone(),
                        format: BookFormat::Epub,
                        current_index: existing_index.min(doc.chapters.len().saturating_sub(1)),
                        total_units: doc.chapters.len().max(1),
                        scroll_offset: 0.0,
                        last_opened: Utc::now(),
                        cover_cache_path,
                    };
                    self.library.upsert_book(recent);

                    let reader = ReaderView::new(
                        ActiveDocument::Epub {
                            doc,
                            inline_textures: HashMap::new(),
                        },
                        existing_index,
                    );
                    self.screen = Screen::Reader(reader);
                }
                Err(e) => self.error_banner = Some(e),
            },
            BookFormat::Pdf => match PdfDocumentHandle::open(path, ctx.clone()) {
                Ok(mut pdf) => {
                    let cover_cache_path = existing_cover.or_else(|| {
                        pdf.take_cover_image()
                            .and_then(|img| formats::save_color_image_cover(path, &img))
                    });

                    if let Some(cover_path) = &cover_cache_path {
                        if let Some(img) = formats::load_color_image_from_path(cover_path) {
                            let tex = ctx.load_texture(
                                format!("cover_{}", path.display()),
                                img,
                                egui::TextureOptions::LINEAR,
                            );
                            self.cover_textures.insert(path.to_path_buf(), tex);
                        }
                    }

                    let recent = RecentBook {
                        path: path.to_path_buf(),
                        title: pdf.title.clone(),
                        author: pdf.author.clone(),
                        description: format!(
                            "Documento PDF com {} páginas. Renderizado via {}.",
                            pdf.page_count,
                            pdf.backend_name()
                        ),
                        format: BookFormat::Pdf,
                        current_index: existing_index.min(pdf.page_count.saturating_sub(1)),
                        total_units: pdf.page_count.max(1),
                        scroll_offset: 0.0,
                        last_opened: Utc::now(),
                        cover_cache_path,
                    };
                    self.library.upsert_book(recent);

                    let reader = ReaderView::new(ActiveDocument::Pdf(pdf), existing_index);
                    self.screen = Screen::Reader(reader);
                }
                Err(e) => self.error_banner = Some(e),
            },
        }
    }
}

impl eframe::App for BluePagesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dropped_path: Option<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .find_map(|f| f.path.clone())
        });
        if let Some(path) = dropped_path {
            self.open_book(ctx, &path);
        }

        match &mut self.screen {
            Screen::Home => {
                let action = self.home.show(
                    ctx,
                    &self.library,
                    &self.cover_textures,
                    self.app_icon.as_ref(),
                    self.error_banner.as_deref(),
                );
                match action {
                    HomeAction::None => {}
                    HomeAction::OpenFileDialog => self.open_file_dialog(ctx),
                    HomeAction::OpenBook(path) => self.open_book(ctx, &path),
                    HomeAction::RemoveBook(path) => {
                        self.library.remove_book(&path);
                        self.cover_textures.remove(&path);
                    }
                }
            }
            Screen::Reader(reader) => {
                let (action, progress_changed) = reader.show(ctx, &mut self.library.settings);

                if progress_changed {
                    let path = reader.document.path().clone();
                    let idx = reader.current_index;
                    let total = reader.document.total_units();
                    self.library.update_progress(&path, idx, total, 0.0);
                }

                if let ReaderAction::BackToHome = action {
                    let path = reader.document.path().clone();
                    let idx = reader.current_index;
                    let total = reader.document.total_units();
                    self.library.update_progress(&path, idx, total, 0.0);
                    self.screen = Screen::Home;
                }
            }
        }
    }
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Blue Pages — Leitor de EPUB e PDF")
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([860.0, 600.0]);

    if let Ok(icon_data) = eframe::icon_data::from_png_bytes(APP_ICON_PNG) {
        viewport = viewport.with_icon(icon_data);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Blue Pages",
        options,
        Box::new(|cc| Ok(Box::new(BluePagesApp::new(cc)))),
    )
}

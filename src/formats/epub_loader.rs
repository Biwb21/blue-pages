use epub::doc::EpubDoc;
use regex::Regex;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub enum EpubBlock {
    Heading(String),
    Paragraph(String),
    Image {
        id: String,
        rgba: Vec<u8>,
        width: usize,
        height: usize,
    },
}

#[derive(Clone)]
pub struct EpubChapter {
    pub title: String,
    pub blocks: Vec<EpubBlock>,
}

pub struct EpubDocument {
    pub path: PathBuf,
    pub title: String,
    pub author: String,
    pub description: String,
    pub chapters: Vec<EpubChapter>,
    pub cover_bytes: Option<Vec<u8>>,
}

impl EpubDocument {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut doc = EpubDoc::new(path).map_err(|e| format!("Falha ao abrir EPUB: {}", e))?;

        let fallback_title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Livro sem título".to_string());

        let title = doc
            .mdata("title")
            .map(|m| m.value.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or(fallback_title);

        let author = doc
            .mdata("creator")
            .map(|m| m.value.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Autor desconhecido".to_string());

        let description = doc
            .mdata("description")
            .map(|m| strip_html_tags(&m.value))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Livro em formato EPUB pronto para leitura.".to_string());

        let cover_bytes = doc.get_cover().map(|(bytes, _mime)| bytes);

        let num_chapters = doc.get_num_chapters();
        let mut chapters = Vec::with_capacity(num_chapters);

        // Mapeia títulos do índice (TOC) pelo caminho do arquivo
        let mut toc_titles = std::collections::HashMap::new();
        for nav in &doc.toc {
            let clean_path = nav
                .content
                .to_string_lossy()
                .split('#')
                .next()
                .unwrap_or("")
                .to_string();
            if !clean_path.is_empty() && !nav.label.trim().is_empty() {
                toc_titles.insert(clean_path, nav.label.trim().to_string());
            }
        }

        let tag_regex = Regex::new(
            r#"(?is)<(h[1-3])[^>]*>(.*?)</h[1-3]>|<(p|li|blockquote)[^>]*>(.*?)</(p|li|blockquote)>|<img[^>]+src=["']([^"']+)["'][^>]*>|<image[^>]+(?:xlink:href|href)=["']([^"']+)["'][^>]*>"#,
        )
        .unwrap();

        for idx in 0..num_chapters {
            if !doc.set_current_chapter(idx) {
                continue;
            }

            let chapter_path = doc.get_current_path().unwrap_or_default();
            let chapter_dir = chapter_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_default();

            let html = match doc.get_current_str() {
                Some((content, _mime)) => content,
                None => continue,
            };

            let mut blocks = Vec::new();
            let mut first_heading: Option<String> = None;

            for (cap_idx, cap) in tag_regex.captures_iter(&html).enumerate() {
                if let Some(heading_inner) = cap.get(2) {
                    let text = strip_html_tags(heading_inner.as_str());
                    if !text.is_empty() {
                        if first_heading.is_none() {
                            first_heading = Some(text.clone());
                        }
                        blocks.push(EpubBlock::Heading(text));
                    }
                } else if let Some(para_inner) = cap.get(4) {
                    let text = strip_html_tags(para_inner.as_str());
                    if !text.is_empty() {
                        blocks.push(EpubBlock::Paragraph(text));
                    }
                } else if let Some(img_src) = cap.get(6).or_else(|| cap.get(7)) {
                    let raw_src = img_src.as_str().trim();
                    let resolved = resolve_epub_relative_path(&chapter_dir, raw_src);
                    if let Some(bytes) = doc
                        .get_resource_by_path(&resolved)
                        .or_else(|| doc.get_resource_by_path(raw_src))
                    {
                        if let Ok(img) = image::load_from_memory(&bytes) {
                            let rgba_img = img.to_rgba8();
                            blocks.push(EpubBlock::Image {
                                id: format!("epub_img_{}_{}", idx, cap_idx),
                                width: rgba_img.width() as usize,
                                height: rgba_img.height() as usize,
                                rgba: rgba_img.into_raw(),
                            });
                        }
                    }
                }
            }

            // Fallback caso o capítulo não use tags <p> padrão
            if blocks.is_empty() {
                let plain = strip_html_tags(&html);
                if !plain.is_empty() {
                    for line in plain.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            blocks.push(EpubBlock::Paragraph(trimmed.to_string()));
                        }
                    }
                }
            }

            if blocks.is_empty() {
                continue;
            }

            let chapter_path_str = chapter_path.to_string_lossy().to_string();
            let chapter_title = toc_titles
                .get(&chapter_path_str)
                .cloned()
                .or(first_heading)
                .unwrap_or_else(|| format!("Capítulo {}", chapters.len() + 1));

            chapters.push(EpubChapter {
                title: chapter_title,
                blocks,
            });
        }

        if chapters.is_empty() {
            chapters.push(EpubChapter {
                title: "Capítulo 1".to_string(),
                blocks: vec![EpubBlock::Paragraph(
                    "Nenhum conteúdo legível encontrado neste arquivo EPUB.".to_string(),
                )],
            });
        }

        Ok(Self {
            path: path.to_path_buf(),
            title,
            author,
            description,
            chapters,
            cover_bytes,
        })
    }
}

fn resolve_epub_relative_path(base_dir: &Path, rel: &str) -> PathBuf {
    let mut result = base_dir.to_path_buf();
    for part in rel.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                result.pop();
            }
            seg => result.push(seg),
        }
    }
    result
}

fn strip_html_tags(input: &str) -> String {
    let tag_re = Regex::new(r"(?is)<script[^>]*>.*?</script>|<style[^>]*>.*?</style>|<[^>]+>").unwrap();
    let without_tags = tag_re.replace_all(input, " ");
    let decoded = decode_html_entities(&without_tags);

    let ws_re = Regex::new(r"[ \t\r\n]+").unwrap();
    ws_re.replace_all(&decoded, " ").trim().to_string()
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
        .replace("&ldquo;", "“")
        .replace("&rdquo;", "”")
        .replace("&lsquo;", "‘")
        .replace("&rsquo;", "’")
}


impl EpubDocument {
    /// Estima página virtual (~1800 caracteres/página) e porcentagem ponderada de leitura
    pub fn page_and_percent(&self, chap_idx: usize) -> (usize, usize, f32) {
        let mut chapter_chars = Vec::with_capacity(self.chapters.len());
        let mut total_chars: usize = 0;

        for chap in &self.chapters {
            let mut c: usize = 0;
            for b in &chap.blocks {
                match b {
                    EpubBlock::Heading(t) | EpubBlock::Paragraph(t) => c += t.chars().count(),
                    EpubBlock::Image { .. } => c += 900,
                }
            }
            let weight = c.max(300);
            chapter_chars.push(weight);
            total_chars += weight;
        }

        let total_pages = ((total_chars as f32 / 1800.0).ceil() as usize)
            .max(self.chapters.len())
            .max(1);

        let clamped = chap_idx.min(self.chapters.len().saturating_sub(1));
        let chars_up_to: usize = chapter_chars.iter().take(clamped + 1).sum();
        let fraction = if total_chars == 0 {
            1.0
        } else {
            (chars_up_to as f32 / total_chars as f32).clamp(0.0, 1.0)
        };

        let current_page = ((fraction * total_pages as f32).round() as usize)
            .clamp(1, total_pages);

        (current_page, total_pages, fraction * 100.0)
    }
}

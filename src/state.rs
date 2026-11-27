use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BookFormat {
    Epub,
    Pdf,
}

impl BookFormat {
    pub fn label(&self) -> &'static str {
        match self {
            BookFormat::Epub => "EPUB",
            BookFormat::Pdf => "PDF",
        }
    }

    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "epub" => Some(BookFormat::Epub),
            "pdf" => Some(BookFormat::Pdf),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReadingMode {
    Paged,
    Vertical,
}

impl Default for ReadingMode {
    fn default() -> Self {
        ReadingMode::Paged
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpubFontChoice {
    DefaultSans,
    Caladea,
    Carlito,
    LiberationSerif,
    DejaVuSerif,
    LiberationMono,
}

impl Default for EpubFontChoice {
    fn default() -> Self {
        EpubFontChoice::Caladea
    }
}

impl EpubFontChoice {
    pub const ALL: &'static [EpubFontChoice] = &[
        EpubFontChoice::DefaultSans,
        EpubFontChoice::Caladea,
        EpubFontChoice::Carlito,
        EpubFontChoice::LiberationSerif,
        EpubFontChoice::DejaVuSerif,
        EpubFontChoice::LiberationMono,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            EpubFontChoice::DefaultSans => "Padrão (Sans)",
            EpubFontChoice::Caladea => "Caladea (Livro)",
            EpubFontChoice::Carlito => "Carlito (Humanista)",
            EpubFontChoice::LiberationSerif => "Liberation Serif",
            EpubFontChoice::DejaVuSerif => "DejaVu Serif",
            EpubFontChoice::LiberationMono => "Liberation Mono",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpubColorScheme {
    DarkBlue,
    Amoled,
    Sepia,
    Light,
    SlateNight,
}

impl Default for EpubColorScheme {
    fn default() -> Self {
        EpubColorScheme::DarkBlue
    }
}

impl EpubColorScheme {
    pub const ALL: &'static [EpubColorScheme] = &[
        EpubColorScheme::DarkBlue,
        EpubColorScheme::Amoled,
        EpubColorScheme::Sepia,
        EpubColorScheme::Light,
        EpubColorScheme::SlateNight,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            EpubColorScheme::DarkBlue => "Escuro",
            EpubColorScheme::Amoled => "AMOLED",
            EpubColorScheme::Sepia => "Sépia",
            EpubColorScheme::Light => "Claro",
            EpubColorScheme::SlateNight => "Noite Suave",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentBook {
    pub path: PathBuf,
    pub title: String,
    pub author: String,
    pub description: String,
    pub format: BookFormat,
    /// Página atual (0-indexed) para PDF, ou índice do capítulo (0-indexed) para EPUB
    pub current_index: usize,
    pub total_units: usize,
    pub scroll_offset: f32,
    pub last_opened: DateTime<Utc>,
    pub cover_cache_path: Option<PathBuf>,
}

impl RecentBook {
    pub fn progress_fraction(&self) -> f32 {
        if self.total_units == 0 {
            return 0.0;
        }
        ((self.current_index + 1) as f32 / self.total_units as f32).clamp(0.0, 1.0)
    }

    pub fn progress_percent(&self) -> u32 {
        (self.progress_fraction() * 100.0).round() as u32
    }

    pub fn progress_label(&self) -> String {
        let unit = match self.format {
            BookFormat::Epub => "Capítulo",
            BookFormat::Pdf => "Página",
        };
        let total = self.total_units.max(1);
        let current = (self.current_index + 1).min(total);
        format!("{} {} de {}  ·  {}%", unit, current, total, self.progress_percent())
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReaderSettings {
    pub mode: ReadingMode,
    /// Escala de zoom de 0.5 a 2.2 (1.0 = 100%)
    pub zoom: f32,
    /// Tamanho base da fonte para leitura de EPUB (em pt)
    pub epub_font_size: f32,
    /// Espaçamento entre páginas no modo rolagem vertical
    pub vertical_gap: f32,
    #[serde(default)]
    pub epub_font: EpubFontChoice,
    #[serde(default)]
    pub epub_theme: EpubColorScheme,
    #[serde(default = "default_true")]
    pub show_toc_sidebar: bool,
}

impl Default for ReaderSettings {
    fn default() -> Self {
        Self {
            mode: ReadingMode::Paged,
            zoom: 1.0,
            epub_font_size: 18.0,
            vertical_gap: 16.0,
            epub_font: EpubFontChoice::Caladea,
            epub_theme: EpubColorScheme::DarkBlue,
            show_toc_sidebar: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibraryState {
    pub recent_books: Vec<RecentBook>,
    pub settings: ReaderSettings,
}

impl LibraryState {
    pub fn data_dir() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("app", "bluepages", "BluePages") {
            let dir = proj_dirs.data_local_dir().to_path_buf();
            let _ = fs::create_dir_all(&dir);
            return dir;
        }
        PathBuf::from(".")
    }

    fn legacy_state_file() -> Option<PathBuf> {
        ProjectDirs::from("app", "karikari", "KarikariReader")
            .map(|d| d.data_local_dir().join("library.json"))
    }

    pub fn covers_dir() -> PathBuf {
        let dir = Self::data_dir().join("covers");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn state_file() -> PathBuf {
        Self::data_dir().join("library.json")
    }

    pub fn load() -> Self {
        let path = Self::state_file();
        let raw_opt = fs::read_to_string(&path).ok().or_else(|| {
            Self::legacy_state_file().and_then(|p| fs::read_to_string(p).ok())
        });
        if let Some(raw) = raw_opt {
            if let Ok(mut parsed) = serde_json::from_str::<LibraryState>(&raw) {
                parsed
                    .recent_books
                    .sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
                return parsed;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::state_file();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn upsert_book(&mut self, book: RecentBook) {
        self.recent_books.retain(|b| b.path != book.path);
        self.recent_books.insert(0, book);
        if self.recent_books.len() > 40 {
            self.recent_books.truncate(40);
        }
        self.save();
    }

    pub fn update_progress(
        &mut self,
        path: &Path,
        current_index: usize,
        total_units: usize,
        scroll_offset: f32,
    ) {
        if let Some(pos) = self.recent_books.iter().position(|b| b.path == path) {
            let mut item = self.recent_books.remove(pos);
            item.current_index = current_index;
            item.total_units = total_units;
            item.scroll_offset = scroll_offset;
            item.last_opened = Utc::now();
            self.recent_books.insert(0, item);
            self.save();
        }
    }

    pub fn remove_book(&mut self, path: &Path) {
        self.recent_books.retain(|b| b.path != path);
        self.save();
    }
}

pub mod epub_loader;
pub mod pdf_loader;

use crate::state::LibraryState;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Salva uma imagem de capa em cache local (PNG redimensionado) para exibição rápida na tela inicial
pub fn save_cover_to_cache(book_path: &Path, image_bytes: &[u8]) -> Option<PathBuf> {
    let img = image::load_from_memory(image_bytes).ok()?;
    let resized = img.thumbnail(640, 900);

    let mut hasher = DefaultHasher::new();
    book_path.hash(&mut hasher);
    let hash = hasher.finish();

    let out_path = LibraryState::covers_dir().join(format!("cover_{:x}.png", hash));
    resized.save(&out_path).ok()?;
    Some(out_path)
}

/// Salva um `egui::ColorImage` diretamente como PNG de capa em cache
pub fn save_color_image_cover(book_path: &Path, color_img: &egui::ColorImage) -> Option<PathBuf> {
    let width = color_img.size[0] as u32;
    let height = color_img.size[1] as u32;
    let rgba: Vec<u8> = color_img
        .pixels
        .iter()
        .flat_map(|c| c.to_array())
        .collect();

    let buf = image::RgbaImage::from_raw(width, height, rgba)?;
    let dyn_img = image::DynamicImage::ImageRgba8(buf);
    let resized = dyn_img.thumbnail(640, 900);

    let mut hasher = DefaultHasher::new();
    book_path.hash(&mut hasher);
    let hash = hasher.finish();

    let out_path = LibraryState::covers_dir().join(format!("cover_{:x}.png", hash));
    resized.save(&out_path).ok()?;
    Some(out_path)
}

pub fn load_color_image_from_bytes(bytes: &[u8]) -> Option<egui::ColorImage> {
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Some(egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
}

pub fn load_color_image_from_path(path: &Path) -> Option<egui::ColorImage> {
    let bytes = std::fs::read(path).ok()?;
    load_color_image_from_bytes(&bytes)
}

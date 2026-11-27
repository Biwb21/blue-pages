use crate::state::ReadingMode;
use crate::theme::BluePagesTheme;
use egui::{pos2, vec2, Color32, Rect, RichText, Rounding, Sense, Stroke, Vec2};

/// Botão com seta vetorial para a esquerda (voltar) — nunca vira □
pub fn back_arrow_button(ui: &mut egui::Ui) -> egui::Response {
    let size = vec2(32.0, 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let bg = if resp.hovered() {
        BluePagesTheme::BG_PILL
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, Rounding::same(8.0), bg);

    let c = rect.center();
    let stroke = Stroke::new(2.0_f32, BluePagesTheme::TEXT_PRIMARY);
    ui.painter()
        .line_segment([pos2(c.x + 6.0, c.y), pos2(c.x - 6.0, c.y)], stroke);
    ui.painter()
        .line_segment([pos2(c.x - 1.0, c.y - 5.0), pos2(c.x - 6.0, c.y)], stroke);
    ui.painter()
        .line_segment([pos2(c.x - 1.0, c.y + 5.0), pos2(c.x - 6.0, c.y)], stroke);
    resp
}

/// Ícone vetorial de lupa para a barra de busca
pub fn draw_search_icon(ui: &mut egui::Ui, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), Sense::hover());
    let c = rect.center() - vec2(1.5, 1.5);
    let stroke = Stroke::new(1.6_f32, color);
    ui.painter().circle_stroke(c, 4.5, stroke);
    ui.painter()
        .line_segment([pos2(c.x + 3.2, c.y + 3.2), pos2(c.x + 6.8, c.y + 6.8)], stroke);
}

/// Botão pill com ícone vetorial de Sumário (3 linhas de índice)
pub fn toc_toggle_button(ui: &mut egui::Ui, active: bool) -> egui::Response {
    let (fg, stroke_col) = if active {
        (BluePagesTheme::ACCENT, BluePagesTheme::ACCENT)
    } else {
        (BluePagesTheme::TEXT_PRIMARY, BluePagesTheme::BORDER)
    };
    let size = vec2(96.0, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if resp.hovered() {
        BluePagesTheme::BG_CARD_HOVER
    } else {
        BluePagesTheme::BG_PILL
    };
    ui.painter().rect_filled(rect, Rounding::same(15.0), fill);
    ui.painter()
        .rect_stroke(rect, Rounding::same(15.0), Stroke::new(1.0_f32, stroke_col));

    // Desenha ícone de lista (3 linhas)
    let x0 = rect.left() + 12.0;
    let cy = rect.center().y;
    let s = Stroke::new(1.6_f32, fg);
    for dy in [-4.0_f32, 0.0, 4.0] {
        ui.painter()
            .line_segment([pos2(x0, cy + dy), pos2(x0 + 11.0, cy + dy)], s);
    }

    ui.painter().text(
        pos2(x0 + 18.0, cy),
        egui::Align2::LEFT_CENTER,
        "Sumário",
        egui::FontId::proportional(12.8),
        fg,
    );
    resp
}

/// Botão pill com ícone vetorial de alternância (Por páginas / Rolagem vertical)
pub fn mode_toggle_button(ui: &mut egui::Ui, mode: ReadingMode) -> egui::Response {
    let label = match mode {
        ReadingMode::Paged => "Por páginas",
        ReadingMode::Vertical => "Rolagem",
    };
    let size = vec2(118.0, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if resp.hovered() {
        BluePagesTheme::BG_CARD_HOVER
    } else {
        BluePagesTheme::BG_PILL
    };
    ui.painter().rect_filled(rect, Rounding::same(15.0), fill);
    ui.painter().rect_stroke(
        rect,
        Rounding::same(15.0),
        Stroke::new(1.0_f32, BluePagesTheme::ACCENT),
    );

    let x0 = rect.left() + 12.0;
    let cy = rect.center().y;
    let s = Stroke::new(1.5_f32, BluePagesTheme::ACCENT);

    match mode {
        ReadingMode::Paged => {
            // Ícone de página/livro aberto
            let page_rect = Rect::from_min_size(pos2(x0, cy - 6.0), vec2(12.0, 12.0));
            ui.painter()
                .rect_stroke(page_rect, Rounding::same(2.0), s);
            ui.painter().line_segment(
                [pos2(x0 + 6.0, cy - 6.0), pos2(x0 + 6.0, cy + 6.0)],
                s,
            );
        }
        ReadingMode::Vertical => {
            // Ícone de rolagem vertical (seta cima/baixo)
            let cx = x0 + 6.0;
            ui.painter()
                .line_segment([pos2(cx, cy - 6.0), pos2(cx, cy + 6.0)], s);
            ui.painter()
                .line_segment([pos2(cx - 3.0, cy - 3.0), pos2(cx, cy - 6.0)], s);
            ui.painter()
                .line_segment([pos2(cx + 3.0, cy - 3.0), pos2(cx, cy - 6.0)], s);
            ui.painter()
                .line_segment([pos2(cx - 3.0, cy + 3.0), pos2(cx, cy + 6.0)], s);
            ui.painter()
                .line_segment([pos2(cx + 3.0, cy + 3.0), pos2(cx, cy + 6.0)], s);
        }
    }

    ui.painter().text(
        pos2(x0 + 18.0, cy),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(12.8),
        BluePagesTheme::ACCENT,
    );
    resp
}

/// Botão pill com ícone vetorial de Ajustes (sliders horizontais)
pub fn settings_toggle_button(ui: &mut egui::Ui, active: bool) -> egui::Response {
    let (fg, stroke_col) = if active {
        (BluePagesTheme::ACCENT, BluePagesTheme::ACCENT)
    } else {
        (BluePagesTheme::TEXT_PRIMARY, BluePagesTheme::BORDER)
    };
    let size = vec2(92.0, 30.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if resp.hovered() {
        BluePagesTheme::BG_CARD_HOVER
    } else {
        BluePagesTheme::BG_PILL
    };
    ui.painter().rect_filled(rect, Rounding::same(15.0), fill);
    ui.painter()
        .rect_stroke(rect, Rounding::same(15.0), Stroke::new(1.0_f32, stroke_col));

    // Ícone de sliders (3 linhas com bolinhas)
    let x0 = rect.left() + 12.0;
    let cy = rect.center().y;
    let s = Stroke::new(1.4_f32, fg);
    let offsets = [(-4.0_f32, 3.0_f32), (0.0, 8.5), (4.0, 4.5)];
    for (dy, dot_x) in offsets {
        ui.painter()
            .line_segment([pos2(x0, cy + dy), pos2(x0 + 12.0, cy + dy)], s);
        ui.painter().circle_filled(pos2(x0 + dot_x, cy + dy), 2.0, fg);
    }

    ui.painter().text(
        pos2(x0 + 18.0, cy),
        egui::Align2::LEFT_CENTER,
        "Ajustes",
        egui::FontId::proportional(12.8),
        fg,
    );
    resp
}

/// Botão principal azul-ararinha com triângulo vetorial opcional (sem glifo Unicode que falha!)
pub fn primary_play_button(ui: &mut egui::Ui, size: Vec2, label: &str) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let fill = if resp.hovered() {
        Color32::from_rgb(96, 206, 252)
    } else {
        BluePagesTheme::ACCENT
    };
    ui.painter().rect_filled(rect, Rounding::same(22.0), fill);

    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(14.5),
        BluePagesTheme::ACCENT_TEXT,
    );
    let icon_w = 10.0;
    let gap = 8.0;
    let total_w = icon_w + gap + galley.size().x;
    let start_x = rect.center().x - total_w * 0.5;
    let cy = rect.center().y;

    // Triângulo Play vetorial
    let tri = vec![
        pos2(start_x, cy - 5.5),
        pos2(start_x, cy + 5.5),
        pos2(start_x + 9.0, cy),
    ];
    ui.painter()
        .add(egui::Shape::convex_polygon(tri, BluePagesTheme::ACCENT_TEXT, Stroke::NONE));

    ui.painter().galley(
        pos2(start_x + icon_w + gap, cy - galley.size().y * 0.5),
        galley,
        BluePagesTheme::ACCENT_TEXT,
    );
    resp
}

/// Texto auxiliar sem depender de símbolos de seta Unicode
pub fn small_pill_text(text: impl Into<String>, color: Color32) -> RichText {
    RichText::new(text).size(12.5).color(color)
}

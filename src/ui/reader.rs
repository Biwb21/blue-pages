use crate::formats::epub_loader::{EpubBlock, EpubDocument};
use crate::formats::pdf_loader::{PageTextureState, PdfDocumentHandle};
use crate::state::{EpubColorScheme, EpubFontChoice, ReaderSettings, ReadingMode};
use crate::theme::BluePagesTheme;
use crate::ui::icons;
use egui::{Align, Color32, FontId, Key, Layout, Margin, RichText, Rounding, Sense, Stroke, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;

pub enum ActiveDocument {
    Epub {
        doc: EpubDocument,
        inline_textures: HashMap<String, egui::TextureHandle>,
    },
    Pdf(PdfDocumentHandle),
}

impl ActiveDocument {
    pub fn path(&self) -> &PathBuf {
        match self {
            ActiveDocument::Epub { doc, .. } => &doc.path,
            ActiveDocument::Pdf(pdf) => &pdf.path,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            ActiveDocument::Epub { doc, .. } => &doc.title,
            ActiveDocument::Pdf(pdf) => &pdf.title,
        }
    }

    pub fn total_units(&self) -> usize {
        match self {
            ActiveDocument::Epub { doc, .. } => doc.chapters.len().max(1),
            ActiveDocument::Pdf(pdf) => pdf.page_count.max(1),
        }
    }

    pub fn subtitle(&self, index: usize) -> String {
        match self {
            ActiveDocument::Epub { doc, .. } => doc
                .chapters
                .get(index)
                .map(|c| c.title.clone())
                .unwrap_or_else(|| format!("Capítulo {}", index + 1)),
            ActiveDocument::Pdf(pdf) => {
                format!("PÁGINA {} DE {}", index + 1, pdf.page_count.max(1))
            }
        }
    }

    pub fn is_epub(&self) -> bool {
        matches!(self, ActiveDocument::Epub { .. })
    }

    /// Retorna `(pagina_atual, total_paginas, porcentagem_0_a_100)`
    pub fn page_and_percent(&self, index: usize) -> (usize, usize, f32) {
        match self {
            ActiveDocument::Epub { doc, .. } => doc.page_and_percent(index),
            ActiveDocument::Pdf(pdf) => {
                let total = pdf.page_count.max(1);
                let cur = (index + 1).min(total);
                let pct = (cur as f32 / total as f32) * 100.0;
                (cur, total, pct)
            }
        }
    }
}

pub enum ReaderAction {
    None,
    BackToHome,
}

pub struct ReaderView {
    pub document: ActiveDocument,
    pub current_index: usize,
    pub show_settings_panel: bool,
    pub is_fullscreen: bool,
    pub page_input_buf: String,
    pub scroll_to_index: Option<usize>,
}

impl ReaderView {
    pub fn new(document: ActiveDocument, start_index: usize) -> Self {
        let clamped = start_index.min(document.total_units().saturating_sub(1));
        Self {
            document,
            current_index: clamped,
            show_settings_panel: false,
            is_fullscreen: false,
            page_input_buf: (clamped + 1).to_string(),
            scroll_to_index: Some(clamped),
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        settings: &mut ReaderSettings,
    ) -> (ReaderAction, bool) {
        if let ActiveDocument::Pdf(pdf) = &mut self.document {
            pdf.poll_results(ctx);
        }

        let mut action = ReaderAction::None;
        let mut progress_changed = false;
        let total = self.document.total_units();

        // Atalhos de teclado (send_viewport_cmd sempre FORA de ctx.input para evitar deadlock)
        let mut toggle_fullscreen: Option<bool> = None;
        if !ctx.wants_keyboard_input() {
            ctx.input(|i| {
                if i.key_pressed(Key::ArrowRight) && self.current_index + 1 < total {
                    self.current_index += 1;
                    self.page_input_buf = (self.current_index + 1).to_string();
                    self.scroll_to_index = Some(self.current_index);
                    progress_changed = true;
                }
                if i.key_pressed(Key::ArrowLeft) && self.current_index > 0 {
                    self.current_index -= 1;
                    self.page_input_buf = (self.current_index + 1).to_string();
                    self.scroll_to_index = Some(self.current_index);
                    progress_changed = true;
                }
                if i.key_pressed(Key::F) {
                    self.is_fullscreen = !self.is_fullscreen;
                    toggle_fullscreen = Some(self.is_fullscreen);
                }
                if i.key_pressed(Key::Escape) {
                    if self.is_fullscreen {
                        self.is_fullscreen = false;
                        toggle_fullscreen = Some(false);
                    } else if self.show_settings_panel {
                        self.show_settings_panel = false;
                    }
                }
                if i.key_pressed(Key::Plus) || i.key_pressed(Key::Equals) {
                    settings.zoom = (settings.zoom + 0.1).min(2.2);
                }
                if i.key_pressed(Key::Minus) {
                    settings.zoom = (settings.zoom - 0.1).max(0.5);
                }
            });
        }
        if let Some(fs) = toggle_fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(fs));
        }

        let (cur_page, total_pages, percent) = self.document.page_and_percent(self.current_index);

        // Barra superior do leitor (ícones 100% vetoriais — zero glifos □)
        egui::TopBottomPanel::top("blue_pages_reader_topbar")
            .exact_height(54.0)
            .frame(
                egui::Frame::none()
                    .fill(BluePagesTheme::BG_READER_BAR)
                    .stroke(Stroke::new(1.0_f32, BluePagesTheme::BORDER))
                    .inner_margin(Margin::symmetric(16.0, 6.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if icons::back_arrow_button(ui)
                        .on_hover_text("Voltar para Início")
                        .clicked()
                    {
                        if self.is_fullscreen {
                            self.is_fullscreen = false;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                        }
                        action = ReaderAction::BackToHome;
                    }

                    ui.add_space(6.0);

                    if icons::toc_toggle_button(ui, settings.show_toc_sidebar)
                        .on_hover_text("Mostrar ou ocultar o sumário lateral")
                        .clicked()
                    {
                        settings.show_toc_sidebar = !settings.show_toc_sidebar;
                        progress_changed = true;
                    }

                    ui.add_space(10.0);

                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.label(
                            RichText::new(self.document.title().to_uppercase())
                                .size(11.0)
                                .color(BluePagesTheme::TEXT_SECONDARY),
                        );
                        ui.label(
                            RichText::new(self.document.subtitle(self.current_index))
                                .size(14.0)
                                .strong()
                                .color(BluePagesTheme::TEXT_PRIMARY),
                        );
                    });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if icons::settings_toggle_button(ui, self.show_settings_panel)
                            .on_hover_text("Fontes, cores e ajustes de leitura")
                            .clicked()
                        {
                            self.show_settings_panel = !self.show_settings_panel;
                        }

                        ui.add_space(8.0);

                        if icons::mode_toggle_button(ui, settings.mode)
                            .on_hover_text("Alternar entre Por páginas e Rolagem vertical")
                            .clicked()
                        {
                            settings.mode = match settings.mode {
                                ReadingMode::Paged => ReadingMode::Vertical,
                                ReadingMode::Vertical => ReadingMode::Paged,
                            };
                            self.scroll_to_index = Some(self.current_index);
                            progress_changed = true;
                        }

                        ui.add_space(10.0);

                        let next_enabled = self.current_index + 1 < total;
                        if ui
                            .add_enabled(
                                next_enabled,
                                egui::Button::new(
                                    RichText::new("»")
                                        .size(19.0)
                                        .color(BluePagesTheme::TEXT_PRIMARY),
                                )
                                .frame(false),
                            )
                            .on_hover_text("Próximo")
                            .clicked()
                        {
                            self.current_index += 1;
                            self.page_input_buf = (self.current_index + 1).to_string();
                            self.scroll_to_index = Some(self.current_index);
                            progress_changed = true;
                        }

                        let prev_enabled = self.current_index > 0;
                        if ui
                            .add_enabled(
                                prev_enabled,
                                egui::Button::new(
                                    RichText::new("«")
                                        .size(19.0)
                                        .color(BluePagesTheme::TEXT_PRIMARY),
                                )
                                .frame(false),
                            )
                            .on_hover_text("Anterior")
                            .clicked()
                        {
                            self.current_index -= 1;
                            self.page_input_buf = (self.current_index + 1).to_string();
                            self.scroll_to_index = Some(self.current_index);
                            progress_changed = true;
                        }
                    });
                });
            });

        // Barra inferior com barra de progresso + páginas + porcentagem + controle de zoom
        egui::TopBottomPanel::bottom("blue_pages_reader_bottombar")
            .exact_height(52.0)
            .frame(
                egui::Frame::none()
                    .fill(BluePagesTheme::BG_READER_BAR)
                    .stroke(Stroke::new(1.0_f32, BluePagesTheme::BORDER))
                    .inner_margin(Margin::symmetric(16.0, 6.0)),
            )
            .show(ctx, |ui| {
                // Linha fina de progresso no topo da barra inferior
                let panel_rect = ui.max_rect();
                let bar_rect = egui::Rect::from_min_max(
                    egui::pos2(panel_rect.left() - 16.0, panel_rect.top() - 6.0),
                    egui::pos2(panel_rect.right() + 16.0, panel_rect.top() - 3.0),
                );
                let fill_w = bar_rect.width() * (percent / 100.0).clamp(0.0, 1.0);
                ui.painter()
                    .rect_filled(bar_rect, Rounding::ZERO, BluePagesTheme::PROGRESS_BG);
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(bar_rect.min, Vec2::new(fill_w, bar_rect.height())),
                    Rounding::ZERO,
                    BluePagesTheme::ACCENT,
                );

                ui.horizontal_centered(|ui| {
                    let status_text = if self.document.is_epub() {
                        format!(
                            "Pág. ~{} de {}  ·  Cap. {} de {}  ·  {:.0}%  ·  Setas Esq/Dir  ·  F tela cheia",
                            cur_page,
                            total_pages,
                            self.current_index + 1,
                            total,
                            percent
                        )
                    } else {
                        format!(
                            "Página {} de {}  ·  {:.0}%  ·  Setas Esq/Dir  ·  F tela cheia",
                            cur_page, total_pages, percent
                        )
                    };

                    egui::Frame::none()
                        .fill(BluePagesTheme::BG_PILL)
                        .rounding(Rounding::same(16.0))
                        .inner_margin(Margin::symmetric(14.0, 5.0))
                        .show(ui, |ui| {
                            ui.label(icons::small_pill_text(
                                status_text,
                                BluePagesTheme::TEXT_SECONDARY,
                            ));
                        });

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let pct = (settings.zoom * 100.0).round() as i32;
                        ui.label(
                            RichText::new(format!("{}%", pct))
                                .size(12.5)
                                .color(BluePagesTheme::TEXT_SECONDARY),
                        );

                        ui.add_space(6.0);

                        ui.scope(|ui| {
                            ui.visuals_mut().selection.bg_fill = BluePagesTheme::ACCENT;
                            ui.visuals_mut().widgets.inactive.bg_fill = BluePagesTheme::PROGRESS_BG;
                            ui.add(
                                egui::Slider::new(&mut settings.zoom, 0.5..=2.2)
                                    .show_value(false)
                                    .trailing_fill(true),
                            );
                        });

                        ui.add_space(6.0);

                        let reset_btn = ui.add(
                            egui::Button::new(
                                RichText::new(format!("{}%", pct))
                                    .size(12.0)
                                    .strong()
                                    .color(BluePagesTheme::ACCENT),
                            )
                            .fill(BluePagesTheme::BG_READER_BAR)
                            .stroke(Stroke::new(1.0_f32, BluePagesTheme::ACCENT))
                            .rounding(Rounding::same(6.0)),
                        );
                        if reset_btn.on_hover_text("Resetar zoom para 100%").clicked() {
                            settings.zoom = 1.0;
                        }
                    });
                });
            });

        // Painel lateral ESQUERDO: Sumário / Navegação do livro com destaque de onde você está
        if settings.show_toc_sidebar {
            egui::SidePanel::left("blue_pages_toc_sidebar")
                .exact_width(270.0)
                .frame(
                    egui::Frame::none()
                        .fill(BluePagesTheme::BG_NAVBAR)
                        .stroke(Stroke::new(1.0_f32, BluePagesTheme::BORDER))
                        .inner_margin(Margin::same(14.0)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Sumário")
                                .size(16.5)
                                .strong()
                                .color(BluePagesTheme::TEXT_PRIMARY),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("×")
                                            .size(16.0)
                                            .color(BluePagesTheme::TEXT_MUTED),
                                    )
                                    .frame(false),
                                )
                                .on_hover_text("Ocultar sumário")
                                .clicked()
                            {
                                settings.show_toc_sidebar = false;
                                progress_changed = true;
                            }
                        });
                    });

                    ui.add_space(8.0);

                    // Cartão compacto de progresso atual (páginas + porcentagem)
                    egui::Frame::none()
                        .fill(BluePagesTheme::BG_CARD)
                        .rounding(Rounding::same(10.0))
                        .stroke(Stroke::new(1.0_f32, BluePagesTheme::BORDER))
                        .inner_margin(Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let page_lbl = if self.document.is_epub() {
                                    format!("Pág. ~{} de {}", cur_page, total_pages)
                                } else {
                                    format!("Página {} de {}", cur_page, total_pages)
                                };
                                ui.label(
                                    RichText::new(page_lbl)
                                        .size(12.5)
                                        .color(BluePagesTheme::TEXT_SECONDARY),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{:.0}%", percent))
                                            .size(13.0)
                                            .strong()
                                            .color(BluePagesTheme::ACCENT),
                                    );
                                });
                            });
                            ui.add_space(6.0);
                            let bar_w = ui.available_width();
                            let (bar_rect, _) =
                                ui.allocate_exact_size(Vec2::new(bar_w, 5.0), Sense::hover());
                            ui.painter().rect_filled(
                                bar_rect,
                                Rounding::same(3.0),
                                BluePagesTheme::PROGRESS_BG,
                            );
                            let fill_rect = egui::Rect::from_min_size(
                                bar_rect.min,
                                Vec2::new(
                                    (bar_rect.width() * (percent / 100.0)).max(6.0),
                                    bar_rect.height(),
                                ),
                            );
                            ui.painter().rect_filled(
                                fill_rect,
                                Rounding::same(3.0),
                                BluePagesTheme::ACCENT,
                            );
                        });

                    ui.add_space(12.0);

                    match &self.document {
                        ActiveDocument::Epub { doc, .. } => {
                            egui::ScrollArea::vertical()
                                .id_salt("toc_epub_scroll")
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for (idx, chap) in doc.chapters.iter().enumerate() {
                                        let is_current = idx == self.current_index;
                                        let (row_rect, resp) = ui.allocate_exact_size(
                                            Vec2::new(ui.available_width(), 34.0),
                                            Sense::click(),
                                        );

                                        let bg = if is_current {
                                            BluePagesTheme::ACCENT_SOFT
                                        } else if resp.hovered() {
                                            BluePagesTheme::BG_CARD
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        ui.painter()
                                            .rect_filled(row_rect, Rounding::same(7.0), bg);

                                        if is_current {
                                            // Barra lateral azul indicando exatamente onde você está
                                            let indicator = egui::Rect::from_min_size(
                                                egui::pos2(row_rect.left(), row_rect.top() + 5.0),
                                                Vec2::new(3.5, row_rect.height() - 10.0),
                                            );
                                            ui.painter().rect_filled(
                                                indicator,
                                                Rounding::same(2.0),
                                                BluePagesTheme::ACCENT,
                                            );
                                        }

                                        let title_short = if chap.title.chars().count() > 26 {
                                            format!(
                                                "{}...",
                                                chap.title.chars().take(26).collect::<String>()
                                            )
                                        } else {
                                            chap.title.clone()
                                        };

                                        let text_col = if is_current {
                                            BluePagesTheme::TEXT_PRIMARY
                                        } else {
                                            BluePagesTheme::TEXT_SECONDARY
                                        };

                                        ui.painter().text(
                                            egui::pos2(row_rect.left() + 12.0, row_rect.center().y),
                                            egui::Align2::LEFT_CENTER,
                                            format!("{}. {}", idx + 1, title_short),
                                            FontId::proportional(13.0),
                                            text_col,
                                        );

                                        if resp.clicked() {
                                            self.current_index = idx;
                                            self.scroll_to_index = Some(idx);
                                            progress_changed = true;
                                        }
                                    }
                                });
                        }
                        ActiveDocument::Pdf(pdf) => {
                            let pdf_total = pdf.page_count;
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("Ir para:")
                                        .size(12.5)
                                        .color(BluePagesTheme::TEXT_SECONDARY),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.page_input_buf)
                                        .desired_width(54.0),
                                );
                                if ui.button("Ir").clicked() {
                                    if let Ok(p) = self.page_input_buf.trim().parse::<usize>() {
                                        if p >= 1 && p <= pdf_total {
                                            self.current_index = p - 1;
                                            self.scroll_to_index = Some(self.current_index);
                                            progress_changed = true;
                                        }
                                    }
                                }
                            });
                            ui.add_space(8.0);

                            egui::ScrollArea::vertical()
                                .id_salt("toc_pdf_scroll")
                                .auto_shrink([false, false])
                                .show_rows(ui, 30.0, pdf_total, |ui, row_range| {
                                    for idx in row_range {
                                        let is_current = idx == self.current_index;
                                        let (row_rect, resp) = ui.allocate_exact_size(
                                            Vec2::new(ui.available_width(), 28.0),
                                            Sense::click(),
                                        );
                                        let bg = if is_current {
                                            BluePagesTheme::ACCENT_SOFT
                                        } else if resp.hovered() {
                                            BluePagesTheme::BG_CARD
                                        } else {
                                            Color32::TRANSPARENT
                                        };
                                        ui.painter()
                                            .rect_filled(row_rect, Rounding::same(6.0), bg);

                                        if is_current {
                                            let indicator = egui::Rect::from_min_size(
                                                egui::pos2(row_rect.left(), row_rect.top() + 4.0),
                                                Vec2::new(3.5, row_rect.height() - 8.0),
                                            );
                                            ui.painter().rect_filled(
                                                indicator,
                                                Rounding::same(2.0),
                                                BluePagesTheme::ACCENT,
                                            );
                                        }

                                        let pct_page =
                                            ((idx + 1) as f32 / pdf_total.max(1) as f32) * 100.0;
                                        ui.painter().text(
                                            egui::pos2(row_rect.left() + 12.0, row_rect.center().y),
                                            egui::Align2::LEFT_CENTER,
                                            format!("Página {}", idx + 1),
                                            FontId::proportional(13.0),
                                            if is_current {
                                                BluePagesTheme::TEXT_PRIMARY
                                            } else {
                                                BluePagesTheme::TEXT_SECONDARY
                                            },
                                        );
                                        ui.painter().text(
                                            egui::pos2(row_rect.right() - 10.0, row_rect.center().y),
                                            egui::Align2::RIGHT_CENTER,
                                            format!("{:.0}%", pct_page),
                                            FontId::proportional(11.5),
                                            BluePagesTheme::TEXT_MUTED,
                                        );

                                        if resp.clicked() {
                                            self.current_index = idx;
                                            self.page_input_buf = (idx + 1).to_string();
                                            self.scroll_to_index = Some(idx);
                                            progress_changed = true;
                                        }
                                    }
                                });
                        }
                    }
                });
        }

        // Painel lateral DIREITO: Ajustes (Modo, Cores do EPUB, Fontes do EPUB, Tamanho da fonte)
        if self.show_settings_panel {
            egui::SidePanel::right("blue_pages_reader_settings")
                .exact_width(290.0)
                .frame(
                    egui::Frame::none()
                        .fill(BluePagesTheme::BG_CARD)
                        .stroke(Stroke::new(1.0_f32, BluePagesTheme::BORDER))
                        .inner_margin(Margin::same(16.0)),
                )
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label(
                            RichText::new("Ajustes de Leitura")
                                .size(17.0)
                                .strong()
                                .color(BluePagesTheme::TEXT_PRIMARY),
                        );
                        ui.add_space(12.0);

                        // Modo de exibição
                        ui.label(
                            RichText::new("Modo de exibição")
                                .size(13.0)
                                .strong()
                                .color(BluePagesTheme::TEXT_SECONDARY),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    settings.mode == ReadingMode::Paged,
                                    "Por páginas",
                                )
                                .clicked()
                            {
                                settings.mode = ReadingMode::Paged;
                                progress_changed = true;
                            }
                            if ui
                                .selectable_label(
                                    settings.mode == ReadingMode::Vertical,
                                    "Rolagem vertical",
                                )
                                .clicked()
                            {
                                settings.mode = ReadingMode::Vertical;
                                self.scroll_to_index = Some(self.current_index);
                                progress_changed = true;
                            }
                        });

                        ui.add_space(16.0);

                        if self.document.is_epub() {
                            // 1. Estilo de cores do EPUB
                            ui.label(
                                RichText::new("Estilo de cores do EPUB")
                                    .size(13.0)
                                    .strong()
                                    .color(BluePagesTheme::TEXT_SECONDARY),
                            );
                            ui.add_space(6.0);

                            for &scheme in EpubColorScheme::ALL {
                                let pal = BluePagesTheme::epub_palette(scheme);
                                let selected = settings.epub_theme == scheme;
                                let (rect, resp) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 34.0),
                                    Sense::click(),
                                );
                                let bg = if selected {
                                    BluePagesTheme::ACCENT_SOFT
                                } else if resp.hovered() {
                                    BluePagesTheme::BG_CARD_HOVER
                                } else {
                                    BluePagesTheme::BG_INPUT
                                };
                                ui.painter().rect_filled(rect, Rounding::same(8.0), bg);
                                ui.painter().rect_stroke(
                                    rect,
                                    Rounding::same(8.0),
                                    Stroke::new(
                                        if selected { 1.5_f32 } else { 1.0_f32 },
                                        if selected {
                                            BluePagesTheme::ACCENT
                                        } else {
                                            BluePagesTheme::BORDER
                                        },
                                    ),
                                );

                                // Bolinha de amostra da cor do papel + texto
                                let dot_center = egui::pos2(rect.left() + 18.0, rect.center().y);
                                ui.painter()
                                    .circle_filled(dot_center, 8.5, pal.swatch_preview);
                                ui.painter().circle_stroke(
                                    dot_center,
                                    8.5,
                                    Stroke::new(1.0_f32, pal.heading),
                                );

                                ui.painter().text(
                                    egui::pos2(rect.left() + 36.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    scheme.label(),
                                    FontId::proportional(13.5),
                                    BluePagesTheme::TEXT_PRIMARY,
                                );

                                if resp.clicked() {
                                    settings.epub_theme = scheme;
                                    progress_changed = true;
                                }
                                ui.add_space(4.0);
                            }

                            ui.add_space(14.0);

                            // 2. Personalização de fontes do EPUB (cada opção renderizada na própria fonte!)
                            ui.label(
                                RichText::new("Fonte do EPUB")
                                    .size(13.0)
                                    .strong()
                                    .color(BluePagesTheme::TEXT_SECONDARY),
                            );
                            ui.add_space(6.0);

                            for &font_choice in EpubFontChoice::ALL {
                                let selected = settings.epub_font == font_choice;
                                let family = BluePagesTheme::epub_font_family(font_choice);
                                let (rect, resp) = ui.allocate_exact_size(
                                    Vec2::new(ui.available_width(), 32.0),
                                    Sense::click(),
                                );
                                let bg = if selected {
                                    BluePagesTheme::ACCENT_SOFT
                                } else if resp.hovered() {
                                    BluePagesTheme::BG_CARD_HOVER
                                } else {
                                    BluePagesTheme::BG_INPUT
                                };
                                ui.painter().rect_filled(rect, Rounding::same(8.0), bg);
                                if selected {
                                    ui.painter().rect_stroke(
                                        rect,
                                        Rounding::same(8.0),
                                        Stroke::new(1.2_f32, BluePagesTheme::ACCENT),
                                    );
                                }
                                ui.painter().text(
                                    egui::pos2(rect.left() + 12.0, rect.center().y),
                                    egui::Align2::LEFT_CENTER,
                                    font_choice.label(),
                                    FontId::new(14.0, family),
                                    if selected {
                                        BluePagesTheme::ACCENT
                                    } else {
                                        BluePagesTheme::TEXT_PRIMARY
                                    },
                                );
                                if resp.clicked() {
                                    settings.epub_font = font_choice;
                                    progress_changed = true;
                                }
                                ui.add_space(4.0);
                            }

                            ui.add_space(14.0);

                            // 3. Tamanho da fonte
                            ui.label(
                                RichText::new(format!(
                                    "Tamanho da fonte: {:.0} pt",
                                    settings.epub_font_size
                                ))
                                .size(13.0)
                                .strong()
                                .color(BluePagesTheme::TEXT_SECONDARY),
                            );
                            if ui
                                .add(
                                    egui::Slider::new(&mut settings.epub_font_size, 13.0..=28.0)
                                        .show_value(false),
                                )
                                .changed()
                            {
                                progress_changed = true;
                            }
                        } else if let ActiveDocument::Pdf(pdf) = &self.document {
                            ui.label(
                                RichText::new(format!("Motor PDF: {}", pdf.backend_name()))
                                    .size(12.5)
                                    .color(BluePagesTheme::TEXT_MUTED),
                            );
                        }
                    });
                });
        }

        // Fundo da área de leitura acompanha o tema escolhido para EPUB
        let central_bg = if self.document.is_epub() {
            BluePagesTheme::epub_palette(settings.epub_theme).reader_bg
        } else {
            BluePagesTheme::BG_READER
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(central_bg))
            .show(ctx, |ui| {
                let panel_rect = ui.max_rect();

                match settings.mode {
                    ReadingMode::Paged => {
                        self.render_paged_mode(ctx, ui, settings);

                        if self.current_index > 0
                            && draw_floating_nav_arrow(
                                ui,
                                egui::pos2(panel_rect.left() + 34.0, panel_rect.center().y),
                                "‹",
                            )
                        {
                            self.current_index -= 1;
                            self.page_input_buf = (self.current_index + 1).to_string();
                            progress_changed = true;
                        }

                        if self.current_index + 1 < total
                            && draw_floating_nav_arrow(
                                ui,
                                egui::pos2(panel_rect.right() - 34.0, panel_rect.center().y),
                                "›",
                            )
                        {
                            self.current_index += 1;
                            self.page_input_buf = (self.current_index + 1).to_string();
                            progress_changed = true;
                        }
                    }
                    ReadingMode::Vertical => {
                        if self.render_vertical_mode(ctx, ui, settings) {
                            progress_changed = true;
                        }
                    }
                }
            });

        (action, progress_changed)
    }

    fn render_paged_mode(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        settings: &ReaderSettings,
    ) {
        let idx = self.current_index;
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    match &mut self.document {
                        ActiveDocument::Pdf(pdf) => {
                            match pdf.get_or_request_page(idx, settings.zoom) {
                                PageTextureState::Ready(tex) => {
                                    let base_h = (ui.available_height() - 32.0).max(460.0);
                                    let aspect = tex.aspect_ratio();
                                    let target_h = base_h * settings.zoom;
                                    let target_w = target_h * aspect;
                                    ui.image((tex.id(), Vec2::new(target_w, target_h)));
                                }
                                PageTextureState::Loading => {
                                    ui.add_space(120.0);
                                    ui.spinner();
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(format!("Carregando página {}...", idx + 1))
                                            .color(BluePagesTheme::TEXT_SECONDARY),
                                    );
                                }
                                PageTextureState::Error(err) => {
                                    ui.label(
                                        RichText::new(format!("Erro ao renderizar página: {}", err))
                                            .color(BluePagesTheme::DANGER),
                                    );
                                }
                            }
                            pdf.prefetch(idx + 1, settings.zoom);
                            if idx > 0 {
                                pdf.prefetch(idx - 1, settings.zoom);
                            }
                        }
                        ActiveDocument::Epub {
                            doc,
                            inline_textures,
                        } => {
                            if let Some(chapter) = doc.chapters.get(idx) {
                                render_epub_chapter_card(
                                    ctx,
                                    ui,
                                    &chapter.title,
                                    &chapter.blocks,
                                    inline_textures,
                                    settings,
                                );
                            }
                        }
                    }
                    ui.add_space(24.0);
                });
            });
    }

    fn render_vertical_mode(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        settings: &ReaderSettings,
    ) -> bool {
        let mut index_changed = false;
        let viewport_center_y = ui.clip_rect().center().y;
        let mut best_visible_idx: Option<(usize, f32)> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);

                    match &mut self.document {
                        ActiveDocument::Pdf(pdf) => {
                            let total = pdf.page_count;
                            let base_w = (760.0 * settings.zoom).clamp(360.0, 1800.0);
                            let est_h = base_w * 1.414;

                            for page_idx in 0..total {
                                let (slot_rect, _) = ui.allocate_exact_size(
                                    Vec2::new(base_w, est_h),
                                    Sense::hover(),
                                );

                                if self.scroll_to_index == Some(page_idx) {
                                    ui.scroll_to_rect(slot_rect, Some(Align::TOP));
                                    self.scroll_to_index = None;
                                }

                                let expanded = slot_rect.expand(est_h * 0.35);
                                if ui.is_rect_visible(expanded) {
                                    match pdf.get_or_request_page(page_idx, settings.zoom) {
                                        PageTextureState::Ready(tex) => {
                                            let tex_size = tex.size_vec2();
                                            let scale = (slot_rect.width() / tex_size.x)
                                                .min(slot_rect.height() / tex_size.y);
                                            let draw_size = tex_size * scale;
                                            let img_rect = egui::Rect::from_center_size(
                                                slot_rect.center(),
                                                draw_size,
                                            );
                                            ui.painter().image(
                                                tex.id(),
                                                img_rect,
                                                egui::Rect::from_min_max(
                                                    egui::pos2(0.0, 0.0),
                                                    egui::pos2(1.0, 1.0),
                                                ),
                                                Color32::WHITE,
                                            );
                                        }
                                        PageTextureState::Loading => {
                                            ui.painter().rect_filled(
                                                slot_rect,
                                                Rounding::same(8.0),
                                                BluePagesTheme::BG_CARD,
                                            );
                                            ui.painter().text(
                                                slot_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                format!("Carregando página {}...", page_idx + 1),
                                                FontId::proportional(15.0),
                                                BluePagesTheme::TEXT_MUTED,
                                            );
                                        }
                                        PageTextureState::Error(err) => {
                                            ui.painter().rect_filled(
                                                slot_rect,
                                                Rounding::same(8.0),
                                                BluePagesTheme::BG_CARD,
                                            );
                                            ui.painter().text(
                                                slot_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                format!("Erro pág. {}: {}", page_idx + 1, err),
                                                FontId::proportional(14.0),
                                                BluePagesTheme::DANGER,
                                            );
                                        }
                                    }

                                    let dist = (slot_rect.center().y - viewport_center_y).abs();
                                    if best_visible_idx
                                        .map(|(_, d)| dist < d)
                                        .unwrap_or(true)
                                    {
                                        best_visible_idx = Some((page_idx, dist));
                                    }
                                } else {
                                    ui.painter().rect_filled(
                                        slot_rect,
                                        Rounding::same(6.0),
                                        BluePagesTheme::BG_CARD,
                                    );
                                }

                                ui.add_space(settings.vertical_gap);
                            }
                        }
                        ActiveDocument::Epub {
                            doc,
                            inline_textures,
                        } => {
                            for (chap_idx, chapter) in doc.chapters.iter().enumerate() {
                                let resp = ui.scope(|ui| {
                                    render_epub_chapter_card(
                                        ctx,
                                        ui,
                                        &chapter.title,
                                        &chapter.blocks,
                                        inline_textures,
                                        settings,
                                    );
                                });

                                if self.scroll_to_index == Some(chap_idx) {
                                    ui.scroll_to_rect(resp.response.rect, Some(Align::TOP));
                                    self.scroll_to_index = None;
                                }

                                let dist = (resp.response.rect.center().y - viewport_center_y).abs();
                                if ui.is_rect_visible(resp.response.rect)
                                    && best_visible_idx.map(|(_, d)| dist < d).unwrap_or(true)
                                {
                                    best_visible_idx = Some((chap_idx, dist));
                                }

                                ui.add_space(settings.vertical_gap + 8.0);
                            }
                        }
                    }

                    ui.add_space(32.0);
                });
            });

        if let Some((vis_idx, _)) = best_visible_idx {
            if vis_idx != self.current_index {
                self.current_index = vis_idx;
                self.page_input_buf = (vis_idx + 1).to_string();
                index_changed = true;
            }
        }

        index_changed
    }
}

fn render_epub_chapter_card(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    title: &str,
    blocks: &[EpubBlock],
    inline_textures: &mut HashMap<String, egui::TextureHandle>,
    settings: &ReaderSettings,
) {
    let col_width = (740.0 * settings.zoom).clamp(420.0, 1280.0);
    let font_size = settings.epub_font_size * settings.zoom.clamp(0.85, 1.5);
    let family = BluePagesTheme::epub_font_family(settings.epub_font);
    let palette = BluePagesTheme::epub_palette(settings.epub_theme);

    egui::Frame::none()
        .fill(palette.page_bg)
        .rounding(Rounding::same(12.0))
        .stroke(Stroke::new(1.0_f32, palette.page_border))
        .inner_margin(Margin::symmetric(44.0, 36.0))
        .show(ui, |ui| {
            ui.set_width(col_width);
            ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                ui.add(
                    egui::Label::new(
                        RichText::new(title)
                            .font(FontId::new(font_size * 1.45, family.clone()))
                            .strong()
                            .color(palette.heading),
                    )
                    .wrap(),
                );
                ui.add_space(18.0);

                for block in blocks {
                    match block {
                        EpubBlock::Heading(text) => {
                            ui.add_space(10.0);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(text)
                                        .font(FontId::new(font_size * 1.2, family.clone()))
                                        .strong()
                                        .color(palette.heading),
                                )
                                .wrap(),
                            );
                            ui.add_space(6.0);
                        }
                        EpubBlock::Paragraph(text) => {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(text)
                                        .font(FontId::new(font_size, family.clone()))
                                        .color(palette.text),
                                )
                                .wrap(),
                            );
                            ui.add_space(12.0);
                        }
                        EpubBlock::Image {
                            id,
                            rgba,
                            width,
                            height,
                        } => {
                            let tex = inline_textures
                                .entry(id.clone())
                                .or_insert_with(|| {
                                    let img = egui::ColorImage::from_rgba_unmultiplied(
                                        [*width, *height],
                                        rgba,
                                    );
                                    ctx.load_texture(id, img, egui::TextureOptions::LINEAR)
                                })
                                .clone();

                            let max_w = col_width - 24.0;
                            let tex_size = tex.size_vec2();
                            let scale = (max_w / tex_size.x).min(1.0);
                            ui.vertical_centered(|ui| {
                                ui.image((tex.id(), tex_size * scale));
                            });
                            ui.add_space(14.0);
                        }
                    }
                }
            });
        });
}

/// Botão circular flutuante de navegação lateral (`‹` / `›`)
fn draw_floating_nav_arrow(ui: &mut egui::Ui, center: egui::Pos2, symbol: &str) -> bool {
    let radius = 21.0;
    let rect = egui::Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    let id = ui.make_persistent_id(format!("floating_arrow_{}", symbol));
    let response = ui.interact(rect, id, Sense::click());

    let bg = if response.hovered() {
        Color32::from_rgba_unmultiplied(68, 72, 82, 230)
    } else {
        Color32::from_rgba_unmultiplied(44, 46, 52, 200)
    };

    ui.painter().circle_filled(center, radius, bg);
    ui.painter().text(
        center - Vec2::new(0.0, 1.0),
        egui::Align2::CENTER_CENTER,
        symbol,
        FontId::proportional(24.0),
        BluePagesTheme::TEXT_PRIMARY,
    );

    response.clicked()
}

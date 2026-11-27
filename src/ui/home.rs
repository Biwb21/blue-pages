use crate::state::{LibraryState, RecentBook};
use crate::theme::BluePagesTheme;
use crate::ui::icons;
use egui::{Align, Color32, Layout, Margin, RichText, Rounding, Sense, Stroke, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;

pub enum HomeAction {
    None,
    OpenFileDialog,
    OpenBook(PathBuf),
    RemoveBook(PathBuf),
}

pub struct HomeView {
    pub search_query: String,
    pub active_tab: HomeTab,
    pub featured_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeTab {
    Inicio,
    Biblioteca,
}

impl Default for HomeView {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_tab: HomeTab::Inicio,
            featured_index: 0,
        }
    }
}

impl HomeView {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        library: &LibraryState,
        cover_textures: &HashMap<PathBuf, egui::TextureHandle>,
        app_icon: Option<&egui::TextureHandle>,
        error_banner: Option<&str>,
    ) -> HomeAction {
        let mut action = HomeAction::None;

        // Barra superior do Blue Pages
        egui::TopBottomPanel::top("blue_pages_home_navbar")
            .exact_height(64.0)
            .frame(
                egui::Frame::none()
                    .fill(BluePagesTheme::BG_NAVBAR)
                    .stroke(Stroke::new(1.0_f32, BluePagesTheme::BORDER))
                    .inner_margin(Margin::symmetric(24.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    // Ícone da Ararinha-Azul + Marca BLUE PAGES
                    if let Some(icon) = app_icon {
                        ui.image((icon.id(), Vec2::splat(34.0)));
                        ui.add_space(6.0);
                    }
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        ui.label(
                            RichText::new("BLUE")
                                .size(21.0)
                                .strong()
                                .color(BluePagesTheme::TEXT_PRIMARY),
                        );
                        ui.label(
                            RichText::new("PAGES")
                                .size(21.0)
                                .strong()
                                .color(BluePagesTheme::ACCENT),
                        );
                    });

                    ui.add_space(28.0);

                    // Abas de navegação
                    if nav_tab(ui, "Início", self.active_tab == HomeTab::Inicio) {
                        self.active_tab = HomeTab::Inicio;
                    }
                    ui.add_space(16.0);
                    if nav_tab(ui, "Biblioteca", self.active_tab == HomeTab::Biblioteca) {
                        self.active_tab = HomeTab::Biblioteca;
                    }

                    ui.add_space(28.0);

                    // Barra de busca arredondada (pill)
                    egui::Frame::none()
                        .fill(BluePagesTheme::BG_INPUT)
                        .rounding(Rounding::same(20.0))
                        .stroke(Stroke::new(1.0_f32, BluePagesTheme::BORDER))
                        .inner_margin(Margin::symmetric(14.0, 6.0))
                        .show(ui, |ui| {
                            ui.set_min_width(220.0);
                            ui.set_max_width(340.0);
                            ui.horizontal(|ui| {
                                icons::draw_search_icon(ui, BluePagesTheme::TEXT_MUTED);
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.search_query)
                                        .hint_text("Buscar obras, autores...")
                                        .frame(false)
                                        .desired_width(240.0),
                                );
                                if !self.search_query.is_empty()
                                    && ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new("×")
                                                    .color(BluePagesTheme::TEXT_SECONDARY),
                                            )
                                            .frame(false),
                                        )
                                        .clicked()
                                {
                                    self.search_query.clear();
                                }
                            });
                        });

                    // Botão à direita para abrir EPUB / PDF
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let open_btn = ui.add_sized(
                            [170.0, 36.0],
                            BluePagesTheme::primary_button("+ Abrir EPUB / PDF"),
                        );
                        if open_btn.clicked() {
                            action = HomeAction::OpenFileDialog;
                        }
                    });
                });
            });

        // Área principal com rolagem
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(BluePagesTheme::BG_HOME)
                    .inner_margin(Margin::symmetric(28.0, 22.0)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(err) = error_banner {
                            egui::Frame::none()
                                .fill(Color32::from_rgb(65, 24, 28))
                                .rounding(Rounding::same(10.0))
                                .stroke(Stroke::new(1.0_f32, BluePagesTheme::DANGER))
                                .inner_margin(Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(format!("Aviso: {}", err))
                                            .color(Color32::from_rgb(254, 202, 202)),
                                    );
                                });
                            ui.add_space(16.0);
                        }

                        // Filtra leituras recentes pela busca
                        let query = self.search_query.trim().to_lowercase();
                        let filtered: Vec<&RecentBook> = library
                            .recent_books
                            .iter()
                            .filter(|b| {
                                query.is_empty()
                                    || b.title.to_lowercase().contains(&query)
                                    || b.author.to_lowercase().contains(&query)
                                    || b.format.label().to_lowercase().contains(&query)
                            })
                            .collect();

                        if library.recent_books.is_empty() {
                            if let HomeAction::OpenFileDialog = draw_empty_state(ui, app_icon) {
                                action = HomeAction::OpenFileDialog;
                            }
                            return;
                        }

                        // Hero de destaque (na aba Início)
                        if self.active_tab == HomeTab::Inicio && !filtered.is_empty() {
                            let max_featured = filtered.len().min(5);
                            if self.featured_index >= max_featured {
                                self.featured_index = 0;
                            }
                            let featured = filtered[self.featured_index];

                            if let Some(path) = draw_hero_section(
                                ui,
                                featured,
                                cover_textures,
                                max_featured,
                                &mut self.featured_index,
                            ) {
                                action = HomeAction::OpenBook(path);
                            }

                            ui.add_space(28.0);
                        }

                        // Cabeçalho "Continuar lendo"
                        ui.horizontal(|ui| {
                            let heading = if self.active_tab == HomeTab::Inicio {
                                "Continuar lendo"
                            } else {
                                "Todas as leituras recentes"
                            };
                            ui.label(
                                RichText::new(heading)
                                    .size(20.0)
                                    .strong()
                                    .color(BluePagesTheme::TEXT_PRIMARY),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(format!("({})", filtered.len()))
                                    .size(14.0)
                                    .color(BluePagesTheme::TEXT_MUTED),
                            );
                        });

                        ui.add_space(14.0);

                        if filtered.is_empty() {
                            ui.label(
                                RichText::new("Nenhuma obra encontrada para essa busca.")
                                    .color(BluePagesTheme::TEXT_SECONDARY),
                            );
                        } else {
                            // Grid responsivo de cartões com largura e altura consistentes
                            let avail_w = ui.available_width();
                            let gap = 16.0_f32;
                            let card_w = 340.0_f32.min(avail_w);
                            let cols = ((avail_w + gap) / (card_w + gap)).floor().max(1.0) as usize;

                            for row_books in filtered.chunks(cols) {
                                ui.horizontal_top(|ui| {
                                    ui.spacing_mut().item_spacing.x = gap;
                                    for book in row_books {
                                        let card_action = draw_continue_reading_card(
                                            ui,
                                            book,
                                            cover_textures,
                                            card_w,
                                        );
                                        match card_action {
                                            HomeAction::None => {}
                                            other => action = other,
                                        }
                                    }
                                });
                                ui.add_space(gap);
                            }
                        }
                    });
            });

        action
    }
}

fn nav_tab(ui: &mut egui::Ui, label: &str, active: bool) -> bool {
    let text_color = if active {
        BluePagesTheme::TEXT_PRIMARY
    } else {
        BluePagesTheme::TEXT_SECONDARY
    };

    ui.vertical(|ui| {
        let btn = ui.add(
            egui::Button::new(RichText::new(label).size(14.5).color(text_color).strong())
                .frame(false),
        );
        if active {
            let rect = btn.rect;
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left(), rect.bottom() + 4.0),
                    egui::pos2(rect.right(), rect.bottom() + 4.0),
                ],
                Stroke::new(2.5_f32, BluePagesTheme::ACCENT),
            );
        }
        btn.clicked()
    })
    .inner
}

fn draw_hero_section(
    ui: &mut egui::Ui,
    book: &RecentBook,
    cover_textures: &HashMap<PathBuf, egui::TextureHandle>,
    carousel_count: usize,
    featured_index: &mut usize,
) -> Option<PathBuf> {
    let mut open_path = None;
    let total_w = ui.available_width();
    let left_w = (total_w * 0.54).clamp(320.0, 680.0);
    let right_w = (total_w - left_w - 32.0).max(260.0);
    let banner_h = 320.0;

    ui.horizontal_top(|ui| {
        // Coluna esquerda: Banner / Capa grande arredondada + indicadores de carrossel
        ui.vertical(|ui| {
            ui.set_width(left_w);

            let (rect, response) =
                ui.allocate_exact_size(Vec2::new(left_w, banner_h), Sense::click());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, Rounding::same(16.0), BluePagesTheme::BG_CARD);
            painter.rect_stroke(
                rect,
                Rounding::same(16.0),
                Stroke::new(1.0_f32, BluePagesTheme::BORDER),
            );

            if let Some(tex) = cover_textures.get(&book.path) {
                let tex_size = tex.size_vec2();
                let scale = (rect.width() / tex_size.x).min(rect.height() / tex_size.y);
                let draw_size = tex_size * scale;
                let img_rect = egui::Rect::from_center_size(rect.center(), draw_size);
                painter.image(
                    tex.id(),
                    img_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            } else {
                painter.text(
                    rect.center() - Vec2::new(0.0, 14.0),
                    egui::Align2::CENTER_CENTER,
                    book.format.label(),
                    egui::FontId::proportional(38.0),
                    BluePagesTheme::ACCENT,
                );
                painter.text(
                    rect.center() + Vec2::new(0.0, 26.0),
                    egui::Align2::CENTER_CENTER,
                    &book.title,
                    egui::FontId::proportional(18.0),
                    BluePagesTheme::TEXT_SECONDARY,
                );
            }

            if response.clicked() {
                open_path = Some(book.path.clone());
            }

            if carousel_count > 1 {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    for i in 0..carousel_count {
                        let is_active = i == *featured_index;
                        let size = if is_active {
                            Vec2::new(22.0, 5.0)
                        } else {
                            Vec2::new(6.0, 5.0)
                        };
                        let (dot_rect, dot_resp) = ui.allocate_exact_size(size, Sense::click());
                        let color = if is_active {
                            BluePagesTheme::ACCENT
                        } else {
                            BluePagesTheme::TEXT_MUTED
                        };
                        ui.painter()
                            .rect_filled(dot_rect, Rounding::same(3.0), color);
                        if dot_resp.clicked() {
                            *featured_index = i;
                        }
                    }
                });
            }
        });

        ui.add_space(32.0);

        // Coluna direita com largura explícita para quebra de linha limpa
        ui.vertical(|ui| {
            ui.set_width(right_w);
            ui.add_space(12.0);
            ui.label(
                RichText::new(format!("Leitura recente · {}", book.format.label()))
                    .size(13.0)
                    .color(BluePagesTheme::ACCENT),
            );

            ui.add_space(6.0);

            ui.add(
                egui::Label::new(
                    RichText::new(book.title.to_uppercase())
                        .size(32.0)
                        .strong()
                        .color(BluePagesTheme::TEXT_PRIMARY),
                )
                .wrap(),
            );

            ui.add_space(8.0);

            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("por ")
                        .size(14.0)
                        .color(BluePagesTheme::TEXT_SECONDARY),
                );
                ui.label(
                    RichText::new(&book.author)
                        .size(14.0)
                        .strong()
                        .color(BluePagesTheme::TEXT_PRIMARY),
                );
                ui.label(RichText::new("  ·  ").color(BluePagesTheme::TEXT_MUTED));
                
                ui.label(
                    RichText::new(book.progress_label())
                        .size(14.0)
                        .color(BluePagesTheme::TEXT_SECONDARY),
                );
            });

            ui.add_space(14.0);

            let desc = if book.description.chars().count() > 220 {
                format!(
                    "{}...",
                    book.description.chars().take(220).collect::<String>()
                )
            } else {
                book.description.clone()
            };

            ui.add(
                egui::Label::new(
                    RichText::new(desc)
                        .size(14.5)
                        .color(BluePagesTheme::TEXT_SECONDARY),
                )
                .wrap(),
            );

            ui.add_space(22.0);

            let cta_label = if book.current_index == 0 {
                "Ler agora"
            } else {
                "Continuar leitura"
            };

            let cta = icons::primary_play_button(ui, Vec2::new(210.0, 44.0), cta_label);
            if cta.clicked() {
                open_path = Some(book.path.clone());
            }
        });
    });

    open_path
}

fn draw_continue_reading_card(
    ui: &mut egui::Ui,
    book: &RecentBook,
    cover_textures: &HashMap<PathBuf, egui::TextureHandle>,
    card_width: f32,
) -> HomeAction {
    let mut action = HomeAction::None;
    let inner_w = (card_width - 32.0).max(200.0);
    let thumb_w = 68.0_f32;
    let thumb_h = 98.0_f32;
    let text_col_w = (inner_w - thumb_w - 14.0).max(120.0);

    // IMPORTANTE: ui.vertical(...) explícito dentro do Frame evita herdar o layout horizontal da linha!
    BluePagesTheme::card_frame().show(ui, |ui| {
        ui.vertical(|ui| {
            ui.set_width(inner_w);

            // 1. Cabeçalho do card: Badge EPUB/PDF à esquerda + botão × à direita
            ui.horizontal(|ui| {
                ui.set_width(inner_w);
                egui::Frame::none()
                    .fill(BluePagesTheme::ACCENT_SOFT)
                    .rounding(Rounding::same(6.0))
                    .inner_margin(Margin::symmetric(8.0, 2.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(book.format.label())
                                .size(11.0)
                                .strong()
                                .color(BluePagesTheme::ACCENT),
                        );
                    });

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let remove_btn = ui.add(
                        egui::Button::new(
                            RichText::new("×")
                                .size(16.0)
                                .color(BluePagesTheme::TEXT_MUTED),
                        )
                        .frame(false),
                    );
                    if remove_btn
                        .on_hover_text("Remover das leituras recentes")
                        .clicked()
                    {
                        action = HomeAction::RemoveBook(book.path.clone());
                    }
                });
            });

            ui.add_space(10.0);

            // 2. Corpo do card: Capa à esquerda + Título/Progresso/Barra à direita
            ui.horizontal_top(|ui| {
                ui.set_width(inner_w);

                let (thumb_rect, thumb_resp) =
                    ui.allocate_exact_size(Vec2::new(thumb_w, thumb_h), Sense::click());
                let painter = ui.painter_at(thumb_rect);
                painter.rect_filled(thumb_rect, Rounding::same(8.0), BluePagesTheme::BG_INPUT);

                if let Some(tex) = cover_textures.get(&book.path) {
                    let tex_size = tex.size_vec2();
                    let scale = (thumb_rect.width() / tex_size.x).min(thumb_rect.height() / tex_size.y);
                    let draw_size = tex_size * scale;
                    let img_rect = egui::Rect::from_center_size(thumb_rect.center(), draw_size);
                    painter.image(
                        tex.id(),
                        img_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                } else {
                    painter.text(
                        thumb_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        book.format.label(),
                        egui::FontId::proportional(14.0),
                        BluePagesTheme::ACCENT,
                    );
                }

                if thumb_resp.clicked() {
                    action = HomeAction::OpenBook(book.path.clone());
                }

                ui.add_space(12.0);

                ui.vertical(|ui| {
                    ui.set_width(text_col_w);
                    ui.set_min_height(thumb_h);

                    let clean_title = if book.title.chars().count() > 42 {
                        format!("{}…", book.title.chars().take(42).collect::<String>())
                    } else {
                        book.title.clone()
                    };

                    ui.add(
                        egui::Label::new(
                            RichText::new(clean_title)
                                .size(15.0)
                                .strong()
                                .color(BluePagesTheme::TEXT_PRIMARY),
                        )
                        .wrap(),
                    );

                    ui.add_space(4.0);

                    ui.label(
                        RichText::new(book.progress_label())
                            .size(12.5)
                            .color(BluePagesTheme::TEXT_SECONDARY),
                    );

                    ui.add_space(12.0);

                    // Barra de progresso horizontal
                    let (bar_rect, _) =
                        ui.allocate_exact_size(Vec2::new(text_col_w, 5.0), Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(bar_rect, Rounding::same(3.0), BluePagesTheme::PROGRESS_BG);
                    let fill_rect = egui::Rect::from_min_size(
                        bar_rect.min,
                        Vec2::new(
                            (bar_rect.width() * book.progress_fraction()).max(6.0),
                            bar_rect.height(),
                        ),
                    );
                    p.rect_filled(fill_rect, Rounding::same(3.0), BluePagesTheme::PROGRESS_FILL);
                });
            });

            ui.add_space(14.0);

            // 3. Rodapé do card: Botão largura total "▶ Continuar"
            let btn = icons::primary_play_button(ui, Vec2::new(inner_w, 38.0), "Continuar");
            if btn.clicked() {
                action = HomeAction::OpenBook(book.path.clone());
            }
        });
    });

    action
}

fn draw_empty_state(ui: &mut egui::Ui, app_icon: Option<&egui::TextureHandle>) -> HomeAction {
    let mut action = HomeAction::None;

    ui.add_space(40.0);
    ui.vertical_centered(|ui| {
        BluePagesTheme::card_frame().show(ui, |ui| {
            ui.set_max_width(540.0);
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                if let Some(icon) = app_icon {
                    ui.image((icon.id(), Vec2::splat(72.0)));
                    ui.add_space(12.0);
                }
                ui.label(
                    RichText::new("BEM-VINDO AO BLUE PAGES")
                        .size(25.0)
                        .strong()
                        .color(BluePagesTheme::TEXT_PRIMARY),
                );
                ui.add_space(10.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(
                            "Arraste um arquivo .epub ou .pdf para esta janela ou clique no botão abaixo para começar a ler. Suas leituras recentes aparecerão automaticamente aqui.",
                        )
                        .size(14.5)
                        .color(BluePagesTheme::TEXT_SECONDARY),
                    )
                    .wrap(),
                );
                ui.add_space(22.0);
                if icons::primary_play_button(ui, Vec2::new(230.0, 44.0), "Abrir primeiro livro")
                    .clicked()
                {
                    action = HomeAction::OpenFileDialog;
                }
                ui.add_space(12.0);
            });
        });
    });

    action
}

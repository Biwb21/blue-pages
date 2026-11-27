use crate::state::{EpubColorScheme, EpubFontChoice};
use egui::{
    Color32, FontData, FontDefinitions, FontFamily, FontId, Margin, Rounding, Stroke, TextStyle,
    Visuals,
};

const FONT_CALADEA: &[u8] = include_bytes!("../assets/fonts/Caladea-Regular.ttf");
const FONT_CARLITO: &[u8] = include_bytes!("../assets/fonts/Carlito-Regular.ttf");
const FONT_LIBERATION_SERIF: &[u8] = include_bytes!("../assets/fonts/LiberationSerif-Regular.ttf");
const FONT_DEJAVU_SERIF: &[u8] = include_bytes!("../assets/fonts/DejaVuSerif.ttf");
const FONT_LIBERATION_MONO: &[u8] = include_bytes!("../assets/fonts/LiberationMono-Regular.ttf");

pub struct EpubPalette {
    pub reader_bg: Color32,
    pub page_bg: Color32,
    pub page_border: Color32,
    pub text: Color32,
    pub heading: Color32,
    pub swatch_preview: Color32,
}

/// Paleta escura moderna do Blue Pages com acento Ararinha-Azul
pub struct BluePagesTheme;

impl BluePagesTheme {
    // Superfícies principais (Tela inicial)
    pub const BG_HOME: Color32 = Color32::from_rgb(16, 20, 28);       // #10141C
    pub const BG_NAVBAR: Color32 = Color32::from_rgb(20, 23, 31);     // #14171F
    pub const BG_CARD: Color32 = Color32::from_rgb(26, 31, 42);       // #1A1F2A
    pub const BG_CARD_HOVER: Color32 = Color32::from_rgb(33, 40, 54); // #212836
    pub const BG_INPUT: Color32 = Color32::from_rgb(24, 29, 40);      // #181D28

    // Superfícies do leitor (Modo leitura)
    pub const BG_READER: Color32 = Color32::from_rgb(22, 24, 29);     // #16181D
    pub const BG_READER_BAR: Color32 = Color32::from_rgb(17, 19, 24); // #111318
    pub const BG_PILL: Color32 = Color32::from_rgb(32, 37, 48);       // #202530

    // Acentos Blue Pages (Ararinha-Azul)
    pub const ACCENT: Color32 = Color32::from_rgb(56, 189, 248);      // #38BDF8
    pub const ACCENT_SOFT: Color32 = Color32::from_rgb(22, 48, 68);
    pub const ACCENT_TEXT: Color32 = Color32::from_rgb(11, 18, 32);

    // Tipografia e bordas
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(244, 247, 252);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 167, 184);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 118, 136);
    pub const BORDER: Color32 = Color32::from_rgb(41, 48, 63);
    pub const PROGRESS_BG: Color32 = Color32::from_rgb(50, 59, 75);
    pub const PROGRESS_FILL: Color32 = Color32::from_rgb(56, 189, 248);
    pub const DANGER: Color32 = Color32::from_rgb(239, 68, 68);

    /// Registra as 5 fontes TrueType embutidas para personalização de leitura de EPUBs
    pub fn install_fonts(ctx: &egui::Context) {
        let mut fonts = FontDefinitions::default();

        let bundled: &[(&str, &[u8])] = &[
            ("caladea", FONT_CALADEA),
            ("carlito", FONT_CARLITO),
            ("liberation_serif", FONT_LIBERATION_SERIF),
            ("dejavu_serif", FONT_DEJAVU_SERIF),
            ("liberation_mono", FONT_LIBERATION_MONO),
        ];

        for (name, bytes) in bundled {
            fonts
                .font_data
                .insert((*name).to_string(), FontData::from_static(bytes));
            fonts.families.insert(
                FontFamily::Name((*name).into()),
                vec![(*name).to_string(), "Ubuntu-Light".to_string()],
            );
        }

        ctx.set_fonts(fonts);
    }

    pub fn epub_font_family(choice: EpubFontChoice) -> FontFamily {
        match choice {
            EpubFontChoice::DefaultSans => FontFamily::Proportional,
            EpubFontChoice::Caladea => FontFamily::Name("caladea".into()),
            EpubFontChoice::Carlito => FontFamily::Name("carlito".into()),
            EpubFontChoice::LiberationSerif => FontFamily::Name("liberation_serif".into()),
            EpubFontChoice::DejaVuSerif => FontFamily::Name("dejavu_serif".into()),
            EpubFontChoice::LiberationMono => FontFamily::Name("liberation_mono".into()),
        }
    }

    pub fn epub_palette(scheme: EpubColorScheme) -> EpubPalette {
        match scheme {
            EpubColorScheme::DarkBlue => EpubPalette {
                reader_bg: Color32::from_rgb(20, 23, 30),
                page_bg: Color32::from_rgb(26, 30, 39),
                page_border: Color32::from_rgb(43, 50, 66),
                text: Color32::from_rgb(228, 233, 242),
                heading: Color32::from_rgb(56, 189, 248),
                swatch_preview: Color32::from_rgb(26, 30, 39),
            },
            EpubColorScheme::Amoled => EpubPalette {
                reader_bg: Color32::from_rgb(4, 5, 7),
                page_bg: Color32::from_rgb(10, 11, 15),
                page_border: Color32::from_rgb(30, 34, 44),
                text: Color32::from_rgb(216, 222, 233),
                heading: Color32::from_rgb(96, 206, 252),
                swatch_preview: Color32::from_rgb(10, 11, 15),
            },
            EpubColorScheme::Sepia => EpubPalette {
                reader_bg: Color32::from_rgb(227, 214, 187),
                page_bg: Color32::from_rgb(245, 236, 215),
                page_border: Color32::from_rgb(209, 193, 160),
                text: Color32::from_rgb(59, 46, 30),
                heading: Color32::from_rgb(138, 68, 22),
                swatch_preview: Color32::from_rgb(245, 236, 215),
            },
            EpubColorScheme::Light => EpubPalette {
                reader_bg: Color32::from_rgb(228, 233, 240),
                page_bg: Color32::from_rgb(252, 253, 255),
                page_border: Color32::from_rgb(203, 213, 225),
                text: Color32::from_rgb(26, 32, 44),
                heading: Color32::from_rgb(2, 132, 199),
                swatch_preview: Color32::from_rgb(252, 253, 255),
            },
            EpubColorScheme::SlateNight => EpubPalette {
                reader_bg: Color32::from_rgb(28, 34, 45),
                page_bg: Color32::from_rgb(36, 44, 58),
                page_border: Color32::from_rgb(56, 68, 88),
                text: Color32::from_rgb(218, 228, 240),
                heading: Color32::from_rgb(125, 211, 252),
                swatch_preview: Color32::from_rgb(36, 44, 58),
            },
        }
    }

    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        let mut visuals = Visuals::dark();
        visuals.panel_fill = Self::BG_HOME;
        visuals.window_fill = Self::BG_CARD;
        visuals.extreme_bg_color = Self::BG_INPUT;
        visuals.faint_bg_color = Self::BG_CARD;
        visuals.selection.bg_fill = Self::ACCENT;
        visuals.selection.stroke = Stroke::new(1.0_f32, Self::ACCENT_TEXT);

        visuals.widgets.noninteractive.bg_fill = Self::BG_CARD;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Self::BORDER);

        visuals.widgets.inactive.bg_fill = Self::BG_PILL;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.inactive.rounding = Rounding::same(8.0);

        visuals.widgets.hovered.bg_fill = Self::BG_CARD_HOVER;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, Self::TEXT_PRIMARY);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Self::ACCENT);
        visuals.widgets.hovered.rounding = Rounding::same(8.0);

        visuals.widgets.active.bg_fill = Self::ACCENT;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Self::ACCENT_TEXT);
        visuals.widgets.active.rounding = Rounding::same(8.0);

        style.visuals = visuals;
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = Margin::same(16.0);

        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(26.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(15.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(14.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.0, FontFamily::Proportional),
        );

        ctx.set_style(style);
    }

    pub fn primary_button<'a>(text: &'a str) -> egui::Button<'a> {
        egui::Button::new(
            egui::RichText::new(text)
                .color(Self::ACCENT_TEXT)
                .strong()
                .size(14.5),
        )
        .fill(Self::ACCENT)
        .stroke(Stroke::NONE)
        .rounding(Rounding::same(22.0))
    }

    pub fn card_frame() -> egui::Frame {
        egui::Frame::none()
            .fill(Self::BG_CARD)
            .rounding(Rounding::same(14.0))
            .stroke(Stroke::new(1.0_f32, Self::BORDER))
            .inner_margin(Margin::same(16.0))
    }
}

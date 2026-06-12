use egui::{Pos2, Vec2};

use crate::app::page::painter::PdfOps;

mod page;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TemplateApp {
    label: String,

    #[serde(skip)]
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            label: "Hello World!".to_owned(),
            value: 2.7,
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default()
    }
}

impl eframe::App for TemplateApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            if ui.button("export").clicked() {
                use printpdf::*;

                let content_rect = egui::Rect::from_min_size(Pos2::ZERO, Vec2::new(1872., 1404.));

                let mut doc = PdfDocument::new("Calendar");
                let mut index_contents = vec![Op::Marker {
                    id: "index".to_owned(),
                }];
                page::Page::Index.paint(
                    &mut page::painter::Painter::Pdf {
                        ops: PdfOps::new(&mut index_contents, content_rect.height()),
                    },
                    content_rect,
                );
                let index_page = PdfPage::new(
                    Pt(content_rect.width()).into(),
                    Pt(content_rect.height()).into(),
                    index_contents,
                );

                let mut warnings = Vec::new();
                let pdf_bytes: Vec<u8> = doc
                    .with_pages(vec![index_page])
                    .save(&PdfSaveOptions::default(), &mut warnings);

                eprintln!("warnings: {warnings:#?}");

                std::fs::write("/home/dark/a.pdf", pdf_bytes).unwrap();
            }
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            page::Page::Index.paint(
                &mut page::painter::Painter::Egui {
                    ui,
                    painter: ui.painter().with_clip_rect(ui.clip_rect()),
                },
                ui.available_rect_before_wrap(),
            );
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}

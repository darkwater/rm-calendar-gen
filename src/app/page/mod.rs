use egui::vec2;
use printpdf::BuiltinFont;

use crate::app::page::painter::{Color, Painter, Width};

pub mod painter;

pub enum Page {
    Index,
}

impl Page {
    pub fn paint(&self, painter: &mut Painter<'_>, rect: egui::Rect) {
        match self {
            Self::Index => {
                let (header, months) = rect.split_top_bottom_at_fraction(0.15);

                let a = header.shrink2(vec2(0., header.height() * 0.2));
                painter.text("2026", a, Color::Strong, BuiltinFont::TimesBold);
                painter.debug_rect(header, egui::Color32::BLUE, "");
                painter.debug_rect(a, egui::Color32::RED, "");

                painter.line(
                    vec![header.left_bottom(), header.right_bottom()],
                    Width::Thick,
                    Color::Strong,
                );

                for y in [(1. / 3.), (2. / 3.)] {
                    let (_, row) = months.split_top_bottom_at_fraction(y);
                    painter.line(
                        vec![row.left_top(), row.right_top()],
                        Width::Normal,
                        Color::Strong,
                    );
                }

                for x in [(1. / 4.), (2. / 4.), (3. / 4.)] {
                    let (col, _) = months.split_left_right_at_fraction(x);
                    painter.line(
                        vec![col.right_top(), col.right_bottom()],
                        Width::Normal,
                        Color::Strong,
                    );
                }

                let cell_size = vec2(months.width() / 4., months.height() / 3.);
                for month in 1..=12 {
                    let col = (month - 1) % 4;
                    let row = (month - 1) / 4;

                    let cell = egui::Rect::from_min_size(
                        months.left_top()
                            + vec2(col as f32 * cell_size.x, row as f32 * cell_size.y),
                        cell_size,
                    );

                    painter.text(month.to_string(), cell, Color::Weak, BuiltinFont::TimesBold);
                    painter.debug_rect(cell, egui::Color32::GREEN, month.to_string());
                }
            }
        }
    }
}

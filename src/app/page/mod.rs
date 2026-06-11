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

                painter.line(
                    vec![header.left_bottom(), header.right_bottom()],
                    Width::Normal,
                    Color::Primary,
                );

                for y in [(1. / 3.), (2. / 3.)] {
                    let (_, row) = months.split_top_bottom_at_fraction(y);
                    painter.line(
                        vec![row.left_top(), row.right_top()],
                        Width::Normal,
                        Color::Primary,
                    );
                }

                for x in [(1. / 4.), (2. / 4.), (3. / 4.)] {
                    let (col, _) = months.split_left_right_at_fraction(x);
                    painter.line(
                        vec![col.right_top(), col.right_bottom()],
                        Width::Normal,
                        Color::Primary,
                    );
                }
            }
        }
    }
}

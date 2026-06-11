use egui::{Color32, Pos2, emath::GuiRounding as _};
use printpdf::{LinePoint, Op, Point, Pt};

pub enum Painter<'a> {
    Egui {
        ui: &'a egui::Ui,
        painter: egui::Painter,
    },
    Pdf {
        ops: &'a mut Vec<Op>,
        page_height: f32,
    },
}

#[derive(Clone, Copy)]
pub enum Color {
    Primary,
    Secondary,
}

impl Color {
    pub fn to_egui(self, ui: &egui::Ui) -> Color32 {
        match self {
            Self::Primary => ui.visuals().strong_text_color(),
            Self::Secondary => ui.visuals().weak_text_color(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum Width {
    Normal,
}

impl Width {
    pub fn to_egui(self, ui: &egui::Ui) -> f32 {
        match self {
            Self::Normal => 1.,
        }
    }
}

impl Painter<'_> {
    pub fn line(&mut self, mut points: Vec<Pos2>, width: Width, color: Color) {
        match self {
            Painter::Egui { ui, painter } => {
                for point in &mut points {
                    *point = point.round_to_pixel_center(ui.pixels_per_point());
                }
                painter.line(points, (width.to_egui(ui), color.to_egui(ui)));
            }
            Painter::Pdf { ops, page_height } => {
                ops.push(Op::DrawLine {
                    line: printpdf::Line {
                        points: points
                            .into_iter()
                            .map(|p| LinePoint {
                                p: Point::new(Pt(p.x).into(), Pt(*page_height - p.y).into()),
                                bezier: false,
                            })
                            .collect(),
                        is_closed: false,
                    },
                });
            }
        }
    }
}

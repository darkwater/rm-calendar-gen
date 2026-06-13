use azul_layout::{
    font_traits::{BidiDirection, Script, StyleProperties},
    hyphenation::Language,
};
use egui::{Color32, FontId, Pos2, Rect, emath::GuiRounding as _, text::LayoutJob, vec2};
use printpdf::{
    Actions, BorderArray, BuiltinFont, ColorArray, Destination, HighlightingMode, LinePoint,
    LinkAnnotation, Mm, Op, PdfFontHandle, Point, Pt, TextItem, text_shaping::ParsedFontTrait as _,
};

pub enum Painter<'a> {
    Egui {
        ui: &'a egui::Ui,
        painter: egui::Painter,
    },
    Pdf {
        ops: PdfOps<'a>,
    },
}

pub struct PdfOps<'a> {
    ops: &'a mut Vec<Op>,
    page_height: f32,
    color: Option<Color>,
    width: Option<Width>,
}

impl<'a> PdfOps<'a> {
    pub fn new(ops: &'a mut Vec<Op>, page_height: f32) -> Self {
        Self {
            ops,
            page_height,
            color: None,
            width: None,
        }
    }

    pub fn y(&self, y: f32) -> f32 {
        self.page_height - y
    }

    pub fn push(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn set_color(&mut self, color: Color) {
        if self.color == Some(color) {
            return;
        }

        self.push(Op::SetOutlineColor {
            col: color.to_pdf(),
        });
        self.push(Op::SetFillColor {
            col: color.to_pdf(),
        });
        self.color = Some(color);
    }

    pub fn set_width(&mut self, width: Width) {
        if self.width == Some(width) {
            return;
        }

        self.push(Op::SetOutlineThickness {
            pt: width.to_pdf().into(),
        });
        self.width = Some(width);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Strong,
    Normal,
    Weak,
}

impl Color {
    pub fn to_egui(self, ui: &egui::Ui) -> Color32 {
        match self {
            Self::Strong => ui.visuals().strong_text_color(),
            Self::Normal => ui.visuals().text_color(),
            Self::Weak => ui.visuals().weak_text_color(),
        }
    }

    pub fn to_pdf(self) -> printpdf::Color {
        match self {
            Self::Strong => printpdf::Color::Greyscale(printpdf::Greyscale::new(0., None)),
            Self::Normal => printpdf::Color::Greyscale(printpdf::Greyscale::new(0.4, None)),
            Self::Weak => printpdf::Color::Greyscale(printpdf::Greyscale::new(0.8, None)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Width {
    Thin,
    Normal,
    Thick,
}

impl Width {
    pub fn to_egui(self, ui: &egui::Ui) -> f32 {
        match self {
            Self::Thin => 0.5,
            Self::Normal => 1.,
            Self::Thick => 2.,
        }
    }

    pub fn to_pdf(self) -> Mm {
        match self {
            Self::Thin => Pt(1.),
            Self::Normal => Pt(2.),
            Self::Thick => Pt(4.),
        }
        .into()
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
            Painter::Pdf { ops } => {
                ops.set_color(color);
                ops.set_width(width);
                ops.push(Op::DrawLine {
                    line: printpdf::Line {
                        points: points
                            .into_iter()
                            .map(|p| LinePoint {
                                p: Point::new(Pt(p.x).into(), Pt(ops.y(p.y)).into()),
                                bezier: false,
                            })
                            .collect(),
                        is_closed: false,
                    },
                });
            }
        }
    }

    pub fn text(&mut self, text: impl ToString, rect: egui::Rect, color: Color, font: BuiltinFont) {
        match self {
            Painter::Egui { ui, painter } => {
                let height = rect.height() * 1.25;

                let job = LayoutJob::simple_singleline(
                    text.to_string(),
                    FontId::proportional(height),
                    color.to_egui(ui),
                );
                let galley = painter.layout_job(job);

                let width = galley.size().x;

                let target = Rect::from_center_size(rect.center(), vec2(width, rect.height()));

                painter.galley(
                    target.left_bottom() - vec2(0., height),
                    galley,
                    Color32::PLACEHOLDER,
                );
            }
            Painter::Pdf { ops } => {
                let height = rect.height() * 1.47;

                let text = text.to_string();
                let shape = font
                    .get_parsed_font()
                    .expect("parsing builtin font failed")
                    .shape_text(
                        &text,
                        Script::Latin,
                        Language::EnglishUS,
                        BidiDirection::Ltr,
                        &StyleProperties {
                            font_size_px: height,
                            ..Default::default()
                        },
                    )
                    .expect("shaping failed");

                let width = shape.iter().map(|g| g.advance).sum::<f32>();

                let target = Rect::from_center_size(rect.center(), vec2(width, rect.height()));

                ops.set_color(color);
                ops.push(Op::StartTextSection);
                ops.push(Op::SetFont {
                    font: PdfFontHandle::Builtin(font),
                    size: Pt(height),
                });
                ops.push(Op::SetTextCursor {
                    pos: Point::new(Pt(target.left()).into(), Pt(ops.y(target.bottom())).into()),
                });
                ops.push(Op::ShowText {
                    items: vec![TextItem::Text(text)],
                });
                ops.push(Op::EndTextSection);
            }
        }
    }

    pub fn debug_rect(&mut self, rect: egui::Rect, color: Color32, label: impl ToString) {
        if let Painter::Egui { painter, .. } = self {
            painter.debug_rect(rect, color, label);
        }
    }

    pub fn pdf_link(&mut self, page: usize, cell: Rect) {
        if let Painter::Pdf { ops } = self {
            ops.push(Op::LinkAnnotation {
                link: {
                    LinkAnnotation {
                        rect: printpdf::Rect {
                            x: Pt(cell.left()),
                            y: Pt(ops.y(cell.bottom())),
                            width: Pt(cell.width()),
                            height: Pt(cell.height()),
                            mode: None,
                            winding_order: None,
                        },
                        border: BorderArray::Solid([0., 0., 0.]),
                        color: ColorArray::Transparent,
                        actions: Actions::go_to(Destination::Xyz {
                            page,
                            left: None,
                            top: None,
                            zoom: None,
                        }),
                        highlighting: HighlightingMode::None,
                    }
                },
            });
        }
    }
}

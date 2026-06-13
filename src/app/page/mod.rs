pub mod painter;

use chrono::Datelike as _;
use egui::{Rect, vec2};
use printpdf::BuiltinFont;

use crate::app::page::painter::{Color, Painter, Width};

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

pub enum Page {
    Index,
    Month(i32, i32),
}

impl Page {
    pub fn paint(&self, painter: &mut Painter<'_>, rect: egui::Rect) {
        let year = 2026;
        let first_weekday = chrono::Weekday::Mon;

        match self {
            Self::Index => {
                let (header, months) = rect.split_top_bottom_at_fraction(0.15);

                let a = padding_vert(header, 0.2);
                painter.text(year.to_string(), a, Color::Strong, BuiltinFont::TimesBold);
                painter.debug_rect(header, egui::Color32::BLUE, "");
                painter.debug_rect(a, egui::Color32::RED, "");

                painter.line(
                    vec![header.left_bottom(), header.right_bottom()],
                    Width::Thick,
                    Color::Strong,
                );

                for y in [1., 2.] {
                    let y = y / 3.;
                    let (_, row) = months.split_top_bottom_at_fraction(y);
                    painter.line(
                        vec![row.left_top(), row.right_top()],
                        Width::Normal,
                        Color::Strong,
                    );
                }

                for x in [1., 2., 3.] {
                    let x = x / 4.;
                    let (col, _) = months.split_left_right_at_fraction(x);
                    painter.line(
                        vec![col.right_top(), col.right_bottom()],
                        Width::Normal,
                        Color::Strong,
                    );
                }

                let cell_size = vec2(months.width() / 4., months.height() / 3.);
                for (month_idx, month_name) in MONTH_NAMES.iter().enumerate() {
                    let col = month_idx % 4;
                    let row = month_idx / 4;

                    let cell = egui::Rect::from_min_size(
                        months.left_top()
                            + vec2(col as f32 * cell_size.x, row as f32 * cell_size.y),
                        cell_size,
                    );

                    let rows = split_hor::<7>(cell);
                    let [header, rows @ ..] = rows;

                    painter.text(
                        month_name,
                        padding_vert(header, 0.2),
                        Color::Strong,
                        BuiltinFont::TimesBold,
                    );

                    let first_day =
                        chrono::NaiveDate::from_ymd_opt(year, (month_idx + 1) as u32, 1)
                            .expect("invalid date");

                    let mut first_day_on_grid = first_day;
                    while first_day_on_grid.weekday() != first_weekday {
                        first_day_on_grid -= chrono::Duration::days(1);
                    }

                    let mut current_day = first_day_on_grid;
                    for row in rows {
                        let cols = split_vert::<7>(padding_hor(row, 0.02));
                        for col in cols {
                            if current_day.month() == (month_idx + 1) as u32 {
                                painter.text(
                                    current_day.day().to_string(),
                                    padding_vert(col, 0.25),
                                    Color::Normal,
                                    BuiltinFont::TimesRoman,
                                );
                            }

                            current_day += chrono::Duration::days(1);
                        }
                    }

                    painter.pdf_link(month_idx + 2, cell);
                }
            }
            Self::Month(year, month) => {
                let rows = split_hor::<7>(rect);
                let [header, rows @ ..] = rows;
                let [first_row, .., last_row] = rows;

                let grid_area = Rect::from_min_max(first_row.left_top(), last_row.right_bottom());

                painter.line(
                    vec![header.left_bottom(), header.right_bottom()],
                    Width::Thick,
                    Color::Strong,
                );

                for y in 1..6 {
                    let y = y as f32 / 6.;
                    let (_, row) = grid_area.split_top_bottom_at_fraction(y);
                    painter.line(
                        vec![row.left_top(), row.right_top()],
                        Width::Normal,
                        Color::Weak,
                    );
                }

                for x in 1..7 {
                    let x = x as f32 / 7.;
                    let (col, _) = grid_area.split_left_right_at_fraction(x);
                    painter.line(
                        vec![col.right_top(), col.right_bottom()],
                        Width::Normal,
                        Color::Weak,
                    );
                }

                let month_name = MONTH_NAMES[*month as usize];

                painter.text(
                    month_name,
                    padding_vert(header, 0.2),
                    Color::Strong,
                    BuiltinFont::TimesBold,
                );

                let first_day = chrono::NaiveDate::from_ymd_opt(*year, (month + 1) as u32, 1)
                    .expect("invalid date");

                let mut first_day_on_grid = first_day;
                while first_day_on_grid.weekday() != first_weekday {
                    first_day_on_grid -= chrono::Duration::days(1);
                }

                let mut current_day = first_day_on_grid;
                for row in rows {
                    let cols = split_vert::<7>(row);
                    for col in cols {
                        if current_day.month() == (month + 1) as u32 {
                            let rect = Rect::from_min_size(col.left_top(), col.size() * 0.25);

                            painter.text(
                                current_day.day().to_string(),
                                padding(rect, 0.2, 0.2),
                                Color::Normal,
                                BuiltinFont::TimesRoman,
                            );
                        }

                        current_day += chrono::Duration::days(1);
                    }
                }
            }
        }
    }
}

fn padding_hor(rect: egui::Rect, padding: f32) -> egui::Rect {
    rect.shrink2(vec2(rect.width() * padding, 0.))
}

fn padding_vert(rect: egui::Rect, padding: f32) -> egui::Rect {
    rect.shrink2(vec2(0., rect.height() * padding))
}

fn padding(rect: egui::Rect, vert: f32, hor: f32) -> egui::Rect {
    padding_vert(padding_hor(rect, hor), vert)
}

fn split_hor<const N: usize>(rect: egui::Rect) -> [egui::Rect; N] {
    let mut out = [rect; N];
    for (i, out) in out.iter_mut().enumerate() {
        out.min.y = rect.top() + (i as f32 / N as f32) * rect.height();
        out.max.y = rect.top() + ((i + 1) as f32 / N as f32) * rect.height();
    }
    out
}

fn split_vert<const N: usize>(rect: egui::Rect) -> [egui::Rect; N] {
    let mut out = [rect; N];
    for (i, out) in out.iter_mut().enumerate() {
        out.min.x = rect.left() + (i as f32 / N as f32) * rect.width();
        out.max.x = rect.left() + ((i + 1) as f32 / N as f32) * rect.width();
    }
    out
}

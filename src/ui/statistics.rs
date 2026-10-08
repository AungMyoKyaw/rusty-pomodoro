//! Statistics panel: totals for the day or week, a 7-day bar chart, and CSV export.

use egui::{Align2, FontId, Pos2, Rect, Sense, Stroke, Vec2};

use crate::app::{App, Panel};
use crate::stats::{self, Totals};

pub fn draw(app: &mut App, ctx: &egui::Context) {
    if app.panel != Panel::Statistics {
        return;
    }
    let mut open = true;
    egui::Window::new("Statistics")
        .open(&mut open)
        .resizable(true)
        .default_width(420.0)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let day_selected = app.stats_scope_day;
                if ui
                    .selectable_label(day_selected, "Today")
                    .on_hover_text("Totals for today")
                    .clicked()
                {
                    app.stats_scope_day = true;
                }
                if ui
                    .selectable_label(!day_selected, "Week")
                    .on_hover_text("Totals for the last 7 days")
                    .clicked()
                {
                    app.stats_scope_day = false;
                }
            });
            ui.separator();

            let today = stats::today_index();
            let totals = if app.stats_scope_day {
                app.stats
                    .totals_for_day(today, app.settings.show_completed_only)
            } else {
                app.stats
                    .totals_for_week(today, app.settings.show_completed_only)
            };
            match totals {
                Ok(totals) => summary(ui, app, totals),
                Err(error) => {
                    ui.label(format!("Statistics read failed: {error}"));
                }
            };

            ui.separator();
            ui.label("Last 7 days");
            chart(ui, app, today);

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Export CSV").clicked() {
                    let target = app.stats.path().with_file_name("tomito-stats.csv");
                    match app.stats.export_csv(&target) {
                        Ok(()) => app.status = format!("Exported to {}", target.display()),
                        Err(error) => app.status = format!("Export failed: {error}"),
                    }
                }
                if ui.button("Reset statistics").clicked() {
                    app.confirm_reset = true;
                }
            });
            if app.confirm_reset {
                ui.label("Delete all activity history? This cannot be undone.");
                ui.horizontal(|ui| {
                    if ui.button("Delete history").clicked() {
                        app.status = match app.stats.reset() {
                            Ok(()) => "Statistics cleared".into(),
                            Err(error) => format!("Reset failed: {error}"),
                        };
                        app.confirm_reset = false;
                    }
                    if ui.button("Cancel").clicked() {
                        app.confirm_reset = false;
                    }
                });
            }
            if !app.settings.show_completed_only {
                ui.colored_label(
                    app.palette.dim_text,
                    "Skipped activities are included in totals.",
                );
            }
        });
    if !open {
        app.panel = Panel::None;
    }
}

fn summary(ui: &mut egui::Ui, app: &App, totals: Totals) {
    ui.columns(2, |columns| {
        columns[0].label(format!("Sessions: {}", totals.sessions));
        columns[1].label(format!("Breaks: {}", totals.breaks));
        columns[0].label(format!("Long breaks: {}", totals.long_breaks));
        columns[1].label(format!("Skipped: {}", totals.skipped));
    });
    ui.label(format!(
        "Total session time: {}",
        stats::format_duration(totals.session_seconds)
    ));
    ui.label(format!(
        "Total break time: {}",
        stats::format_duration(totals.break_seconds)
    ));
    if totals.sessions == 0 && totals.breaks == 0 {
        ui.colored_label(app.palette.dim_text, "No activities yet");
    }
}

fn chart(ui: &mut egui::Ui, app: &App, today: i64) {
    let palette = app.palette;
    let series = match app
        .stats
        .daily_totals(today, app.settings.show_completed_only)
    {
        Ok(series) => series,
        Err(error) => {
            ui.label(format!("Chart read failed: {error}"));
            return;
        }
    };
    let height = 120.0;
    let (response, painter) =
        ui.allocate_painter(Vec2::new(ui.available_width(), height), Sense::hover());
    let rect = Rect::from_min_size(response.rect.min, Vec2::new(response.rect.width(), height));
    let max_seconds = series
        .iter()
        .map(|(_, totals)| totals.session_seconds + totals.break_seconds)
        .max()
        .unwrap_or(1)
        .max(1);

    painter.rect_filled(rect, 4.0, palette.panel);
    let columns = series.len().max(1) as f32;
    let column_width = rect.width() / columns;
    let bar_width = (column_width * 0.55).max(4.0);

    for (index, (day, totals)) in series.iter().enumerate() {
        let total = totals.session_seconds + totals.break_seconds;
        let bar_height = (height - 26.0) * (total as f32 / max_seconds as f32);
        let x = rect.min.x + column_width * (index as f32 + 0.5);
        let bar_rect = Rect::from_min_max(
            Pos2::new(x - bar_width / 2.0, rect.max.y - 18.0 - bar_height),
            Pos2::new(x + bar_width / 2.0, rect.max.y - 18.0),
        );
        let color = if total == 0 {
            palette.accent.gamma_multiply(0.15)
        } else {
            palette.accent
        };
        painter.rect_filled(bar_rect, 3.0, color);

        painter.text(
            Pos2::new(x, rect.max.y - 12.0),
            Align2::CENTER_CENTER,
            stats::format_date(*day)[5..].to_string(),
            FontId::proportional(10.0),
            palette.dim_text,
        );
        if totals.sessions > 0 {
            painter.text(
                Pos2::new(x, bar_rect.min.y - 8.0),
                Align2::CENTER_CENTER,
                totals.sessions.to_string(),
                FontId::proportional(10.0),
                palette.dim_text,
            );
        }
    }
    painter.rect_stroke(
        rect,
        4.0,
        Stroke::new(1.0, palette.accent.gamma_multiply(0.3)),
        egui::StrokeKind::Outside,
    );
}

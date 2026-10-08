//! Main timer window: circular progress, big time readout, and transport controls.

use egui::{Align2, FontId, Pos2, Sense, Stroke, Vec2};

use crate::app::{App, Panel};
use crate::config::Theme;
use crate::timer::RunState;

pub fn draw(app: &mut App, root: &mut egui::Ui) {
  let palette = app.palette;
  let frame = egui::Frame::central_panel(root.style())
    .fill(palette.background)
    .inner_margin(egui::Margin::same(18));
  egui::CentralPanel::default().frame(frame).show(root, |ui| {
    header(ui, app);
    ui.add_space(6.0);
    dial(ui, app);
    ui.add_space(10.0);
    controls(ui, app);
    ui.add_space(6.0);
    footer(ui, app);
    if !app.status.is_empty() {
      ui.add_space(4.0);
      ui.label(
        egui::RichText::new(&app.status)
          .color(palette.dim_text)
          .size(12.0),
      );
    }
  });
}

fn header(ui: &mut egui::Ui, app: &mut App) {
  ui.horizontal(|ui| {
    ui.heading(app.timer.activity().label());
    let state = match app.timer.state() {
      RunState::Idle => "Ready",
      RunState::Running => "Running",
      RunState::Paused => "Paused",
    };
    ui.colored_label(app.palette.dim_text, state);
  });
  ui.horizontal(|ui| {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
      let open = app.panel == Panel::Settings;
      let toggle = ui
        .selectable_label(open, "Settings")
        .on_hover_text("Settings (changes save automatically)");
      if toggle.clicked() {
        app.panel = if open { Panel::None } else { Panel::Settings };
      }
      let stats_open = app.panel == Panel::Statistics;
      let stats = ui
        .selectable_label(stats_open, "Statistics")
        .on_hover_text("Statistics and CSV export");
      if stats.clicked() {
        app.panel = if stats_open {
          Panel::None
        } else {
          Panel::Statistics
        };
      }
      if ui.button("Quit").clicked() {
        app.exit_requested = true;
      }
    });
  });
}

fn dial(ui: &mut egui::Ui, app: &App) {
  let palette = app.palette;
  let available = ui.available_width().min(ui.available_height() - 165.0);
  let size = Vec2::splat(available.max(160.0));
  let (response, painter) = ui.allocate_painter(size, Sense::hover());
  let rect = response.rect;
  let center = rect.center();
  let radius = (rect.width().min(rect.height()) / 2.0) - 10.0;

  painter.circle_stroke(center, radius, Stroke::new(10.0, palette.panel));

  let progress = app.timer.progress();
  if progress > 0.0 {
    // Open stroked arc: no polygon triangulation or image texture.
    let segments = 96;
    let start = -std::f32::consts::FRAC_PI_2;
    let sweep = progress * std::f32::consts::TAU;
    let mut points = Vec::with_capacity(segments + 1);
    for step in 0..=segments {
      let angle = start + sweep * (step as f32 / segments as f32);
      points.push(Pos2::new(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
      ));
    }
    painter.add(egui::Shape::line(points, Stroke::new(10.0, palette.accent)));
  }

  // Cycle pips: one per completed session in the current long-break cycle.
  let cycle = app.settings.long_break_cycle.max(1) as usize;
  let completed = (app.timer.completed_sessions() as usize) % cycle;
  let pip_radius = 4.0;
  let spacing = 16.0;
  let total_width = spacing * (cycle.saturating_sub(1)) as f32;
  let start_x = center.x - total_width / 2.0;
  let pip_y = center.y + radius - 34.0;
  for index in 0..cycle {
    let pip_center = Pos2::new(start_x + spacing * index as f32, pip_y);
    let color = if index < completed {
      palette.accent
    } else {
      palette.accent.gamma_multiply(0.25)
    };
    painter.circle_filled(pip_center, pip_radius, color);
  }

  let remaining = app.display_seconds();
  let minutes = remaining / 60;
  let seconds = remaining % 60;
  let sign = if app.is_overtime() { "+" } else { "" };
  let time_text = format!("{sign}{minutes:02}:{seconds:02}");

  // Fixed font size prevents a new font cache for every resize pixel.
  let time_size = 48.0;
  painter.text(
    center - Vec2::new(0.0, time_size * 0.25),
    Align2::CENTER_CENTER,
    time_text,
    FontId::monospace(time_size),
    palette.text,
  );

  // Session counter under the dial.
  let counter = format!(
    "Session {} of {} in cycle",
    (app.timer.completed_sessions() % cycle as u32) + 1,
    cycle
  );
  painter.text(
    center + Vec2::new(0.0, radius - 12.0),
    Align2::CENTER_CENTER,
    counter,
    FontId::proportional(12.0),
    palette.dim_text,
  );
}

fn controls(ui: &mut egui::Ui, app: &mut App) {
  let palette = app.palette;
  ui.vertical_centered(|ui| {
    ui.horizontal_wrapped(|ui| {
      if app.is_overtime() && ui.button("Finish").on_hover_text("F").clicked() {
        app.complete_current();
      }
      let running = app.timer.state() == RunState::Running;
      let (label, color) = if running {
        ("Pause", palette.panel)
      } else {
        ("Start", palette.background)
      };
      let button = egui::Button::new(egui::RichText::new(label).color(color).size(16.0))
        .fill(palette.accent)
        .min_size(Vec2::new(90.0, 36.0));
      if ui.add(button).on_hover_text("Space").clicked() {
        app.toggle();
      }
      if ui.button("Restart").on_hover_text("R").clicked() {
        app.restart();
      }
      if ui.button("Skip").on_hover_text("S").clicked() {
        app.skip();
      }
      if ui.button("Stop").on_hover_text("X").clicked() {
        app.stop();
      }
    });
  });
}

fn footer(ui: &mut egui::Ui, app: &mut App) {
  let palette = app.palette;
  ui.vertical(|ui| {
    ui.colored_label(
      palette.dim_text,
      format!(
        "{}m focus / {}m short / {}m long (every {})",
        app.settings.session_minutes,
        app.settings.short_break_minutes,
        app.settings.long_break_minutes,
        app.settings.long_break_cycle
      ),
    );
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
      let mut theme = app.settings.theme;
      egui::ComboBox::from_id_salt("theme")
        .selected_text(theme.label())
        .width(120.0)
        .show_ui(ui, |ui| {
          for option in Theme::ALL {
            ui.selectable_value(&mut theme, option, option.label());
          }
        });
      if theme != app.settings.theme {
        app.set_theme(theme);
      }
    });
  });
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::stats::StatsStore;

  #[test]
  fn accent_comes_from_the_palette() {
    let dir = std::env::temp_dir().join(format!("tomito-ui-test-{}", std::process::id()));
    let app = App::new(
      crate::config::Settings::default(),
      dir.join("settings.conf"),
      StatsStore::new(dir.join("stats.csv")),
    );
    assert_ne!(app.palette.accent, app.palette.background);
  }
}

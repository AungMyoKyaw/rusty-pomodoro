//! Persistent settings panel. Every editable value marks settings dirty.
use crate::app::{App, Panel};
use crate::config::{Sound, Theme};

pub fn draw(app: &mut App, ctx: &egui::Context) {
    if app.panel != Panel::Settings {
        return;
    }
    let before = app.settings;
    let mut open = true;
    egui::Window::new("Settings")
        .open(&mut open)
        .resizable(true)
        .default_width(360.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(ui, |ui| {
                    ui.heading("Durations");
                    minutes(ui, "Session", &mut app.settings.session_minutes, 1..=180);
                    minutes(
                        ui,
                        "Short break",
                        &mut app.settings.short_break_minutes,
                        1..=60,
                    );
                    minutes(
                        ui,
                        "Long break",
                        &mut app.settings.long_break_minutes,
                        1..=120,
                    );
                    ui.horizontal(|ui| {
                        ui.label("Long break after");
                        ui.add(
                            egui::DragValue::new(&mut app.settings.long_break_cycle)
                                .range(2..=10)
                                .suffix(" sessions"),
                        );
                    });
                    ui.add_space(8.0);
                    ui.heading("Timer");
                    ui.checkbox(
                        &mut app.settings.auto_start_session,
                        "Start sessions automatically",
                    );
                    ui.checkbox(
                        &mut app.settings.auto_start_break,
                        "Start breaks automatically",
                    );
                    ui.checkbox(&mut app.settings.disable_breaks, "Disable all breaks");
                    ui.checkbox(&mut app.settings.disable_long_breaks, "Disable long breaks");
                    ui.checkbox(
                        &mut app.settings.overtime,
                        "Continue past zero until Finish",
                    );
                    if ui.button("Reset session counter").clicked() {
                        app.timer.reset_counter();
                    }
                    ui.add_space(8.0);
                    ui.heading("Window");
                    #[cfg(target_os = "macos")]
                    {
                        ui.checkbox(&mut app.settings.hide_on_launch, "Hide on launch");
                        ui.checkbox(&mut app.settings.hide_on_start, "Hide when timer starts");
                        ui.checkbox(&mut app.settings.show_on_finish, "Show when timer ends");
                        ui.label(
                            egui::RichText::new(
                                "Restore from the menu bar. Closing the window hides it.",
                            )
                            .small(),
                        );
                    }
                    ui.checkbox(&mut app.settings.keep_in_front, "Keep in front");
                    #[cfg(target_os = "macos")]
                    {
                        ui.add_space(8.0);
                        ui.heading("Sleep");
                        ui.checkbox(&mut app.settings.pause_on_sleep, "Pause when Mac sleeps");
                        ui.checkbox(&mut app.settings.resume_on_wake, "Resume when Mac wakes");
                    }
                    ui.add_space(8.0);
                    ui.heading("Statistics");
                    ui.checkbox(
                        &mut app.settings.show_completed_only,
                        "Show completed activities only",
                    );
                    ui.add_space(8.0);
                    ui.heading("Appearance");
                    let mut theme = app.settings.theme;
                    egui::ComboBox::from_id_salt("settings-theme")
                        .selected_text(theme.label())
                        .show_ui(ui, |ui| {
                            for option in Theme::ALL {
                                ui.selectable_value(&mut theme, option, option.label());
                            }
                        });
                    if theme != app.settings.theme {
                        app.set_theme(theme);
                    }
                    #[cfg(target_os = "macos")]
                    {
                        ui.add_space(8.0);
                        ui.heading("Sounds");
                        let mut sound = app.settings.notification_sound;
                        egui::ComboBox::from_id_salt("sound")
                            .selected_text(sound.label())
                            .show_ui(ui, |ui| {
                                for option in Sound::ALL {
                                    ui.selectable_value(&mut sound, option, option.label());
                                }
                            });
                        if sound != app.settings.notification_sound {
                            app.set_sound(sound);
                        }
                        if ui.button("Preview sound").clicked() {
                            app.notify_requested = true;
                        }
                        ui.checkbox(&mut app.settings.ticking_sound, "Tick once per second");
                        ui.checkbox(
                            &mut app.settings.mute_ticking_on_break,
                            "Mute ticking during breaks",
                        );
                        ui.label(
                            egui::RichText::new("Uses macOS sounds. No bundled audio files.")
                                .small(),
                        );
                    }
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save now").clicked() {
                            app.apply_settings();
                            app.save_settings();
                        }
                        if ui.button("Close").clicked() {
                            app.panel = Panel::None;
                        }
                    });
                });
        });
    if before != app.settings {
        app.apply_settings();
    }
    if !open {
        app.panel = Panel::None;
    }
}
fn minutes(ui: &mut egui::Ui, label: &str, value: &mut u32, range: std::ops::RangeInclusive<u32>) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add(egui::DragValue::new(value).range(range).suffix(" min"));
    });
}

use std::time::{Duration, Instant};

use eframe::egui::{self, Align, Layout, RichText, TextureHandle, Ui, vec2};

use super::{menus, theme};

pub(super) const DURATION: Duration = Duration::from_millis(1800);

#[derive(Default)]
pub(super) struct Startup {
    pub(super) first_frame: Option<Instant>,
}

impl Startup {
    pub fn show(&mut self, ui: &mut Ui, logo: &TextureHandle, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(*self.first_frame.get_or_insert(now));
        let Some(remaining) = DURATION.checked_sub(elapsed).filter(|time| !time.is_zero()) else {
            return false;
        };
        ui.ctx().request_repaint_after(remaining);
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::CHROME))
            .show(ui, |ui| {
                let top = ((ui.available_height() - 221.0) / 2.0).max(0.0);
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.add_space(top);
                    ui.image((logo.id(), vec2(76.0, 76.0)));
                    ui.add_space(7.0);
                    ui.label(
                        RichText::new("SLX Viewer")
                            .size(29.0)
                            .strong()
                            .color(theme::BRAND),
                    );
                    ui.label(
                        RichText::new("Inspect saved SLX diagrams, signals, and block properties")
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new("Open-source project · MIT License · Read-only")
                            .color(theme::MUTED)
                            .small(),
                    );
                    ui.add_space(7.0);
                    menus::support_link(ui);
                    if let Some(owner) = menus::LICENSE
                        .lines()
                        .find_map(|line| line.strip_prefix("Copyright (c) "))
                    {
                        ui.add_space(7.0);
                        ui.label(
                            RichText::new(format!("© {owner}"))
                                .color(theme::MUTED)
                                .small(),
                        );
                    }
                });
            });
        true
    }
}

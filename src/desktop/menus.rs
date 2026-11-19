use std::path::PathBuf;

use eframe::egui::{
    self, Align, Button, Context, Id, Key, Layout, Modifiers, Popup, PopupCloseBehavior, RectAlign,
    Response, ScrollArea, Ui,
};

const NOTICES: &str = include_str!("../../THIRD_PARTY_NOTICES.txt");
pub(super) const LICENSE: &str = include_str!("../../LICENSE");
const SUPPORT_URL: &str = "https://www.paypal.com/paypalme/12dazapp";

pub(super) fn support_link(ui: &mut Ui) -> Response {
    let icon = Id::new("coffee-cup");
    let button = Button::new((
        egui::Atom::custom(icon, egui::vec2(18.0, 18.0)),
        egui::RichText::new("Buy us a coffee").color(ui.visuals().hyperlink_color),
    ))
    .gap(7.0)
    .atom_ui(ui);
    if let Some(rect) = button.rect(icon) {
        let at = |x, y| rect.min + egui::vec2(x, y);
        let stroke = egui::Stroke::new(1.35, ui.visuals().hyperlink_color);
        let painter = ui.painter();
        painter.add(egui::Shape::line(
            vec![
                at(2.5, 6.5),
                at(12.5, 6.5),
                at(12.5, 11.5),
                at(10.0, 14.5),
                at(5.0, 14.5),
                at(2.5, 11.5),
                at(2.5, 6.5),
            ],
            stroke,
        ));
        painter.add(egui::Shape::line(
            vec![
                at(12.5, 7.5),
                at(15.0, 7.5),
                at(16.0, 9.0),
                at(15.0, 11.0),
                at(12.5, 11.0),
            ],
            stroke,
        ));
        painter.line_segment([at(5.5, 4.0), at(5.5, 2.0)], stroke);
        painter.line_segment([at(9.5, 4.0), at(9.5, 1.5)], stroke);
    }
    let response = button
        .response
        .on_hover_text("Open PayPal to support SLX Viewer");
    if response.clicked() {
        ui.open_url(egui::OpenUrl::new_tab(SUPPORT_URL));
    }
    response
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Menu {
    File,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum HelpDialog {
    About,
    License,
}

#[derive(Debug, PartialEq)]
pub enum Action {
    Open,
    Recent(PathBuf),
}

fn readable_path(path: &std::path::Path) -> String {
    let display = path.display().to_string();
    if let Some(unc) = display.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{unc}")
    } else {
        display.strip_prefix(r"\\?\").unwrap_or(&display).to_owned()
    }
}

#[derive(Default)]
pub struct Menus {
    active: Option<Menu>,
    recent_open: bool,
    selected: usize,
    dialog: Option<HelpDialog>,
}

impl Menus {
    pub fn modal_open(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn close(&mut self) {
        self.active = None;
        self.recent_open = false;
        self.selected = 0;
    }

    fn activate(&mut self, recent: &[PathBuf]) -> Option<Action> {
        if self.recent_open {
            let action = recent.get(self.selected).cloned().map(Action::Recent);
            self.close();
            return action;
        }
        match (self.active, self.selected) {
            (Some(Menu::File), 0) => {
                self.close();
                Some(Action::Open)
            }
            (Some(Menu::File), _) if !recent.is_empty() => {
                self.recent_open = true;
                self.selected = 0;
                None
            }
            (Some(Menu::Help), index) => {
                self.dialog = Some(if index == 0 {
                    HelpDialog::About
                } else {
                    HelpDialog::License
                });
                self.close();
                None
            }
            _ => None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        recent: &[PathBuf],
        enabled: bool,
    ) -> (Option<Action>, bool) {
        let was_open = self.active.is_some() || self.dialog.is_some();
        let mut action = None;
        if enabled && self.dialog.is_none() {
            for (key, menu) in [(Key::F, Menu::File), (Key::H, Menu::Help)] {
                if ui.input_mut(|input| input.consume_key(Modifiers::ALT, key)) {
                    self.active = Some(menu);
                    self.selected = 0;
                    self.recent_open = false;
                }
            }
            if self.active.is_some() {
                if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
                    if self.recent_open {
                        self.recent_open = false;
                        self.selected = 1;
                    } else {
                        self.close();
                    }
                } else if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
                    if self.recent_open {
                        self.recent_open = false;
                        self.selected = 1;
                    } else {
                        self.active = Some(if self.active == Some(Menu::Help) {
                            Menu::File
                        } else {
                            Menu::Help
                        });
                        self.selected = 0;
                    }
                } else if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowRight))
                    && !self.recent_open
                {
                    if self.active == Some(Menu::File) && self.selected == 1 && !recent.is_empty() {
                        self.recent_open = true;
                    } else {
                        self.active = Some(if self.active == Some(Menu::File) {
                            Menu::Help
                        } else {
                            Menu::File
                        });
                        self.recent_open = false;
                    }
                    self.selected = 0;
                }
                let count = if self.recent_open {
                    recent.len()
                } else if self.active == Some(Menu::File) && recent.is_empty() {
                    1
                } else {
                    2
                };
                if count > 0 {
                    self.selected %= count;
                    if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                        self.selected = (self.selected + 1) % count;
                    }
                    if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                        self.selected = (self.selected + count - 1) % count;
                    }
                    if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter)) {
                        action = self.activate(recent);
                    }
                }
            }
        }
        let mut regions = Vec::new();
        ui.add_enabled_ui(enabled && self.dialog.is_none(), |ui| {
            ui.horizontal(|ui| {
                egui::menu::menu_style(ui.style_mut());
                for (menu, title) in [(Menu::File, "File"), (Menu::Help, "Help")] {
                    let response = ui.selectable_label(self.active == Some(menu), title);
                    regions.push(response.rect);
                    let moving = ui.input(|input| input.pointer.delta() != egui::Vec2::ZERO);
                    if response.clicked()
                        || (self.active.is_some()
                            && moving
                            && response.hovered()
                            && self.active != Some(menu))
                    {
                        if self.active == Some(menu) && response.clicked() {
                            self.close();
                        } else {
                            self.active = Some(menu);
                            self.recent_open = false;
                            self.selected = 0;
                        }
                    }
                    if self.active == Some(menu) {
                        let mut open = true;
                        let popup = Popup::menu(&response)
                            .open_bool(&mut open)
                            .layout(Layout::top_down_justified(Align::Min))
                            .close_behavior(PopupCloseBehavior::IgnoreClicks)
                            .show(|ui| {
                                ui.set_min_width(230.0);
                                match menu {
                                    Menu::File => {
                                        let open_button = ui.add(
                                            Button::new("Open…")
                                                .shortcut_text("Ctrl+O")
                                                .selected(!self.recent_open && self.selected == 0),
                                        );
                                        if open_button.hovered() && moving {
                                            self.recent_open = false;
                                            self.selected = 0;
                                        }
                                        if open_button.clicked() {
                                            self.close();
                                            action = Some(Action::Open);
                                        }
                                        let recent_button = ui.add_enabled(
                                            !recent.is_empty(),
                                            Button::new("Open Recent  >")
                                                .selected(self.recent_open || self.selected == 1),
                                        );
                                        if recent_button.clicked() {
                                            self.recent_open = !self.recent_open;
                                            self.selected = 0;
                                        } else if recent_button.hovered()
                                            && moving
                                            && !recent.is_empty()
                                        {
                                            self.recent_open = true;
                                            self.selected = 0;
                                        }
                                        if self.recent_open {
                                            let mut submenu_open = true;
                                            let submenu = Popup::menu(&recent_button)
                                                .id(Id::new("recent-models-menu"))
                                                .open_bool(&mut submenu_open)
                                                .align(RectAlign::RIGHT_START)
                                                .layout(Layout::top_down_justified(Align::Min))
                                                .close_behavior(PopupCloseBehavior::IgnoreClicks)
                                                .show(|ui| {
                                                    for (index, path) in recent.iter().enumerate() {
                                                        let name = path
                                                            .file_name()
                                                            .unwrap_or(path.as_os_str())
                                                            .to_string_lossy();
                                                        let parent = readable_path(
                                                            path.parent().unwrap_or(path),
                                                        );
                                                        let response = self
                                                            .row(
                                                                ui,
                                                                index,
                                                                &format!("{name} — {parent}"),
                                                                true,
                                                            )
                                                            .on_hover_text(readable_path(path));
                                                        if response.clicked() {
                                                            self.selected = index;
                                                            action = self.activate(recent);
                                                        }
                                                    }
                                                });
                                            if let Some(submenu) = submenu {
                                                regions.push(submenu.response.rect);
                                            }
                                        }
                                    }
                                    Menu::Help => {
                                        for (index, label) in
                                            ["About…", "License…"].iter().enumerate()
                                        {
                                            if self.row(ui, index, label, true).clicked() {
                                                self.selected = index;
                                                action = self.activate(recent);
                                            }
                                        }
                                    }
                                }
                            });
                        if let Some(popup) = popup {
                            regions.push(popup.response.rect);
                        }
                    }
                }
            });
        });
        if self.active.is_some()
            && ui.input(|input| {
                input.pointer.any_click()
                    && input
                        .pointer
                        .interact_pos()
                        .is_some_and(|position| !regions.iter().any(|rect| rect.contains(position)))
            })
        {
            self.close();
        }
        let blocked =
            was_open || self.active.is_some() || self.dialog.is_some() || action.is_some();
        (action, blocked)
    }

    fn row(&mut self, ui: &mut Ui, index: usize, label: &str, enabled: bool) -> Response {
        let response = ui.add_enabled(
            enabled,
            Button::new(label).wrap().selected(self.selected == index),
        );
        if response.hovered() && ui.input(|input| input.pointer.delta() != egui::Vec2::ZERO) {
            self.selected = index;
        }
        response
    }

    pub fn dialogs(&mut self, ctx: &Context, backend: &str) {
        let Some(dialog) = self.dialog else { return };
        let response = egui::Modal::new(Id::new("help-dialog")).show(ctx, |ui| {
            ui.set_width(620.0_f32.min(ctx.content_rect().width() - 40.0).max(240.0));
            match dialog {
                HelpDialog::About => {
                    ui.heading("About SLX Viewer");
                    egui::Grid::new("build-details")
                        .spacing([18.0, 8.0])
                        .show(ui, |ui| {
                            for (label, value) in [
                                ("Application", "SLX Viewer"),
                                ("Version", env!("CARGO_PKG_VERSION")),
                                ("Built at (UTC)", env!("SLX_BUILD_UTC")),
                                ("Source revision", env!("SLX_BUILD_REVISION")),
                                ("Target", env!("SLX_BUILD_TARGET")),
                                ("Build profile", env!("SLX_BUILD_PROFILE")),
                                ("Graphics backend", backend),
                            ] {
                                ui.label(label);
                                ui.label(value);
                                ui.end_row();
                            }
                        });
                    ui.separator();
                    support_link(ui);
                }
                HelpDialog::License => {
                    ui.heading("License");
                    ui.label(LICENSE);
                    ui.separator();
                    ui.strong("Third-party licenses and notices");
                    let height = ui.text_style_height(&egui::TextStyle::Monospace);
                    ScrollArea::both()
                        .max_height((ctx.content_rect().height() - 220.0).clamp(100.0, 480.0))
                        .show_rows(ui, height, NOTICES.lines().count(), |ui, range| {
                            for line in NOTICES.lines().skip(range.start).take(range.len()) {
                                ui.add(
                                    egui::Label::new(egui::RichText::new(line).monospace())
                                        .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            }
                        });
                }
            }
            ui.separator();
            ui.button("Close").clicked()
        });
        if response.inner || response.should_close() {
            ctx.input_mut(|input| {
                input.consume_key(Modifiers::NONE, Key::Escape);
            });
            self.dialog = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coffee_icon_opens_the_support_page() {
        let ctx = Context::default();
        let frame = |events, button: &mut egui::Rect| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ui| *button = support_link(ui).rect,
            );
            output.textures_delta.clear();
            output
        };
        let mut button = egui::Rect::NOTHING;
        frame(Vec::new(), &mut button);
        let pos = egui::pos2(button.left() + 14.0, button.center().y);
        let mut events = vec![egui::Event::PointerMoved(pos)];
        events.extend([true, false].map(|pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }));
        let output = frame(events, &mut button);
        assert!(output.platform_output.commands.iter().any(|command| {
            matches!(command, egui::OutputCommand::OpenUrl(link) if link.url == SUPPORT_URL)
        }));
    }

    #[test]
    fn recent_submenu_supports_arrow_navigation_and_escape() {
        let ctx = Context::default();
        let mut menus = Menus::default();
        let recent = [PathBuf::from("first.slx"), PathBuf::from("second.slx")];
        let frame = |menus: &mut Menus, key, modifiers| {
            let mut result = None;
            let events = [true, false]
                .map(|pressed| egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers,
                })
                .to_vec();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 650.0),
                    )),
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ui| {
                    result = menus.show(ui, &recent, true).0;
                },
            );
            output.textures_delta.clear();
            result
        };
        frame(&mut menus, Key::F, Modifiers::ALT);
        assert_eq!(menus.active, Some(Menu::File));
        frame(&mut menus, Key::ArrowDown, Modifiers::NONE);
        frame(&mut menus, Key::ArrowRight, Modifiers::NONE);
        assert!(menus.recent_open);
        frame(&mut menus, Key::Escape, Modifiers::NONE);
        assert!(!menus.recent_open);
        assert_eq!(menus.active, Some(Menu::File));
        frame(&mut menus, Key::ArrowRight, Modifiers::NONE);
        frame(&mut menus, Key::ArrowDown, Modifiers::NONE);
        assert_eq!(
            frame(&mut menus, Key::Enter, Modifiers::NONE),
            Some(Action::Recent(recent[1].clone()))
        );
        assert!(menus.active.is_none());
    }
}

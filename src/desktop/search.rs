use eframe::egui::{
    self, Key, Modifiers, Popup, PopupCloseBehavior, RectAlign, ScrollArea, TextEdit, Ui,
};
use slx_viewer::model::{BlockIndex, Model, SystemId};

const MAX_RESULTS: usize = 50;

#[derive(Default)]
pub(super) struct Search {
    query: String,
    open: bool,
    results: Vec<Match>,
}

struct Match {
    system: SystemId,
    block: BlockIndex,
    path: String,
}

impl Search {
    fn update_results(&mut self, model: &Model) {
        self.results.clear();
        let query = self.query.trim().to_lowercase();
        if query.is_empty() {
            return;
        }
        for (system, contents) in model.systems.iter().enumerate() {
            let mut names = vec![contents.name.as_str()];
            let mut parent = contents.parent;
            while let Some((id, _)) = parent {
                names.push(&model.systems[id].name);
                parent = model.systems[id].parent;
            }
            names.reverse();
            let path = names.join("/");
            let prefix = format!("{}/", path.to_lowercase());
            for (block, item) in contents.blocks.iter().enumerate() {
                if format!("{prefix}{}", item.name.to_lowercase()).contains(&query) {
                    self.results.push(Match {
                        system,
                        block,
                        path: path.clone(),
                    });
                    if self.results.len() == MAX_RESULTS {
                        return;
                    }
                }
            }
        }
    }

    pub fn show(&mut self, ui: &mut Ui, model: &Model) -> (Option<(SystemId, BlockIndex)>, bool) {
        let width = (ui.available_width() * 0.30).clamp(155.0, 245.0);
        let edit = ui
            .add_sized(
                [width, 32.0],
                TextEdit::singleline(&mut self.query)
                    .hint_text("Search blocks…")
                    .margin(egui::Margin::symmetric(9, 0))
                    .vertical_align(egui::Align::Center)
                    .desired_width(width),
            )
            .on_hover_text("Search block names and paths (Ctrl+K)");
        if !ui.is_enabled() {
            return (None, false);
        }
        if ui.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::K)) {
            edit.request_focus();
            self.open = true;
        }
        if edit.changed() {
            self.update_results(model);
        }
        if edit.gained_focus() || edit.clicked() || edit.changed() {
            self.open = true;
        }
        let was_active = edit.has_focus() || edit.lost_focus() || self.open;
        if was_active && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Escape)) {
            self.open = false;
            edit.surrender_focus();
            return (None, true);
        }
        let mut selected = None;
        if self.open && !self.query.trim().is_empty() {
            if edit.lost_focus()
                && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter))
            {
                selected = self.results.first().map(|item| (item.system, item.block));
            }
            Popup::from_response(&edit)
                .align(RectAlign::BOTTOM_END)
                .gap(3.0)
                .open_bool(&mut self.open)
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.set_width(width);
                    ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                        if self.results.is_empty() {
                            ui.weak("No matching blocks");
                        }
                        for item in &self.results {
                            let name = &model.systems[item.system].blocks[item.block].name;
                            let label = format!("{name}  ·  {}", item.path);
                            if ui
                                .add(egui::Button::new(&label).truncate())
                                .on_hover_text(label)
                                .clicked()
                            {
                                selected = Some((item.system, item.block));
                            }
                        }
                    });
                });
        } else if edit.lost_focus() {
            self.open = false;
        }
        if selected.is_some() {
            self.open = false;
            self.query.clear();
            self.results.clear();
            edit.surrender_focus();
        }
        (selected, was_active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slx_viewer::{Limits, load};

    #[test]
    fn search_matches_block_names_and_disambiguates_system_paths() {
        let fixture = "tests/fixtures/dc_motor_speed_control.slx";
        let mut model = load(fixture, &Limits::default()).unwrap();
        let mut child = load(fixture, &Limits::default()).unwrap().systems.remove(0);
        child.name = "Controller".into();
        child.parent = Some((model.root, 0));
        model.systems.push(child);
        let mut search = Search {
            query: "backemf".into(),
            ..Search::default()
        };

        search.update_results(&model);
        assert_eq!(search.results.len(), 2);
        assert_eq!(search.results[0].path, "dc_motor_speed_control");
        assert_eq!(search.results[1].path, "dc_motor_speed_control/Controller");

        search.query = "CONTROLLER/BackEMF".into();
        search.update_results(&model);
        assert_eq!(search.results.len(), 1);
        let result = &search.results[0];
        assert_eq!(
            model.systems[result.system].blocks[result.block].properties["Gain"],
            "p.Ke"
        );
    }
}

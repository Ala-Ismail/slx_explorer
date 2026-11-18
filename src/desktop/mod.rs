mod canvas;
mod menus;
mod scene;
mod search;
mod settings;
mod startup;
mod stateflow;
mod theme;
mod worker;

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::Instant,
};

use eframe::egui::{
    self, Align, Context, Key, Layout, Modifiers, Panel, RichText, ScrollArea, Ui, vec2,
};
use slx_viewer::model::{
    BlockIndex, Model, SystemId,
    stateflow::{ChartId, ChartNodeKind},
};

use self::{
    canvas::{View, ViewAction},
    menus::{Action, Menus},
    scene::Scene,
    search::Search,
    settings::Settings,
    startup::Startup,
    stateflow::ChartScene,
    worker::Loader,
};

pub fn run() -> eframe::Result {
    let initial_path = std::env::args_os().nth(1).map(PathBuf::from);
    let icon = application_icon();
    eframe::run_native(
        "SLX Viewer",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 800.0])
                .with_min_inner_size([740.0, 460.0])
                .with_icon(icon.clone()),
            ..Default::default()
        },
        Box::new(move |creation| {
            let settings = Settings::load();
            theme::apply(&creation.egui_ctx);
            let mut app = App {
                loader: Loader::new(creation.egui_ctx.clone())?,
                tabs: Vec::new(),
                active_slot: 0,
                current_path: None,
                document: None,
                error: None,
                show_diagnostics: false,
                settings,
                menus: Menus::default(),
                logo: logo_texture(&creation.egui_ctx, &icon),
                startup: Some(Startup::default()),
                backend: if creation.gl.is_some() {
                    "OpenGL (glow)".into()
                } else {
                    "Unavailable".into()
                },
            };
            if let Some(path) = initial_path {
                app.open(path);
            }
            Ok(Box::new(app))
        }),
    )
}

fn application_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../../assets/slx-viewer-logo.png"))
        .expect("the embedded SLX Viewer logo is a PNG")
}

fn logo_texture(ctx: &Context, icon: &egui::IconData) -> egui::TextureHandle {
    ctx.load_texture(
        "SLX Viewer logo",
        egui::ColorImage::from_rgba_unmultiplied(
            [icon.width as usize, icon.height as usize],
            &icon.rgba,
        ),
        egui::TextureOptions::LINEAR,
    )
}

struct SavedTab {
    path: PathBuf,
    document: Document,
}

struct App {
    active_slot: usize,
    tabs: Vec<SavedTab>,
    current_path: Option<PathBuf>,
    loader: Loader,
    document: Option<Document>,
    error: Option<String>,
    show_diagnostics: bool,
    settings: Settings,
    menus: Menus,
    logo: egui::TextureHandle,
    startup: Option<Startup>,
    backend: String,
}

impl App {
    fn open(&mut self, path: PathBuf) {
        let path = std::fs::canonicalize(&path).unwrap_or(path);
        if self.current_path.as_ref() == Some(&path) {
            return;
        }
        if let Some(index) = self.tabs.iter().position(|tab| tab.path == path) {
            self.switch_tab(index);
            return;
        }
        self.error = self.loader.open(path).err().map(str::to_owned);
    }
    fn receive_load(&mut self, ctx: &Context) {
        if let Some(result) = self.loader.poll() {
            match result {
                Ok((path, model)) => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
                        "{} - SLX Viewer",
                        model.name
                    )));
                    if let (Some(path), Some(document)) =
                        (self.current_path.take(), self.document.take())
                    {
                        self.tabs
                            .insert(self.active_slot, SavedTab { path, document });
                    }
                    self.active_slot = self.tabs.len();
                    self.current_path = Some(path.clone());
                    self.document = Some(Document::new(model));
                    self.error = None;
                    self.show_diagnostics = false;
                    self.settings.record_success(&path);
                }
                Err(error) => self.error = Some(error),
            }
        }
    }

    fn switch_tab(&mut self, index: usize) {
        let target_slot = if self.document.is_some() && index >= self.active_slot {
            index + 1
        } else {
            index
        };
        let tab = self.tabs.remove(index);
        if let (Some(path), Some(document)) = (self.current_path.take(), self.document.take()) {
            self.tabs.insert(
                self.active_slot - usize::from(index < self.active_slot),
                SavedTab { path, document },
            );
        }
        self.active_slot = target_slot;
        self.current_path = Some(tab.path);
        self.document = Some(tab.document);
        self.show_diagnostics = false;
        self.error = None;
    }

    fn contents(&mut self, ui: &mut Ui) -> bool {
        if let Some(path) = ui.input_mut(|input| {
            std::mem::take(&mut input.raw.dropped_files)
                .first()
                .map(|file| file.path().to_owned())
        }) {
            self.open(path);
        }
        if let Some(startup) = &mut self.startup {
            if startup.show(ui, &self.logo, Instant::now()) {
                return false;
            }
            self.startup = None;
        }
        let mut menu_action = None;
        let mut menu_blocked = false;
        Panel::top("menubar")
            .exact_size(42.0)
            .frame(theme::bar(theme::CHROME))
            .show(ui, |ui| {
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.image((self.logo.id(), vec2(28.0, 28.0)));
                    ui.label(RichText::new("SLX Viewer").strong().color(theme::BRAND));
                    chrome_divider(ui);
                    (menu_action, menu_blocked) = self.menus.show(
                        ui,
                        &self.settings.preferences.recent,
                        !self.show_diagnostics,
                    );
                });
            });
        let mut open = !self.show_diagnostics
            && !self.menus.modal_open()
            && ui.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::O));
        match menu_action {
            Some(Action::Open) => open = true,
            Some(Action::Recent(path)) => self.open(path),
            None => {}
        }
        if open {
            self.menus.close();
        }
        {
            let tabs_enabled = !self.show_diagnostics && !self.menus.modal_open();
            let mut selected = None;
            let mut close = None;
            Panel::top("model-tabs")
                .exact_size(36.0)
                .frame(
                    egui::Frame::new()
                        .fill(theme::CHROME)
                        .inner_margin(egui::Margin {
                            left: 8,
                            right: 8,
                            top: 4,
                            bottom: 0,
                        }),
                )
                .show(ui, |ui| {
                    if !tabs_enabled {
                        ui.disable();
                    }
                    ScrollArea::horizontal().show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            for slot in
                                0..self.tabs.len() + usize::from(self.current_path.is_some())
                            {
                                let active =
                                    self.current_path.is_some() && slot == self.active_slot;
                                let index = if slot > self.active_slot {
                                    slot - 1
                                } else {
                                    slot
                                };
                                let path = if active {
                                    self.current_path.as_ref().unwrap()
                                } else {
                                    &self.tabs[index].path
                                };
                                ui.push_id(path, |ui| {
                                    let visuals = ui.visuals();
                                    let fill = if active {
                                        theme::TOOLBAR
                                    } else {
                                        theme::CHROME
                                    };
                                    let stroke = if active {
                                        visuals.selection.stroke
                                    } else {
                                        visuals.widgets.inactive.bg_stroke
                                    };
                                    let tab = egui::Frame::new()
                                        .fill(fill)
                                        .corner_radius(egui::CornerRadius {
                                            nw: 4,
                                            ne: 4,
                                            sw: 0,
                                            se: 0,
                                        })
                                        .inner_margin(egui::Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(RichText::new("□").color(theme::FOCUS));
                                                if ui
                                                    .add(
                                                        egui::Button::new(
                                                            path.file_name()
                                                                .unwrap_or_default()
                                                                .to_string_lossy(),
                                                        )
                                                        .frame(false),
                                                    )
                                                    .on_hover_text(path.display().to_string())
                                                    .clicked()
                                                    && !active
                                                {
                                                    selected = Some(index);
                                                }
                                                if ui
                                                    .add(egui::Button::new("×").frame(false))
                                                    .on_hover_text("Close tab (Ctrl+W)")
                                                    .clicked()
                                                {
                                                    close = Some(if active {
                                                        self.tabs.len()
                                                    } else {
                                                        index
                                                    });
                                                }
                                            });
                                        });
                                    let bounds = tab.response.rect;
                                    let outline = vec![
                                        bounds.left_bottom(),
                                        bounds.left_top(),
                                        bounds.right_top(),
                                        bounds.right_bottom(),
                                    ];
                                    ui.painter().add(egui::Shape::line(outline, stroke));
                                    if !active {
                                        ui.painter().line_segment(
                                            [bounds.left_bottom(), bounds.right_bottom()],
                                            egui::Stroke::new(1.0, theme::DIVIDER),
                                        );
                                    }
                                });
                            }
                        });
                    });
                });
            if tabs_enabled && ui.input_mut(|input| input.consume_key(Modifiers::CTRL, Key::W)) {
                close = Some(self.tabs.len());
            }
            if let Some(index) = selected {
                self.switch_tab(index);
            }
            if let Some(index) = close {
                if index < self.tabs.len() {
                    self.tabs.remove(index);
                    if index < self.active_slot {
                        self.active_slot -= 1;
                    }
                } else {
                    self.document = None;
                    self.current_path = None;
                    if !self.tabs.is_empty() {
                        self.switch_tab(self.active_slot.min(self.tabs.len() - 1));
                    }
                }
            }
        }
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(
            self.current_path.as_ref().map_or_else(
                || "SLX Viewer".into(),
                |path| {
                    format!(
                        "{} - SLX Viewer",
                        path.file_stem().unwrap_or_default().to_string_lossy()
                    )
                },
            ),
        ));
        let document = &mut self.document;
        let mut search_active = false;
        Panel::top("toolbar")
            .exact_size(45.0)
            .frame(theme::bar(theme::TOOLBAR))
            .show(ui, |ui| {
                if let Some(document) = document.as_mut() {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let (selected, active) = ui
                            .add_enabled_ui(
                                !menu_blocked && !self.show_diagnostics && !open,
                                |ui| document.search.show(ui, &document.model),
                            )
                            .inner;
                        search_active = active;
                        if let Some((system, block)) = selected {
                            document.select_block(system, block);
                        }
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            document.toolbar(ui)
                        });
                    });
                } else {
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        navigation_icon(ui, false, false);
                        navigation_icon(ui, false, true);
                    });
                }
            });
        if let Some(document) = document.as_mut()
            && !self.show_diagnostics
            && !menu_blocked
            && !open
            && !search_active
        {
            document.keyboard_navigation(ui);
        }
        Panel::bottom("status")
            .exact_size(45.0)
            .frame(theme::bar(theme::CHROME))
            .show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let view = document
                        .as_mut()
                        .map(|document| document.views.entry(document.current).or_default());
                    zoom_controls(ui, view);
                    let size = egui::vec2(ui.available_width(), ui.spacing().interact_size.y);
                    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
                        ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                        ui.label(RichText::new("Read-only").small().strong());
                        chrome_divider(ui);
                        if let Some(document) = document.as_ref() {
                            if !document.model.diagnostics.is_empty()
                                && ui
                                    .button(format!(
                                        "Diagnostics ({})",
                                        document.model.diagnostics.len()
                                    ))
                                    .clicked()
                            {
                                self.show_diagnostics = true;
                            }
                            if let Some(release) = &document.model.release {
                                ui.label(RichText::new(release).small().color(theme::MUTED));
                            }
                        }
                        if self.loader.pending_path().is_some() && ui.button("Cancel").clicked() {
                            self.loader.cancel();
                        }
                        let (status, color) = if let Some(error) = &self.error {
                            (error.clone(), ui.visuals().error_fg_color)
                        } else if let Some(warning) = &self.settings.warning {
                            (warning.clone(), ui.visuals().warn_fg_color)
                        } else if let Some(path) = self.loader.pending_path() {
                            (
                                format!("Loading {}", path.display()),
                                ui.visuals().text_color(),
                            )
                        } else {
                            (
                                document
                                    .as_ref()
                                    .map(|document| document.status_text())
                                    .unwrap_or_else(|| "No model loaded".into()),
                                theme::MUTED,
                            )
                        };
                        ui.add(
                            egui::Label::new(RichText::new(&status).small().color(color))
                                .truncate(),
                        )
                        .on_hover_text(&status);
                    });
                });
            });
        let width = ui.available_width();
        Panel::left("hierarchy")
            .default_size((width * 0.18).clamp(170.0, 250.0))
            .size_range(145.0..=(width * 0.26).max(170.0))
            .frame(theme::bar(theme::PANEL).inner_margin(egui::Margin::symmetric(12, 6)))
            .resizable(true)
            .show(ui, |ui| {
                panel_heading(ui, "Model hierarchy");
                if let Some(document) = document.as_mut() {
                    let mut target = None;
                    ScrollArea::vertical().show(ui, |ui| {
                        target = document.tree(ui, document.model.root);
                    });
                    document.reveal_tree_selection = false;
                    if let Some(target) = target {
                        document.navigate(target);
                    }
                } else {
                    ui.weak("No model open");
                }
            });
        Panel::right("properties")
            .default_size((width * 0.20).clamp(190.0, 278.0))
            .size_range(155.0..=(width * 0.28).max(190.0))
            .frame(theme::bar(theme::PANEL).inner_margin(egui::Margin::symmetric(12, 6)))
            .resizable(true)
            .show(ui, |ui| {
                panel_heading(ui, "Properties");
                if let Some(document) = document.as_ref() {
                    document.properties(ui);
                } else {
                    ui.weak("Open a model to inspect blocks.");
                }
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(theme::CANVAS))
            .show(ui, |ui| {
                theme::canvas_background(ui.painter(), ui.max_rect());
                if let Some(document) = document.as_mut() {
                    document.workspace(ui);
                } else {
                    let top = ((ui.available_height() - 70.0) / 2.0).max(0.0);
                    ui.with_layout(Layout::top_down(Align::Center), |ui| {
                        ui.add_space(top);
                        if self.loader.pending_path().is_some() {
                            ui.spinner();
                            ui.label("Loading model…");
                        } else {
                            ui.heading("No model open");
                            ui.add_space(4.0);
                            open |= ui.button("Open SLX model…").clicked();
                        }
                    });
                }
            });
        if self.show_diagnostics
            && let Some(document) = self.document.as_ref()
        {
            let dialog = egui::Modal::new(egui::Id::new("diagnostics")).show(ui.ctx(), |ui| {
                ui.heading("Diagnostics");
                ScrollArea::vertical().max_height(330.0).show(ui, |ui| {
                    for diagnostic in &document.model.diagnostics {
                        let object = diagnostic.object.as_deref().unwrap_or("Diagram");
                        ui.label(format!("{object}: {}", diagnostic.message));
                        ui.small(&diagnostic.part);
                        ui.separator();
                    }
                });
                ui.button("Close").clicked()
            });
            if dialog.inner || dialog.should_close() {
                self.show_diagnostics = false;
            }
        }
        self.menus.dialogs(ui.ctx(), &self.backend);
        open && !self.show_diagnostics && !self.menus.modal_open()
    }
}

fn panel_heading(ui: &mut Ui, heading: &str) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 25.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.strong(heading);
        },
    );
    ui.separator();
}

fn chrome_divider(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, 20.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0, theme::DIVIDER);
}

fn navigation_icon(ui: &mut Ui, enabled: bool, up: bool) -> egui::Response {
    let response = ui.add_enabled(enabled, egui::Button::new("").min_size(vec2(32.0, 28.0)));
    let visuals = ui.style().interact(&response);
    let direction = if up {
        egui::vec2(0.0, -10.0)
    } else {
        egui::vec2(-10.0, 0.0)
    };
    ui.painter().arrow(
        response.rect.center() - direction * 0.5,
        direction,
        egui::Stroke::new(
            1.5,
            visuals
                .fg_stroke
                .color
                .gamma_multiply(if enabled { 1.0 } else { 0.35 }),
        ),
    );
    response
}

fn zoom_controls(ui: &mut Ui, view: Option<&mut View>) {
    let mut action = None;
    ui.add_enabled_ui(view.is_some(), |ui| {
        // The status row flows from the right edge toward the left.
        if ui
            .add_sized([28.0, 28.0], egui::Button::new("+"))
            .on_hover_text("Zoom in")
            .clicked()
        {
            action = Some(ViewAction::Scale(1.25));
        }
        let percent = view.as_ref().map_or(100.0, |view| view.percent());
        if ui
            .add_sized([53.0, 28.0], egui::Button::new(format!("{percent:.0}%")))
            .on_hover_text("Reset to 100%")
            .clicked()
        {
            action = Some(ViewAction::Reset);
        }
        if ui
            .add_sized([28.0, 28.0], egui::Button::new("−"))
            .on_hover_text("Zoom out")
            .clicked()
        {
            action = Some(ViewAction::Scale(1.0 / 1.25));
        }
        if ui
            .add_sized([36.0, 28.0], egui::Button::new("Fit"))
            .on_hover_text("Fit current diagram")
            .clicked()
        {
            action = Some(ViewAction::Fit);
        }
    });
    if let (Some(view), Some(action)) = (view, action) {
        view.action = Some(action);
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        self.receive_load(ctx);
    }

    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        if self.contents(ui)
            && let Some(path) = rfd::FileDialog::new()
                .set_parent(frame)
                .set_title("Open Simulink model")
                .add_filter("Simulink model", &["slx"])
                .pick_file()
        {
            self.open(path);
        }
    }
}

struct Document {
    model: Model,
    current: Location,
    history: Vec<Location>,
    views: HashMap<Location, View>,
    scenes: HashMap<SystemId, Scene>,
    chart_scenes: HashMap<ChartId, ChartScene>,
    expanded: HashSet<SystemId>,
    reveal_tree_selection: bool,
    search: Search,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Location {
    System(SystemId),
    Chart { owner: SystemId, block: BlockIndex },
}

impl Location {
    fn system(self) -> SystemId {
        match self {
            Self::System(system) => system,
            Self::Chart { owner, .. } => owner,
        }
    }
}

impl Document {
    fn new(model: Model) -> Self {
        let root = model.root;
        Self {
            model,
            current: Location::System(root),
            history: Vec::new(),
            views: HashMap::new(),
            scenes: HashMap::new(),
            chart_scenes: HashMap::new(),
            expanded: HashSet::from([root]),
            reveal_tree_selection: true,
            search: Search::default(),
        }
    }

    fn show_location(&mut self, target: Location) {
        self.current = target;
        self.reveal_tree_selection = true;
        let mut cursor = target.system();
        if matches!(target, Location::Chart { .. }) {
            self.expanded.insert(cursor);
        }
        while let Some((parent, _)) = self.model.systems[cursor].parent {
            self.expanded.insert(parent);
            cursor = parent;
        }
    }

    fn navigate(&mut self, target: Location) {
        if target != self.current {
            self.history.push(self.current);
            self.show_location(target);
        }
    }

    fn back(&mut self) {
        if let Some(previous) = self.history.pop() {
            self.show_location(previous);
        }
    }

    fn up(&mut self) {
        if let Some((parent, block)) = self.parent() {
            self.select_block(parent, block);
        }
    }

    fn select_block(&mut self, system: SystemId, block: BlockIndex) {
        let location = Location::System(system);
        self.navigate(location);
        let view = self.views.entry(location).or_default();
        view.selected = Some(block);
        view.reveal = Some(block);
    }

    fn keyboard_navigation(&mut self, ui: &Ui) {
        if ui.input_mut(|input| {
            input.consume_key(Modifiers::NONE, Key::Escape)
                || input.consume_key(Modifiers::ALT, Key::ArrowUp)
        }) {
            self.up();
        }
        if ui.input_mut(|input| input.consume_key(Modifiers::ALT, Key::ArrowLeft)) {
            self.back();
        }
    }

    fn parent(&self) -> Option<(SystemId, BlockIndex)> {
        match self.current {
            Location::System(system) => self.model.systems[system].parent,
            Location::Chart { owner, block } => Some((owner, block)),
        }
    }

    fn block_target(&self, owner: SystemId, block: BlockIndex) -> Option<Location> {
        let value = &self.model.systems[owner].blocks[block];
        if value.chart.is_some() {
            Some(Location::Chart { owner, block })
        } else {
            value.child.map(Location::System)
        }
    }

    fn chart_id(&self, owner: SystemId, block: BlockIndex) -> ChartId {
        self.model.systems[owner].blocks[block]
            .chart
            .expect("chart locations have a chart")
    }

    fn status_text(&self) -> String {
        match self.current {
            Location::System(system) => {
                let system = &self.model.systems[system];
                let mut status = format!(
                    "{} blocks · {} signals",
                    system.blocks.len(),
                    system.connections.len()
                );
                if !system.connections.is_empty() {
                    status.push_str(". Port positions and wire endpoints are estimated.");
                }
                status
            }
            Location::Chart { owner, block } => {
                let id = self.chart_id(owner, block);
                let chart = &self.model.charts[id];
                let states = chart
                    .nodes
                    .iter()
                    .filter(|node| node.kind == ChartNodeKind::State)
                    .count();
                let mut status = format!(
                    "Stateflow · {states} states · {} transitions",
                    chart.transitions.len()
                );
                if !chart.transitions.is_empty() {
                    status.push_str(". Transition curves are approximated from saved points.");
                }
                if let Some(scene) = self.chart_scenes.get(&id) {
                    if scene.hidden_subviews {
                        status.push_str(" Separate subviews are unavailable.");
                    }
                    if scene.missing_geometry {
                        status.push_str(" Some objects lack usable geometry.");
                    }
                }
                status
            }
        }
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        let back = navigation_icon(ui, !self.history.is_empty(), false)
            .on_disabled_hover_text("Back (Alt+Left)")
            .on_hover_text("Back (Alt+Left)");
        back.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, back.enabled(), "Back")
        });
        if back.clicked() {
            self.back();
        }
        let has_parent = self.parent().is_some();
        let up = navigation_icon(ui, has_parent, true)
            .on_disabled_hover_text("Up (Esc or Alt+Up)")
            .on_hover_text("Up (Esc or Alt+Up)");
        up.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, up.enabled(), "Up"));
        if up.clicked() {
            self.up();
        }
        chrome_divider(ui);
        ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| self.breadcrumbs(ui));
        });
    }

    fn breadcrumbs(&mut self, ui: &mut Ui) {
        let breadcrumb_location = self.current;
        let mut crumbs = vec![breadcrumb_location.system()];
        while let Some((parent, _)) = self.model.systems[*crumbs.last().unwrap()].parent {
            crumbs.push(parent);
        }
        for system in crumbs.into_iter().rev() {
            let target = Location::System(system);
            if target == self.current {
                ui.add(
                    egui::Label::new(RichText::new(&self.model.systems[system].name).strong())
                        .truncate(),
                );
            } else {
                let name = &self.model.systems[system].name;
                if ui
                    .add(egui::Button::new(name).frame(false).truncate())
                    .on_hover_text(name)
                    .clicked()
                {
                    self.navigate(target);
                    break;
                }
                ui.weak("›");
            }
        }
        if let Location::Chart { owner, block } = self.current {
            ui.add(
                egui::Label::new(
                    RichText::new(&self.model.systems[owner].blocks[block].name).strong(),
                )
                .truncate(),
            );
        }
    }

    fn tree(&mut self, ui: &mut Ui, system: SystemId) -> Option<Location> {
        let node = &self.model.systems[system];
        let children: Vec<_> = node
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(index, block)| {
                if let Some(target) = self.block_target(system, index) {
                    Some((Some(target), block.name.clone()))
                } else if matches!(block.block_type.as_str(), "SubSystem" | "ModelReference") {
                    Some((None, block.name.clone()))
                } else {
                    None
                }
            })
            .collect();
        let mut result = None;
        ui.horizontal(|ui| {
            if children.is_empty() {
                ui.add_space(22.0);
            } else {
                let icon = if self.expanded.contains(&system) {
                    "▾"
                } else {
                    "▸"
                };
                if ui
                    .add_sized([16.0, 25.0], egui::Button::new(icon).frame(false))
                    .on_hover_text("Expand or collapse")
                    .clicked()
                    && !self.expanded.remove(&system)
                {
                    self.expanded.insert(system);
                }
            }
            let selected = Location::System(system) == self.current;
            let label = ui
                .add_sized(
                    [ui.available_width(), 27.0],
                    egui::Button::selectable(selected, "")
                        .left_text(node.name.as_str())
                        .truncate(),
                )
                .on_hover_text(&node.name);
            if self.reveal_tree_selection && selected {
                label.scroll_to_me(None);
            }
            if label.clicked() {
                result = Some(Location::System(system));
            }
        });
        if !children.is_empty() && self.expanded.contains(&system) {
            ui.indent(system, |ui| {
                for (child, name) in children {
                    match child {
                        Some(Location::System(child)) => result = self.tree(ui, child).or(result),
                        Some(target @ Location::Chart { .. }) => {
                            ui.horizontal(|ui| {
                                ui.add_space(22.0);
                                let selected = self.current == target;
                                let label = ui
                                    .selectable_label(selected, name)
                                    .on_hover_text("Stateflow chart");
                                if selected && self.reveal_tree_selection {
                                    label.scroll_to_me(None);
                                }
                                if label.clicked() {
                                    result = Some(target);
                                }
                            });
                        }
                        None => {
                            ui.add_enabled(false, egui::Button::new(name).frame(false))
                                .on_disabled_hover_text("No saved child diagram is available");
                        }
                    }
                }
            });
        }
        result
    }

    fn properties(&self, ui: &mut Ui) {
        let selected = self.views.get(&self.current).and_then(|view| view.selected);
        if let Location::Chart { owner, block } = self.current {
            self.chart_properties(ui, self.chart_id(owner, block), selected);
            return;
        }
        let system = &self.model.systems[self.current.system()];
        let Some(block) = selected.map(|index| &system.blocks[index]) else {
            ui.weak("Select a block to inspect its stored properties.");
            return;
        };
        ScrollArea::vertical().show(ui, |ui| {
            ui.label(RichText::new(&block.name).size(16.0).strong());
            ui.weak(&block.block_type);
            ui.separator();
            ui.strong("General");
            ui.horizontal(|ui| {
                ui.weak("SID");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.monospace(&block.sid);
                });
            });
            if !block.properties.is_empty() {
                ui.separator();
                ui.strong("Parameters");
                for (name, value) in &block.properties {
                    ui.weak(name);
                    ui.label(RichText::new(value).monospace().small());
                    ui.add_space(3.0);
                }
            }
        });
    }

    fn workspace(&mut self, ui: &mut Ui) {
        let (name, kind) = match self.current {
            Location::System(id) => (
                &self.model.systems[id].name,
                if id == self.model.root {
                    "Block diagram"
                } else {
                    "Subsystem diagram"
                },
            ),
            Location::Chart { owner, block } => (
                &self.model.systems[owner].blocks[block].name,
                "Stateflow chart",
            ),
        };
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(20, 10))
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(name).heading().strong()).truncate());
                ui.label(RichText::new(kind).small().color(theme::WIRE));
            });
        let space = ui.available_size();
        ui.allocate_ui_with_layout(
            vec2(space.x, (space.y - 28.0).max(1.0)),
            Layout::top_down(Align::Min),
            |ui| {
                self.canvas(ui);
            },
        );
        ui.horizontal(|ui| {
            ui.add_space(20.0);
            ui.add(
                egui::Label::new(
                    RichText::new("Drag to pan · Ctrl + scroll to zoom")
                        .small()
                        .color(theme::WIRE),
                )
                .truncate(),
            );
        });
    }

    fn chart_properties(&self, ui: &mut Ui, id: ChartId, selected: Option<usize>) {
        let chart = &self.model.charts[id];
        ui.small(format!("Stateflow chart {}", chart.source_id));
        let Some(selected) = selected else {
            ui.label("Select a state, junction, function, or transition.");
            return;
        };
        ScrollArea::vertical().show(ui, |ui| {
            let (kind, ssid, label, properties) = if let Some(node) = chart.nodes.get(selected) {
                let kind = match node.kind {
                    ChartNodeKind::State => "State",
                    ChartNodeKind::GraphicalFunction => "Graphical function",
                    ChartNodeKind::Junction => "Junction",
                };
                (kind, &node.ssid, &node.label, &node.properties)
            } else {
                let transition = &chart.transitions[selected - chart.nodes.len()];
                (
                    "Transition",
                    &transition.ssid,
                    &transition.label,
                    &transition.properties,
                )
            };
            ui.strong(kind);
            ui.small(format!("SSID {ssid}"));
            if let Some(label) = label {
                ui.label(label);
            }
            for (name, value) in properties.iter().filter(|(name, _)| *name != "labelString") {
                ui.separator();
                ui.strong(name);
                ui.label(value);
            }
        });
    }

    fn canvas(&mut self, ui: &mut Ui) {
        match self.current {
            Location::System(id) => {
                let system = &self.model.systems[id];
                let scene = self.scenes.entry(id).or_insert_with(|| Scene::new(system));
                let view = self.views.entry(self.current).or_default();
                let enter = canvas::show(ui, system, scene, view);
                if let Some(target) = enter.and_then(|block| self.block_target(id, block)) {
                    self.navigate(target);
                    ui.ctx().request_repaint();
                }
            }
            Location::Chart { owner, block } => {
                let id = self.chart_id(owner, block);
                let chart = &self.model.charts[id];
                let scene = self.chart_scenes.entry(id).or_insert_with(|| {
                    ui.ctx().request_repaint();
                    ChartScene::new(chart)
                });
                let view = self.views.entry(self.current).or_default();
                stateflow::show(ui, chart, scene, view);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Event, PointerButton, Pos2, RawInput, Rect, Shape, vec2};
    use slx_viewer::{Limits, load};

    fn test_app(ctx: &Context, directory: PathBuf) -> App {
        theme::apply(ctx);
        App {
            loader: Loader::new(ctx.clone()).unwrap(),
            tabs: Vec::new(),
            active_slot: 0,
            current_path: None,
            document: None,
            error: None,
            show_diagnostics: false,
            settings: Settings::at(directory),
            menus: Menus::default(),
            logo: logo_texture(ctx, &application_icon()),
            startup: None,
            backend: "OpenGL (glow)".into(),
        }
    }

    fn app_frame(
        ctx: &Context,
        app: &mut App,
        key: Option<(Key, Modifiers)>,
    ) -> (bool, Vec<String>) {
        let events = key
            .map(|(key, modifiers)| {
                [true, false]
                    .map(|pressed| Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers,
                    })
                    .to_vec()
            })
            .unwrap_or_default();
        app_frame_events(ctx, app, events)
    }

    fn app_frame_events(ctx: &Context, app: &mut App, events: Vec<Event>) -> (bool, Vec<String>) {
        let mut open = false;
        let mut output = ctx.run_ui(
            RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
                events,
                focused: true,
                ..Default::default()
            },
            |ui| {
                open |= app.contents(ui);
            },
        );
        fn texts(shape: &Shape, result: &mut Vec<String>) {
            match shape {
                Shape::Text(text) => result.push(text.galley.text().into()),
                Shape::Vec(shapes) => {
                    for shape in shapes {
                        texts(shape, result);
                    }
                }
                _ => {}
            }
        }
        let mut result = Vec::new();
        for shape in &output.shapes {
            texts(&shape.shape, &mut result);
        }
        output.textures_delta.clear();
        (open, result)
    }

    #[test]
    fn toolbar_search_reveals_an_object_and_preserves_its_stored_expression() {
        let ctx = Context::default();
        let directory = tempfile::tempdir().unwrap();
        let mut app = test_app(&ctx, directory.path().into());
        let model = load(
            "tests/fixtures/dc_motor_speed_control.slx",
            &Limits::default(),
        )
        .unwrap();
        app.document = Some(Document::new(model));

        app_frame(&ctx, &mut app, Some((Key::K, Modifiers::CTRL)));
        app_frame_events(&ctx, &mut app, vec![Event::Text("BackEMF".into())]);
        let (_, texts) = app_frame(&ctx, &mut app, None);
        assert!(
            texts.iter().any(|text| text.starts_with("BackEMF  ·")),
            "{texts:?}"
        );

        app_frame(&ctx, &mut app, Some((Key::Enter, Modifiers::NONE)));
        let (_, texts) = app_frame(&ctx, &mut app, None);
        let doc = app.document.as_ref().unwrap();
        let selection = doc.views[&doc.current].selected.unwrap();
        assert_eq!(
            doc.model.systems[doc.current.system()].blocks[selection].name,
            "BackEMF"
        );
        assert!(texts.iter().any(|text| text.contains("p.Ke")), "{texts:?}");
    }

    #[test]
    fn startup_displays_support_then_reveals_the_loaded_document() {
        let ctx = Context::default();
        let directory = tempfile::tempdir().unwrap();
        let mut app = test_app(&ctx, directory.path().into());
        let model = load(
            "tests/fixtures/dc_motor_speed_control.slx",
            &Limits::default(),
        )
        .unwrap();
        app.document = Some(Document::new(model));
        app.startup = Some(Startup::default());

        let (_, texts) = app_frame(&ctx, &mut app, None);
        for label in [
            "Buy us a coffee",
            "Inspect saved SLX diagrams, signals, and block properties",
            "Open-source project · MIT License · Read-only",
            "© 2026 Lambda Technologies Pty Ltd",
        ] {
            assert!(texts.iter().any(|text| text == label), "{label}");
        }
        assert!(!texts.iter().any(|text| text == "Model hierarchy"));

        app.startup.as_mut().unwrap().first_frame = Some(Instant::now() - startup::DURATION);
        let (_, texts) = app_frame(&ctx, &mut app, None);
        assert!(app.startup.is_none());
        assert!(texts.iter().any(|text| text == "Model hierarchy"));
        assert!(texts.iter().any(|text| text == "dc_motor_speed_control"));
    }

    #[test]
    fn tab_selection_preserves_order_and_repeated_selection() {
        let ctx = Context::default();
        let directory = tempfile::tempdir().unwrap();
        let mut app = test_app(&ctx, directory.path().into());
        let document = || {
            Document::new(
                load(
                    "tests/fixtures/dc_motor_speed_control.slx",
                    &Limits::default(),
                )
                .unwrap(),
            )
        };
        app.tabs = vec![
            SavedTab {
                path: "A.slx".into(),
                document: document(),
            },
            SavedTab {
                path: "B.slx".into(),
                document: document(),
            },
        ];
        app.current_path = Some("C.slx".into());
        app.document = Some(document());
        app.active_slot = 2;
        for (index, slot, name) in [
            (0, 0, "A.slx"),
            (0, 1, "B.slx"),
            (1, 2, "C.slx"),
            (0, 0, "A.slx"),
        ] {
            app.switch_tab(index);
            assert_eq!(app.active_slot, slot);
            assert_eq!(app.current_path.as_ref().unwrap(), &PathBuf::from(name));
            let mut order: Vec<_> = app.tabs.iter().map(|tab| tab.path.clone()).collect();
            order.insert(app.active_slot, app.current_path.clone().unwrap());
            assert_eq!(order, ["A.slx", "B.slx", "C.slx"].map(PathBuf::from));
        }
        app.open("A.slx".into());
        assert_eq!(app.active_slot, 0);
    }

    #[test]
    fn bottom_right_zoom_controls_survive_resize_and_long_status_messages() {
        let ctx = Context::default();
        let directory = tempfile::tempdir().unwrap();
        let mut app = test_app(&ctx, directory.path().into());
        let mut time = 0.0;
        let mut frame = |app: &mut App, size, click: Option<Pos2>| {
            time += 1.0;
            let mut labels = Vec::new();
            let events = click
                .map(|position| {
                    let mut events = vec![Event::PointerMoved(position)];
                    events.extend([true, false].map(|pressed| Event::PointerButton {
                        pos: position,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: Modifiers::NONE,
                    }));
                    events
                })
                .unwrap_or_default();
            let mut output = ctx.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
                    time: Some(time),
                    events,
                    focused: true,
                    ..Default::default()
                },
                |ui| {
                    app.contents(ui);
                },
            );
            fn collect(shape: &Shape, labels: &mut Vec<(String, Rect)>) {
                match shape {
                    Shape::Text(text) => labels.push((
                        text.galley.text().into(),
                        Rect::from_min_size(text.pos, text.galley.size()),
                    )),
                    Shape::Vec(shapes) => {
                        for shape in shapes {
                            collect(shape, labels);
                        }
                    }
                    _ => {}
                }
            }
            for shape in &output.shapes {
                collect(&shape.shape, &mut labels);
            }
            output.textures_delta.clear();
            labels
        };
        let label = |labels: &[(String, Rect)], name: &str| {
            labels
                .iter()
                .find(|(text, _)| text == name)
                .unwrap_or_else(|| panic!("missing {name}: {labels:?}"))
                .1
        };
        let wide = vec2(1200.0, 800.0);
        let empty = frame(&mut app, wide, None);
        frame(&mut app, wide, Some(label(&empty, "+").center()));
        assert!(app.document.is_none());
        let model = load(
            "tests/fixtures/dc_motor_speed_control.slx",
            &Limits::default(),
        )
        .unwrap();
        app.document = Some(Document::new(model));
        frame(&mut app, wide, None);
        for size in [wide, vec2(740.0, 460.0), vec2(1000.0, 700.0)] {
            app.loader
                .open(PathBuf::from(format!(
                    "C:/{}.slx",
                    "long status path ".repeat(100)
                )))
                .unwrap();
            app.error =
                Some("A long error message that must leave the zoom controls visible. ".repeat(6));
            let labels = frame(&mut app, size, None);
            let fit = label(&labels, "Fit");
            let plus = label(&labels, "+");
            let minus = label(&labels, "−");
            let percent = labels
                .iter()
                .find(|(text, _)| text.ends_with('%'))
                .unwrap()
                .1;
            for bounds in [fit, minus, percent, plus] {
                assert!(Rect::from_min_size(Pos2::ZERO, size).contains_rect(bounds));
                assert!(bounds.min.y > size.y - 40.0, "{bounds:?}");
                assert!((bounds.center().y - plus.center().y).abs() < 2.0);
            }
            assert!(
                fit.max.x < minus.min.x
                    && minus.max.x < percent.min.x
                    && percent.max.x < plus.min.x
            );
            assert!(plus.max.x > size.x - 25.0);
            let location = app.document.as_ref().unwrap().current;
            let previous = app.document.as_ref().unwrap().views[&location].percent();
            frame(&mut app, size, Some(plus.center()));
            assert!(app.document.as_ref().unwrap().views[&location].percent() > previous);
            frame(&mut app, size, Some(percent.center()));
            assert_eq!(
                app.document.as_ref().unwrap().views[&location].percent(),
                100.0
            );
            frame(&mut app, size, Some(fit.center()));
            assert_ne!(
                app.document.as_ref().unwrap().views[&location].percent(),
                100.0
            );
            app.loader.cancel();
        }
    }

    #[test]
    fn menus_and_help_consume_escape_before_diagram_navigation() {
        let ctx = Context::default();
        let directory = tempfile::tempdir().unwrap();
        let mut app = test_app(&ctx, directory.path().into());
        let mut model = load(
            "tests/fixtures/dc_motor_speed_control.slx",
            &Limits::default(),
        )
        .unwrap();
        let mut child = load(
            "tests/fixtures/dc_motor_speed_control.slx",
            &Limits::default(),
        )
        .unwrap()
        .systems
        .remove(0);
        child.parent = Some((model.root, 0));
        model.systems.push(child);
        let mut document = Document::new(model);
        document.navigate(Location::System(1));
        app.document = Some(document);

        app_frame(&ctx, &mut app, Some((Key::F, Modifiers::ALT)));
        app_frame(&ctx, &mut app, Some((Key::Escape, Modifiers::NONE)));
        assert_eq!(app.document.as_ref().unwrap().current, Location::System(1));
        app_frame(&ctx, &mut app, Some((Key::H, Modifiers::ALT)));
        app_frame(&ctx, &mut app, Some((Key::Enter, Modifiers::NONE)));
        let (_, texts) = app_frame(&ctx, &mut app, None);
        for value in [
            "About SLX Viewer",
            "Buy us a coffee",
            env!("CARGO_PKG_VERSION"),
            env!("SLX_BUILD_UTC"),
            env!("SLX_BUILD_TARGET"),
            "OpenGL (glow)",
        ] {
            assert!(texts.iter().any(|text| text == value), "{value}");
        }
        app_frame(&ctx, &mut app, Some((Key::Escape, Modifiers::NONE)));
        assert_eq!(app.document.as_ref().unwrap().current, Location::System(1));
        app_frame(&ctx, &mut app, Some((Key::H, Modifiers::ALT)));
        app_frame(&ctx, &mut app, Some((Key::ArrowDown, Modifiers::NONE)));
        app_frame(&ctx, &mut app, Some((Key::Enter, Modifiers::NONE)));
        let (_, texts) = app_frame(&ctx, &mut app, None);
        assert!(texts.iter().any(|text| text.contains("MIT License")));
        assert!(
            texts
                .iter()
                .any(|text| text.contains("Lambda Technologies Pty Ltd"))
        );
        assert!(
            texts
                .iter()
                .any(|text| text.contains("Third-party licenses"))
        );
        app_frame(&ctx, &mut app, Some((Key::Escape, Modifiers::NONE)));
        assert_eq!(app.document.as_ref().unwrap().current, Location::System(1));
        app_frame(&ctx, &mut app, Some((Key::Escape, Modifiers::NONE)));
        assert_eq!(app.document.as_ref().unwrap().current, Location::System(0));
        assert!(app_frame(&ctx, &mut app, Some((Key::O, Modifiers::CTRL))).0);
        assert!(ctx.global_style().visuals.dark_mode);
    }

    #[test]
    fn only_successful_current_loads_enter_the_recent_list() {
        let ctx = Context::default();
        let directory = tempfile::tempdir().unwrap();
        let mut app = test_app(&ctx, directory.path().into());
        let wait_for_load = |app: &mut App| {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
            while app.loader.pending_path().is_some() {
                assert!(
                    std::time::Instant::now() < deadline,
                    "loader did not finish"
                );
                app.receive_load(&ctx);
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        app.open("tests/fixtures/dc_motor_speed_control.slx".into());
        wait_for_load(&mut app);
        assert!(app.error.is_none());
        assert!(app.settings.warning.is_none());
        let recent = app.settings.preferences.recent.clone();
        assert_eq!(recent.len(), 1);
        app.open(directory.path().join("unavailable.slx"));
        wait_for_load(&mut app);
        assert!(app.error.is_some());
        assert_eq!(app.settings.preferences.recent, recent);
        assert_eq!(
            app.document.as_ref().unwrap().model.name,
            "dc_motor_speed_control"
        );
        app.open("tests/fixtures/dc_motor_speed_control.slx".into());
        app.loader.cancel();
        app.receive_load(&ctx);
        assert_eq!(app.settings.preferences.recent, recent);
    }

    #[test]
    fn up_follows_hierarchy_and_back_restores_the_previous_view() {
        let fixture = "tests/fixtures/dc_motor_speed_control.slx";
        let mut model = load(fixture, &Limits::default()).unwrap();
        let mut controller = load(fixture, &Limits::default()).unwrap().systems.remove(0);
        let mut filter = load(fixture, &Limits::default()).unwrap().systems.remove(0);
        model.systems[0].blocks[0].child = Some(1);
        controller.parent = Some((0, 0));
        controller.blocks[1].child = Some(2);
        filter.parent = Some((1, 1));
        model.systems.extend([controller, filter]);

        let mut document = Document::new(model);
        document.up();
        assert!(document.history.is_empty());
        document.navigate(Location::System(2));
        document
            .views
            .entry(Location::System(2))
            .or_default()
            .selected = Some(3);
        assert!(document.expanded.contains(&1));
        document.up();
        assert_eq!(document.current, Location::System(1));
        assert_eq!(document.views[&Location::System(1)].selected, Some(1));
        assert_eq!(document.views[&Location::System(1)].reveal, Some(1));
        document.back();
        assert_eq!(document.current, Location::System(2));
        assert_eq!(document.views[&Location::System(2)].selected, Some(3));
        document.back();
        assert_eq!(document.current, Location::System(0));
    }
}

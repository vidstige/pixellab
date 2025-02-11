use egui::Vec2;
use tiny_skia::Pixmap;

use crate::nodes::node::{Node, Nodes, Pin};

enum PinValue {
    Float(f32),
    String(String),
    Pixmap(Pixmap),
}

pub struct PixelLab {
    nodes: Nodes<PinValue>,
}

impl Default for PixelLab {
    fn default() -> Self {
        Self {
            nodes: Nodes::new(),
        }
    }
}

impl PixelLab {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}
        let mut app: PixelLab = Default::default();

        let mut target = Node::new(PinValue::Float(2.2));
        target.rect = target.rect.translate(Vec2::new(120.0, 10.0));
        target.inputs.push(Pin::new());
        app.nodes.nodes.push(target);

        let mut node1 = Node::new(PinValue::Float(1.1));
        node1.outputs.push(Pin::new());
        app.nodes.nodes.push(node1);

        app
    }
}

// runs the pipeline
fn resolve(nodes: &Nodes<PinValue>, node_index: usize, pin_index: usize) -> PinValue{
    // 1. collect all input pins
    // 2. call this nodes callable
    PinValue::Float(9.9)
}

impl eframe::App for PixelLab {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        //eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.label("Timeline");
            egui::warn_if_debug_build(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pixel Labs");
            self.nodes.show(ui);
        });
    }
}

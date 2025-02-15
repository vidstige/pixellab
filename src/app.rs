use std::{iter::Sum, ops::Add};

use egui::{Color32, Sense, Stroke, Vec2, Widget};
use tiny_skia::Pixmap;

use crate::nodes::node::{Node, NodeWidget, Nodes, Pin};

// time stuff
struct Duration {
    millis: u32,
}
impl Duration {
    fn from_secs(seconds: f32) -> Duration {
        Self { millis: (1000.0 * seconds) as u32 }
    }
    fn from_millis(millis: u32) -> Duration {
        Self { millis, }
    }
    fn as_millis(&self) -> u32 { self.millis }
}
impl Add for &Duration {
    type Output = Duration;

    fn add(self, rhs: Self) -> Self::Output {
        Duration { millis: self.millis + rhs.millis }
    }
}
impl<'a> Sum<&'a Duration> for Duration {
    fn sum<I: Iterator<Item = &'a Duration>>(iter: I) -> Duration {
        Duration::from_millis(iter.map(|d| d.millis).sum())
    }
}
struct Instant {
    millis: u32,
}
impl Default for Instant {
    fn default() -> Self {
        Self { millis: Default::default() }
    }
}
impl Instant {
    fn zero() -> Self { Self { millis: 0, } }
}

/*enum PinValue {
    Float(f32),
    String(String),
    Pixmap(Pixmap),
}*/

#[derive(Clone)]
enum NodeType {
    Float(f32),
    String(String),
}

impl NodeWidget for NodeType {
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            NodeType::Float(value) => ui.add(egui::Slider::new(value, 0.0..=10.0)),
            _ => ui.response(),
        }
    }
}

struct VideoSettings {
    resolution: (usize, usize),
}

pub struct PixelLab {
    video_settings: VideoSettings,
    timeline: Timeline,
    nodes: Nodes<NodeType>,
}

impl Default for PixelLab {
    fn default() -> Self {
        let fps = 30.0;
        Self {
            video_settings: VideoSettings { resolution: (320, 200), },
            timeline: Timeline::new(fps),
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

        let mut target = Node::new(NodeType::Float(1.0));
        target.rect = target.rect.translate(Vec2::new(120.0, 10.0));
        target.inputs.push(Pin::new());
        app.nodes.nodes.push(target);

        let mut node1 = Node::new(NodeType::Float(1.1));
        node1.outputs.push(Pin::new());
        app.nodes.nodes.push(node1);

        // add some stuff on the timeline
        app.timeline.blocks.push(Duration::from_secs(3.0));
        //app.timeline.blocks.push(Duration::from_secs_f32(3.0));
        //app.timeline.blocks.push(Duration::from_secs_f32(3.0));

        app
    }
}

// runs the pipeline
/*fn resolve(nodes: &Nodes<NodeType>, node_index: usize, pin_index: usize) -> NodeType {
    // 1. collect all input pins
    // 2. call this nodes callable
    PinValue::Float(9.9)
}*/

struct Timeline {
    caret: Instant,
    fps: f32,
    blocks: Vec<Duration>,
}

impl Timeline {
    fn new(fps: f32) -> Self {
        Self { caret: Instant::zero(), fps, blocks: Vec::new(), }
    }
    fn duration(&self) -> Duration {
        self.blocks.iter().sum()
    }
}

impl Widget for &mut Timeline {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = Vec2::new(ui.available_width(), 100.0);
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::empty());
        let frame_duration = Duration::from_secs(1.0 / self.fps);
        let total_duration = self.duration();
        let frame_count = total_duration.as_millis() / frame_duration.as_millis();
        let painter = ui.painter();
        for frame_index in 0..frame_count {
            let x = rect.left() + rect.width() * frame_index as f32 / frame_count as f32;
            let y = rect.top()..=rect.top() + 0.5  *rect.height();
            painter.vline(x, y, Stroke::new(1.0, Color32::LIGHT_GRAY));
        }
        response
    }
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
            ui.add(&mut self.timeline);
            egui::warn_if_debug_build(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Pixel Labs");
            self.nodes.show(ctx, ui);
        });
    }
}

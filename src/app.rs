use std::{fmt, iter::Sum, ops::Add, sync::Arc};

use egui::{Color32, ColorImage, ImageData, Sense, Stroke, TextureHandle, TextureOptions, Vec2, Widget};
use tiny_skia::{Color, Pixmap};

use crate::nodes::node::{Graph, Node, NodeWidget, Pin};

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

enum PinValue {
    None,
    Float(f32),
    String(String),
    Color(Color),
    Pixmap(Pixmap),
}
impl PinValue {
    fn pixmap(self) -> Pixmap {
        if let PinValue::Pixmap(pixmap) = self {
            pixmap
        } else {
            panic!("Unexpected pin value")
        }
    }
    
    fn color(self) -> Option<Color> {
        if let PinValue::Color(color) = self {
            Some(color)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
enum NodeType {
    Float(f32),
    String(String),
    Color(Color32),
    Fill,
    Output,
}

impl NodeType {
    fn evaluate(&self, pin_values: Vec<PinValue>, pin_index: usize) -> PinValue {
        match self {
            NodeType::Float(value) => PinValue::Float(*value),
            NodeType::String(value) => PinValue::String(value.clone()),
            NodeType::Color(value) => PinValue::Color(Color::from_rgba8(
                value.r(), value.g(), value.b(), value.a())
            ),
            NodeType::Fill => {
                let color = pin_values.into_iter().next().unwrap_or(PinValue::None).color().unwrap_or(Color::TRANSPARENT);
                let mut pixmap = Pixmap::new(320, 200).unwrap();
                pixmap.fill(color);
                PinValue::Pixmap(pixmap)
            }
            NodeType::Output => pin_values.into_iter().next().unwrap_or(PinValue::None),
        }
    }
}

impl NodeWidget for NodeType {
    fn in_pins(&self) -> Vec<Pin> {
        match self {
            NodeType::Fill => [Pin::new()].into(),
            NodeType::Output => [Pin::new()].into(),
            _ => Vec::new(),
        }
    }
    fn out_pins(&self) -> Vec<Pin> {
        match self {
            NodeType::Float(_) => [Pin::new()].into(),
            NodeType::String(_) => [Pin::new()].into(),
            NodeType::Color(_) => [Pin::new()].into(),
            NodeType::Fill => [Pin::new()].into(),
            NodeType::Output => Vec::new(),
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            NodeType::Float(value) => ui.add(egui::Slider::new(value, 0.0..=10.0)),
            NodeType::Color(value) => {
                egui::color_picker::color_picker_color32(ui, value, egui::color_picker::Alpha::Opaque);
                ui.response()
            },
            _ => ui.response(),
        }
    }
}

struct VideoSettings {
    resolution: [usize; 2],
}

pub struct PixelLab {
    video_settings: VideoSettings,
    output_texture: TextureHandle,
    timeline: Timeline,
    graph: Graph<NodeType>,
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
        let fps = 30.0;
        let resolution = [320, 200];
        let output_texture = cc.egui_ctx.load_texture(
            "output",
            ImageData::Color(Arc::new(ColorImage::new(resolution, Color32::TRANSPARENT))),
            TextureOptions::default(),
        );
        let mut app = PixelLab {
            video_settings: VideoSettings { resolution, },
            output_texture,
            timeline: Timeline::new(fps),
            graph: Graph::new(),
        };

        app.graph.nodes.push(Node::new(NodeType::Output));
        app.graph.nodes.push(Node::new(NodeType::Color(Color32::GRAY)));
        app.graph.nodes.push(Node::new(NodeType::Fill));

        // add some stuff on the timeline
        app.timeline.blocks.push(Duration::from_secs(3.0));
        //app.timeline.blocks.push(Duration::from_secs_f32(3.0));
        //app.timeline.blocks.push(Duration::from_secs_f32(3.0));

        app
    }
}


// runs the pipeline
fn resolve(nodes: &Graph<NodeType>, node_index: usize, pin_index: usize) -> PinValue {
    // 1. collect all input pins
    let input_pins = nodes.inputs_for(node_index);
    // 2. resolve respective output pins
    let input_values: Vec<_> = input_pins
        .iter()
        .map(|pin_id| resolve(nodes, pin_id.node_index, pin_id.pin_index))
        .collect();
    // 3. call this nodes callable
    nodes.nodes[node_index].widget.evaluate(input_values, pin_index)
}

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
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::drag());

        let frame_duration = Duration::from_secs(1.0 / self.fps);
        let total_duration = self.duration();
        let frame_count = total_duration.as_millis() / frame_duration.as_millis();
        let painter = ui.painter();
        for frame_index in 0..frame_count {
            let x = rect.left() + rect.width() * frame_index as f32 / frame_count as f32;
            let y = rect.top()..=rect.top() + 0.5  *rect.height();
            painter.vline(x, y, Stroke::new(1.0, Color32::DARK_GRAY));
        }
        // handle caret drag
        if let Some(pointer) = response.interact_pointer_pos() {
            self.caret.millis = (total_duration.as_millis() as f32 * pointer.x / rect.width()) as u32;
        }
        // draw caret
        let x = rect.left() + self.caret.millis as f32 * rect.width() / total_duration.as_millis() as f32;
        painter.vline(x, rect.bottom_up_range(), Stroke::new(1.0, Color32::LIGHT_GRAY));

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
            // node editor
            self.graph.show(ctx, ui);

            // output window
            // evaluate pixmap
            if let PinValue::Pixmap(pixmap) = resolve(&self.graph, 0, 0) {
                self.output_texture.set(
                    ColorImage::from_rgba_premultiplied(
                        [pixmap.width() as usize, pixmap.height() as usize],
                        pixmap.data(),
                    ),
                    TextureOptions::default(),
                );
            }
            egui::Window::new("Output").show(ctx, |ui| {
                ui.add(egui::Image::from_texture(&self.output_texture));
            });
        });
    }
}

use std::{f32::consts::TAU, path::PathBuf, sync::Arc};

use egui::{Color32, ColorImage, ImageData, Sense, Stroke, TextureHandle, TextureOptions, Vec2, Widget};
use json::JsonValue;
use tiny_skia::{Color, Pixmap, PremultipliedColorU8, Transform};

use crate::{fields::{ConstantField, Field2}, hex::{draw_hex_grid, HexGrid}, nodes::node::{Graph, NodeWidget, Pin, PinDirection, PinId}, time::{Duration, Instant}};

impl Field2<Color> for Pixmap {
    fn at(&self, position: tiny_skia::Point) -> Color {
        let x = position.x as u32;
        let y = position.y as u32;
        
        let color = self.pixel(x, y).unwrap_or(PremultipliedColorU8::TRANSPARENT).demultiply();
        Color::from_rgba8(color.red(), color.green(), color.blue(), color.alpha())
    }
}

//#[derive(Debug)]
enum PinValue {
    None,
    Float(f32),
    String(String),
    Color(Color),
    Transform(Transform),
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
    // try to convert value into a color field
    fn as_color_field(self) -> Option<Box<dyn Field2<Color>>> {
        match self {
            PinValue::Color(color) => Some(Box::new(ConstantField::new(color))),
            PinValue::Pixmap(pixmap) => Some(Box::new(pixmap)),
            _ => None,
        }
    }
    fn color(self) -> Option<Color> {
        if let PinValue::Color(color) = self { Some(color) } else { None }
    }
    fn f32(self) -> Option<f32> {
        if let PinValue::Float(value) = self { Some(value) } else { None }
    }
    fn transform(self) -> Option<Transform> {
        if let PinValue::Transform(value) = self { Some(value) } else { None }
    }
}

#[derive(Clone, Debug)]
enum NodeType {
    Time,
    Float(f32),
    String(String),
    Color(Color32),
    Pixmap(PathBuf),
    Revolution,
    Rotate,
    Hex,
    Fill,
    Output,
}

impl NodeType {
    fn evaluate(&self, pin_values: Vec<PinValue>, pin_index: usize, t: f32) -> PinValue {
        match self {
            NodeType::Time => PinValue::Float(t),
            NodeType::Float(value) => PinValue::Float(*value),
            NodeType::String(value) => PinValue::String(value.clone()),
            NodeType::Color(value) => PinValue::Color(Color::from_rgba8(
                value.r(), value.g(), value.b(), value.a())
            ),
            NodeType::Pixmap(path) => PinValue::Pixmap(Pixmap::load_png(path.as_path()).unwrap()),
            NodeType::Revolution => {
                let value = pin_values.into_iter().next().unwrap_or(PinValue::None).f32().unwrap_or(0.0);
                PinValue::Float(TAU * value)
            }
            NodeType::Rotate => {
                let angle = pin_values.into_iter().next().unwrap_or(PinValue::None).f32().unwrap_or(0.0);
                PinValue::Transform(Transform::post_rotate(&Transform::identity(), angle.to_degrees()))
            },
            NodeType::Hex => {
                // extract inputs
                let mut pins = pin_values.into_iter();
                let color = pins.next().unwrap_or(PinValue::None).as_color_field().unwrap_or(Box::new(ConstantField::new(Color::TRANSPARENT)));
                let spacing = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(8.0);
                let size = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(8.0);
                let transform = pins.next().unwrap_or(PinValue::None).transform().unwrap_or(Transform::identity());
                
                let mut pixmap = Pixmap::new(320, 200).unwrap();
                let grid = HexGrid::new(spacing, size, transform.post_translate(160.0, 120.0));
                
                draw_hex_grid(&mut pixmap, &grid, color.as_ref());
                PinValue::Pixmap(pixmap)
            },
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
            NodeType::Revolution => [Pin::new()].into(),
            NodeType::Rotate => [Pin::new()].into(),
            NodeType::Hex => [Pin::new(), Pin::new(), Pin::new(), Pin::new()].into(),
            NodeType::Fill => [Pin::new()].into(),
            NodeType::Output => [Pin::new()].into(),
            _ => Vec::new(),
        }
    }
    fn out_pins(&self) -> Vec<Pin> {
        match self {
            NodeType::Time => [Pin::new()].into(),
            NodeType::Float(_) => [Pin::new()].into(),
            NodeType::String(_) => [Pin::new()].into(),
            NodeType::Color(_) => [Pin::new()].into(),
            NodeType::Pixmap(_) => [Pin::new()].into(),
            NodeType::Revolution => [Pin::new()].into(),
            NodeType::Rotate => [Pin::new()].into(),
            NodeType::Hex => [Pin::new()].into(),
            NodeType::Fill => [Pin::new()].into(),
            NodeType::Output => Vec::new(),
        }
    }
    fn title(&self) -> String {
        match self {
            NodeType::Time => "time",
            NodeType::Float(_) => "float",
            NodeType::String(_) => "text",
            NodeType::Color(_) => "color",
            NodeType::Pixmap(_) => "pixmap",
            NodeType::Revolution => "revolution",
            NodeType::Rotate => "rotate",
            NodeType::Hex => "hex",
            NodeType::Fill => "fill",
            NodeType::Output => "output",
        }.into()
    }
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            NodeType::Float(value) => ui.add(egui::Slider::new(value, 2.0..=16.0)),
            NodeType::Color(value) => {
                egui::color_picker::color_picker_color32(ui, value, egui::color_picker::Alpha::Opaque);
                ui.response()
            },
            NodeType::Pixmap(path) => {
                let mut text = path.to_str().unwrap_or("").to_string();
                let response = ui.text_edit_singleline(&mut text);
                *path = text.into();
                response
            },
            _ => ui.response(),
        }
    }
}

fn into_node(raw: &json::JsonValue) -> Option<NodeType> {
    let node_type_raw = raw["type"].as_str().unwrap();
    match node_type_raw {
        "time" => Some(NodeType::Time),
        "float" => raw["value"].as_f32().map(|value| NodeType::Float(value)),
        "string" => raw["value"].as_str().map(|value| NodeType::String(value.to_string())),
        "color" => raw["value"].as_str().map(|value| Color32::from_hex(value).ok().map(|value| NodeType::Color(value)))?,
        "pixmap" => raw["path"].as_str().map(|value| NodeType::Pixmap(value.into())),
        "revolution" => Some(NodeType::Revolution),
        "rotate" => Some(NodeType::Rotate),
        "hex" => Some(NodeType::Hex),
        "fill" => Some(NodeType::Fill),
        "output" => Some(NodeType::Output),
        _ => None
    }
}

fn into_pinid(raw: &json::JsonValue, direction: PinDirection) -> PinId {
    PinId {
        node_index: raw["node"].as_usize().unwrap(),
        pin_index: raw["pin"].as_usize().unwrap(),
        direction,
    }
}
fn into_link(raw: &json::JsonValue) -> Option<(PinId, PinId)> {
    Some((into_pinid(&raw["from"], PinDirection::Output), into_pinid(&raw["to"], PinDirection::Input)))
}

// graph io
fn load_graph(raw: &str) -> Result<Graph<NodeType>, json::Error> {
    let root = json::parse(raw)?;
    let nodes = root["nodes"].members().filter_map(|raw| into_node(&raw)).collect();
    let links = root["links"].members().filter_map(|raw| into_link(raw)).collect();
    Ok(Graph { nodes, links})
}

fn from_nodetype(node_type: NodeType) -> json::JsonValue {
    match node_type {
        NodeType::Time => json::object!{"type": "time"},
        NodeType::Float(value) => json::object!{"type": "float", value: value},
        NodeType::String(value) => json::object!{"type": "string", value: value},
        NodeType::Color(value) => json::object!{"type": "color", value: value.to_hex()},
        NodeType::Pixmap(path) => json::object!{"type": "pixmap", path: path.to_str()},
        NodeType::Revolution => json::object!{"type": "revolution"},
        NodeType::Rotate => json::object!{"type": "rotate"},
        NodeType::Hex => json::object!{"type": "hex"},
        NodeType::Fill => json::object!{"type": "fill"},
        NodeType::Output => json::object!{"type": "output"},
    }
}

fn save_graph(graph: &Graph<NodeType>) -> Result<String, json::JsonError> {
    let mut root = json::JsonValue::new_object();
    root["nodes"] = JsonValue::new_array();
    for node in &graph.nodes {
        root["nodes"].push(from_nodetype(node.clone()))?;
    }

    root["links"] = JsonValue::new_array();
    for (from, to) in &graph.links {
        root["links"].push(
            json::object!{
                from: json::object!{node: from.node_index, pin: from.pin_index},
                to: json::object!{node: to.node_index, pin: to.pin_index},
            }
        )?;
    }
    Ok(root.dump())
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

        let mut graph = Graph::new();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            //return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            if let Some(raw) = storage.get_string("graph_json") {
                println!("{}", raw);
                graph = load_graph(&raw).unwrap();
            }
        }

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
            graph,
        };

        // add some stuff on the timeline
        app.timeline.blocks.push(Duration::from_secs(3.0));
        //app.timeline.blocks.push(Duration::from_secs_f32(3.0));
        //app.timeline.blocks.push(Duration::from_secs_f32(3.0));

        app
    }
    fn add_node(&mut self, node: NodeType) {
        self.graph.nodes.push(node);
    }
}


// runs the pipeline
fn resolve(nodes: &Graph<NodeType>, node_index: usize, pin_index: usize, t: f32) -> PinValue {
    // 1. collect all input pins
    let input_pins = nodes.inputs_for(node_index);
    // 2. resolve respective output pins
    let input_values: Vec<_> = input_pins
        .iter()
        .map(|pin_id| resolve(nodes, pin_id.node_index, pin_id.pin_index, t))
        .collect();
    // 3. call this nodes callable
    nodes.nodes[node_index].evaluate(input_values, pin_index, t)
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
        if let Ok(raw) = save_graph(&self.graph) {
            storage.set_string("graph_json", raw);
        } else {
            println!("could not save graph");
        }
        //storage.set_string(eframe::APP_KEY, value);
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
            let response = self.graph.show(ctx, ui);
            response.context_menu(|ui| {
                if ui.button("float").clicked() {
                    self.add_node(NodeType::Float(1.0));
                }
                if ui.button("color").clicked() {
                    self.add_node(NodeType::Color(Color32::GRAY));
                }
                if ui.button("rotate").clicked() {
                    self.add_node(NodeType::Rotate);
                }
                if ui.button("revolution").clicked() {
                    self.add_node(NodeType::Revolution);
                }
                if ui.button("time").clicked() {
                    self.add_node(NodeType::Time);
                }
                if ui.button("hex").clicked() {
                    self.add_node(NodeType::Hex);
                }
                if ui.button("pixmap").clicked() {
                    self.add_node(NodeType::Pixmap(PathBuf::new()));
                }
                if ui.button("fill").clicked() {
                    self.add_node(NodeType::Fill);
                }
            });
    

            // output window
            // evaluate pixmap
            // compute global time
            let t = self.timeline.caret.millis as f32 / self.timeline.duration().as_millis() as f32;
            if let PinValue::Pixmap(pixmap) = resolve(&self.graph, 0, 0, t) {
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

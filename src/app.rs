use std::{f32::consts::TAU, path::PathBuf, sync::Arc};

use egui::{Color32, ColorImage, ImageData, Layout, Response, Sense, Stroke, TextureHandle, TextureOptions, Ui, Vec2, Widget};
use json::JsonValue;
use tiny_skia::{Color, Pixmap, PremultipliedColorU8, Transform};

use crate::{fields::{ConstantField, Field2}, hex::{draw_hex_grid, HexGrid}, nodes::node::{Graph, NodeWidget, Pin, PinDirection, PinId}, time::{Duration, Instant}};

impl Field2<Color> for Pixmap {
    fn at(&self, position: tiny_skia::Point) -> Color {
        let x = (position.x + 0.5 * self.width() as f32) as u32;
        let y = (position.y + 0.5 * self.height() as f32) as u32;
        
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
fn load_graph(root: &json::JsonValue) -> Result<Graph<NodeType>, json::Error> {
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

fn save_graph(graph: &Graph<NodeType>) -> Result<json::JsonValue, json::JsonError> {
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
    Ok(root)
}

fn save_timeline(timeline: &Timeline<Graph<NodeType>>) -> Result<json::JsonValue, json::JsonError> {
    let mut root = json::JsonValue::new_array();
    for (duration, graph) in &timeline.blocks {
        let graph_json = save_graph(graph)?;
        root.push(json::object!{
            duration: duration.as_millis(),
            graph: graph_json,
        })?;
    }
    Ok(root)
}

fn load_timeline(raw: &str) -> Result<Timeline<Graph<NodeType>>, json::Error> {
    let root = json::parse(raw)?;
    let mut timeline = Timeline::new(3.0);
    for block in root.members() {
        let duration = Duration::from_millis(block["duration"].as_u32().unwrap_or(3000));
        let graph = load_graph(&block["graph"])?;
        timeline.blocks.push((duration, graph));

    }
    Ok(timeline)
}

fn create_graph() -> Graph<NodeType> {
    let mut graph = Graph::new();
    graph.nodes.push(NodeType::Output);
    graph
}

struct VideoSettings {
    resolution: [usize; 2],
}

pub struct PixelLab {
    video_settings: VideoSettings,
    output_texture: TextureHandle,
    timeline: Timeline<Graph<NodeType>>,
}

impl PixelLab {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        let fps = 30.0;
        let mut timeline = Timeline::new(fps);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            //return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            if let Some(raw) = storage.get_string("timeline_json") {
                println!("{}", raw);
                timeline = load_timeline(&raw).unwrap();
            }
        }

        let resolution = [320, 200];
        let output_texture = cc.egui_ctx.load_texture(
            "output",
            ImageData::Color(Arc::new(ColorImage::new(resolution, Color32::TRANSPARENT))),
            TextureOptions::default(),
        );
        let mut app = PixelLab {
            video_settings: VideoSettings { resolution, },
            output_texture,
            timeline,
        };

        // add some stuff on the timeline, if empty
        if app.timeline.blocks.is_empty() {
            app.timeline.blocks.push((Duration::from_secs(3.0), create_graph()));
        }

        app
    }
    fn graph(&mut self) -> &mut Graph<NodeType> {
        let index = self.timeline.selected_index().unwrap();
        &mut self.timeline.blocks[index].1
    }
    fn add_node(&mut self, node: NodeType) {
        self.graph().nodes.push(node);
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

struct Timeline<T> {
    caret: Instant,
    fps: f32,
    blocks: Vec<(Duration, T)>,
}

impl<T> Timeline<T> {
    fn new(fps: f32) -> Self {
        Self { caret: Instant::zero(), fps, blocks: Vec::new(), }
    }
    fn duration(&self) -> Duration {
        self.blocks.iter().map(|(duration, _)| duration).sum()
    }
    fn selected_index(&self) -> Option<usize> {
        let mut start = Instant::zero();
        for (index, (duration, _)) in &mut self.blocks.iter().enumerate() {
            let end = start.after(duration);
            if self.caret.millis < end.millis {
                return Some(index);
            }
            start = end;
        }
        None
    }
    fn delete_selected(&mut self) {
        if let Some(index) = self.selected_index() {
            self.blocks.remove(index);
            // update caret
            if self.caret.millis > self.duration().millis {
                self.caret = Instant::zero().after(&Duration::from_millis(self.duration().millis - 1));
            }
        }
    }
    fn selected_mut(&mut self) -> Option<&mut (Duration, T)> {
        self.selected_index().map(|index| &mut self.blocks[index])
    }
    fn show_ticks(&mut self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(ui.available_width(), 25.0);
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::drag());

        let frame_duration = Duration::from_secs(1.0 / self.fps);
        let total_duration = self.duration();
        let frame_count = total_duration.as_millis() / frame_duration.as_millis();
        
        // draw ticks
        let painter = ui.painter();
        for frame_index in 0..frame_count {
            let x = rect.left() + rect.width() * frame_index as f32 / frame_count as f32;
            let y = rect.top()..=rect.top() + rect.height();
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

impl Widget for &mut Timeline<Graph<NodeType>> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            // can't delete the last block
            if self.blocks.len() > 1 && ui.button("delete").clicked() {
                self.delete_selected();
            }
            if ui.button("add").clicked() {
                let duration = Duration::from_secs(3.0);
                self.blocks.push((duration, create_graph()));
            }
            if let Some((duration, _)) = self.selected_mut() {
                ui.add(egui::Slider::new(&mut duration.millis, 1..=5000));
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                self.show_ticks(ui);
                // show blocks
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    let height = 50.0;
                    let total_width = ui.available_width();
                    let total_duration = self.duration();
                    for (duration, _) in &self.blocks {
                        let width = total_width * duration.as_millis() as f32 / total_duration.as_millis() as f32;
                        ui.group(|ui| {
                            ui.allocate_exact_size(Vec2::new(width, height), Sense::empty());
                        });
                    }
                });
            })
        })
        .response
    }
}

impl eframe::App for PixelLab {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(raw) = save_timeline(&self.timeline) {
            storage.set_string("timeline_json", raw.dump());
            //storage.set_string("graph_json", raw);
        } else {
            println!("could not save timeline");
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
            let response = self.graph().show(ctx, ui);
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
            if let PinValue::Pixmap(pixmap) = resolve(&self.graph(), 0, 0, t) {
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

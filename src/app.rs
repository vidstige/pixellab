use std::{f32::consts::TAU, path::PathBuf, sync::Arc};

use egui::{Color32, ColorImage, ImageData, Response, Sense, Stroke, TextureHandle, TextureOptions, Ui, Vec2, Widget};
use json::JsonValue;
use tiny_skia::{Color, Pixmap, PremultipliedColorU8, Transform};

use crate::{fields::{ConstantField, Field2}, hex::{draw_hex_grid, HexGrid}, nodes::node::{Graph, NodeWidget, Pin, PinDirection, PinId}, time::{Duration, Instant}, tweening};

impl Field2<Color> for Pixmap {
    fn at(&self, position: tiny_skia::Point) -> Color {
        let x = (position.x + 0.5 * self.width() as f32) as u32;
        let y = (position.y + 0.5 * self.height() as f32) as u32;
        
        let color = self.pixel(x, y).unwrap_or(PremultipliedColorU8::TRANSPARENT).demultiply();
        Color::from_rgba8(color.red(), color.green(), color.blue(), color.alpha())
    }
}

struct TransformedColorField {
    field: Box<dyn Field2<Color>>,
    transform: Transform,
}
impl Field2<Color> for TransformedColorField {
    fn at(&self, position: tiny_skia::Point) -> Color {
        let mut p = position;
        self.transform.map_point(&mut p);
        self.field.at(p)
    }
}

struct Lerp<T> {
    a: T,
    b: T,
}
impl Lerp<f32> {
    fn eval(&self, t: f32) -> f32 {
        self.a * (1.0 - t) + self.b * t
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
    ColorField(Box<dyn Field2<Color>>),
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
            PinValue::ColorField(field) => Some(field),
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
    // data types
    Time,
    Float(f32),
    String(String),
    Color(Color32),
    // tweens
    Lerp,
    Cubic(bool),
    // color fields
    Pixmap(PathBuf),
    TransformColorField,
    // transforms
    Revolution,
    Rotate,
    Scale,
    Hex,
    Output,
}

impl NodeType {
    fn evaluate(&self, pin_values: Vec<PinValue>, pin_index: usize, t: f32) -> PinValue {
        let mut pins = pin_values.into_iter();
        match self {
            NodeType::Time => PinValue::Float(t),
            NodeType::Float(value) => PinValue::Float(*value),
            NodeType::String(value) => PinValue::String(value.clone()),
            NodeType::Color(value) => PinValue::Color(Color::from_rgba8(
                value.r(), value.g(), value.b(), value.a())
            ),
            NodeType::Lerp => {
                // TODO: Handle colors, positions, etc
                let a = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(0.0);
                let b = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(1.0);
                let t = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(0.0);
                PinValue::Float(Lerp {a, b}.eval(t))
            },
            NodeType::Cubic(bool) => {
                let value = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(0.0);
                PinValue::Float(tweening::cubic_in(value))
            },
            NodeType::Pixmap(path) => PinValue::Pixmap(Pixmap::load_png(path.as_path()).unwrap()),
            NodeType::TransformColorField => {
                let color = pins.next().unwrap_or(PinValue::None).as_color_field().unwrap_or(Box::new(ConstantField::new(Color::TRANSPARENT)));
                let transform = pins.next().unwrap_or(PinValue::None).transform().unwrap_or(Transform::identity());
                PinValue::ColorField(Box::new(TransformedColorField { field: color, transform }))
            }
            NodeType::Revolution => {
                let value = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(0.0);
                PinValue::Float(TAU * value)
            }
            NodeType::Rotate => {
                let angle = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(0.0);
                PinValue::Transform(Transform::post_rotate(&Transform::identity(), angle.to_degrees()))
            },
            NodeType::Scale => {
                let sx = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(1.0);
                let sy = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(sx);
                PinValue::Transform(Transform::post_scale(&Transform::identity(), sx, sy))
            },
            NodeType::Hex => {
                // extract inputs
                let color = pins.next().unwrap_or(PinValue::None).as_color_field().unwrap_or(Box::new(ConstantField::new(Color::TRANSPARENT)));
                let spacing = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(8.0);
                let size = pins.next().unwrap_or(PinValue::None).f32().unwrap_or(8.0);
                let transform = pins.next().unwrap_or(PinValue::None).transform().unwrap_or(Transform::identity());
                
                let mut pixmap = Pixmap::new(320, 200).unwrap();
                let grid = HexGrid::new(spacing, size, transform.post_translate(160.0, 120.0));
                
                draw_hex_grid(&mut pixmap, &grid, color.as_ref());
                PinValue::Pixmap(pixmap)
            },
            NodeType::Output => pins.next().unwrap_or(PinValue::None),
        }
    }
}

impl NodeWidget for NodeType {
    fn in_pins(&self) -> Vec<Pin> {
        match self {
            NodeType::Lerp => [Pin::new(), Pin::new(), Pin::new()].into(),
            NodeType::Cubic(_) => [Pin::new()].into(),
            NodeType::Revolution => [Pin::new()].into(),
            NodeType::Rotate => [Pin::new()].into(),
            NodeType::Scale => [Pin::new(), Pin::new()].into(),
            NodeType::TransformColorField => [Pin::new(), Pin::new()].into(),
            NodeType::Hex => [Pin::new(), Pin::new(), Pin::new(), Pin::new()].into(),
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
            NodeType::Lerp => [Pin::new()].into(),
            NodeType::Cubic(_) => [Pin::new()].into(),
            NodeType::Pixmap(_) => [Pin::new()].into(),
            NodeType::TransformColorField => [Pin::new()].into(),
            NodeType::Revolution => [Pin::new()].into(),
            NodeType::Rotate => [Pin::new()].into(),
            NodeType::Scale => [Pin::new()].into(),
            NodeType::Hex => [Pin::new()].into(),
            NodeType::Output => Vec::new(),
        }
    }
    fn title(&self) -> String {
        match self {
            NodeType::Time => "time",
            NodeType::Float(_) => "float",
            NodeType::String(_) => "text",
            NodeType::Color(_) => "color",
            NodeType::Lerp => "lerp",
            NodeType::Cubic(_) => "cubic",
            NodeType::Pixmap(_) => "pixmap",
            NodeType::TransformColorField => "transform color field",
            NodeType::Revolution => "revolution",
            NodeType::Rotate => "rotate",
            NodeType::Scale => "scale",
            NodeType::Hex => "hex",
            NodeType::Output => "output",
        }.into()
    }
    fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        match self {
            NodeType::Float(value) => ui.add(egui::Slider::new(value, 0.0..=256.0).logarithmic(true)),
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
        "lerp" => Some(NodeType::Lerp),
        "cubic" =>  raw["in"].as_bool().map(|value| NodeType::Cubic(value.into())),
        "pixmap" => raw["path"].as_str().map(|value| NodeType::Pixmap(value.into())),
        "transform-color-field" => Some(NodeType::TransformColorField),
        "revolution" => Some(NodeType::Revolution),
        "rotate" => Some(NodeType::Rotate),
        "scale" => Some(NodeType::Scale),
        "hex" => Some(NodeType::Hex),
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
    let nodes: Vec<NodeType> = root["nodes"].members().filter_map(|raw| into_node(&raw)).collect();
    let mut links: Vec<(PinId, PinId)> = root["links"].members().filter_map(|raw| into_link(raw)).collect();
    // drop bad links
    links.retain(|(from, to)| from.node_index < nodes.len() && to.node_index < nodes.len());
    Ok(Graph { nodes, links })
}

fn from_nodetype(node_type: NodeType) -> json::JsonValue {
    match node_type {
        NodeType::Time => json::object!{"type": "time"},
        NodeType::Float(value) => json::object!{"type": "float", value: value},
        NodeType::String(value) => json::object!{"type": "string", value: value},
        NodeType::Color(value) => json::object!{"type": "color", value: value.to_hex()},
        NodeType::Lerp => json::object!{"type": "lerp"},
        NodeType::Cubic(is_in) => json::object!{"type": "cubic", "in": is_in},
        NodeType::Pixmap(path) => json::object!{"type": "pixmap", path: path.to_str()},
        NodeType::TransformColorField => json::object!{"type": "transform-color-field" },
        NodeType::Revolution => json::object!{"type": "revolution"},
        NodeType::Rotate => json::object!{"type": "rotate"},
        NodeType::Scale => json::object!{"type": "scale"},
        NodeType::Hex => json::object!{"type": "hex"},
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
    let mut timeline = Timeline::new(30.0);
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
    play: bool,
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
            play: false,
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
    fn cap_caret(&mut self) {
        if self.caret.millis > self.duration().millis {
            self.caret = Instant::zero().after(&Duration::from_millis(self.duration().millis - 1));
        }
    }
    fn delete_selected(&mut self) {
        if let Some(index) = self.selected_index() {
            self.blocks.remove(index);
            self.cap_caret();
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
    
    // return global time as 0-1
    fn global_time(&self) -> f32 {
        self.caret.millis as f32 / self.duration().as_millis() as f32
    }

    // returns the time in the block as 0-1
    fn local_time(&self) -> f32 {
        let mut start = Instant::zero();
        for (duration, _) in &self.blocks {
            let end = start.after(duration);
            if self.caret.millis < end.millis {
                return (self.caret.millis - start.millis) as f32 / duration.millis as f32;
            }
            start = end;
        }
        0.0
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
            ui.toggle_value(&mut self.play, "play");
            if self.play {
                // simple play
                self.timeline.caret.millis += 1000 / self.timeline.fps as u32;
                self.timeline.cap_caret();
                ctx.request_repaint_after_secs(1.0 / self.timeline.fps);
            }
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
                if ui.button("lerp").clicked() {
                    self.add_node(NodeType::Lerp);
                }
                if ui.button("cubic").clicked() {
                    self.add_node(NodeType::Cubic(true));
                }
                if ui.button("rotate").clicked() {
                    self.add_node(NodeType::Rotate);
                }
                if ui.button("scale").clicked() {
                    self.add_node(NodeType::Scale);
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
                if ui.button("transform color field").clicked() {
                    self.add_node(NodeType::TransformColorField);
                }
            });
    

            // output window
            // evaluate pixmap
            // compute global time
            let t = self.timeline.global_time();
            // compute local time
            let local_t = self.timeline.local_time();
            if let PinValue::Pixmap(pixmap) = resolve(&self.graph(), 0, 0, local_t) {
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

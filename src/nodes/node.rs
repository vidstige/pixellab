use egui::{Color32, Id, PointerState, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(Clone, Copy)]
enum PinDirection {
    Input,
    Output,
}

#[derive(Clone, Copy)]
struct PinId {
    node_index: usize,
    pin_index: usize,
    direction: PinDirection,
}

#[derive(Debug)]
pub struct Pin {
}

impl Pin {
    pub(crate) fn new() -> Self {
        Self { }
    }
}

#[derive(Debug)]
pub struct Node {
    pub inputs: Vec<Pin>,
    pub outputs: Vec<Pin>,
    pub rect: Rect,
}

impl Node {
    pub(crate) fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            rect: Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(100.0, 100.0)),
        }
    }
}

fn pin_position(rect: &Rect, pin_index: usize, direction: PinDirection) -> Pos2 {
    let x = match direction {
        PinDirection::Input => rect.left(),
        PinDirection::Output => rect.right(),
    };
    let y = rect.top() + 32.0 + pin_index as f32 * 16.0;
    Pos2::new(x, y)
}

pub struct Nodes {
    pub nodes: Vec<Node>,
    pub links: Vec<(usize, usize)>,
    link_from: Option<PinId>, // holds link currently being connected, if any 
}

impl Nodes {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), links: Vec::new(), link_from: None, }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let sense = Sense::drag();
        let (rect, response) = ui.allocate_at_least(ui.available_size(), sense);

        let mut pointer = None;
        ui.input(|input| {
            pointer = input.pointer.latest_pos();
        });
        
        let painter = ui.painter();
        // draw currently connecting link, if any
        /*if let Some(from_id) = &self.link_from {
            if response.contains_pointer() {
                println!("linking {}", response.drag_motion());
                let rect = self.nodes[from_id.node_index].rect;
                let center = pin_position(&rect, from_id.pin_index, from_id.direction);
                let mut lines = Vec::new();
                lines.push(center);
                lines.push(center + response.drag_motion());
                painter.line(lines, Stroke::new(2.0, Color32::WHITE));
            }
        }*/

        for (node_index, node) in self.nodes.iter_mut().enumerate() {
            // draw links
            
            // draw rect
            painter.rect_filled(node.rect, 4.0, Color32::DARK_GRAY);
            
            // draw input pins
            for (pin_index, pin) in node.inputs.iter_mut().enumerate() {
                let center = pin_position(&node.rect, pin_index, PinDirection::Input);
                let radius = 8.0;
                painter.circle_filled(center, radius, Color32::LIGHT_BLUE);
                
                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                let id = ui.id().with(node_index).with(pin_index);
                let response = ui.interact(pin_rect, id, Sense::drag());
                if response.drag_started() {
                    self.link_from = Some(PinId { node_index, pin_index, direction: PinDirection::Input});
                }
                if response.dragged() {
                    if let Some(pointer) = pointer {
                        let mut lines = Vec::new();
                        lines.push(center);
                        lines.push(pointer);
                        painter.line(lines, Stroke::new(2.0, Color32::WHITE));
                    }
                }
                if response.drag_stopped() {
                    self.link_from = None;
                }
            }
            // draw output pins
            for (pin_index, pin) in node.outputs.iter_mut().enumerate() {
                let center = pin_position(&node.rect, pin_index, PinDirection::Output);
                let radius = 8.0;
                painter.circle_filled(center, radius, Color32::LIGHT_BLUE);
            }

            let response = ui.interact(node.rect, ui.id().with(node_index), sense);
            if response.dragged() {
                node.rect = node.rect.translate(response.drag_delta());
            }
        }
        response
    }
}
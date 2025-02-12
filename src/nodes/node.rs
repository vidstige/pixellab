use egui::{Color32, Id, Pos2, Rect, Response, Sense, Stroke, Vec2, Widget};

#[derive(Clone, Copy, Hash)]
enum PinDirection {
    Input,
    Output,
}

impl PinDirection {
    fn opposite(&self) -> &PinDirection {
        match self {
            PinDirection::Input => &PinDirection::Output,
            PinDirection::Output => &PinDirection::Input,
        }
    }
}

#[derive(Clone, Copy)]
struct PinId {
    node_index: usize,
    pin_index: usize,
    direction: PinDirection,
}
impl PinId {
    fn id(&self, ui: &egui::Ui) -> Id {
        ui.id().with(self.node_index).with(self.pin_index).with(self.direction)
    }
}

#[derive(Debug)]
pub struct Pin {
}

impl Pin {
    pub(crate) fn new() -> Self {
        Self { }
    }
}

pub trait NodeWidget {
    fn ui(&mut self, ui: &mut egui::Ui) -> Response;
}

#[derive(Debug)]
pub struct Node<W: NodeWidget> {
    pub widget: W,
    pub inputs: Vec<Pin>,
    pub outputs: Vec<Pin>,
    pub rect: Rect,
}

impl<W: NodeWidget> Node<W> {
    pub(crate) fn new(widget: W) -> Self {
        Self {
            widget,
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

pub struct Nodes<W: NodeWidget> {
    pub nodes: Vec<Node<W>>,
    pub links: Vec<(PinId, PinId)>,
    link_from: Option<PinId>, // holds link currently being connected, if any 
}

impl<W: NodeWidget> Nodes<W> {
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

        
        // draw links
        
        for (from, to) in &self.links {
            let from_node = &self.nodes[from.node_index];
            let from_center = pin_position(&from_node.rect, from.pin_index, from.direction);
            
            let to_node = &self.nodes[to.node_index];
            let to_center = pin_position(&to_node.rect, to.pin_index, to.direction);
            
            let mut lines = Vec::new();
            lines.push(from_center);
            lines.push(to_center);
            let painter = ui.painter();
            painter.line(lines, Stroke::new(2.0, Color32::WHITE));
        }

        for (node_index, node) in self.nodes.iter_mut().enumerate() {
            // draw rect
            //let painter = ui.painter();
            //painter.rect_filled(node.rect, 4.0, Color32::DARK_GRAY);
            
            let mut frame = egui::Frame::group(ui.style());
            ui.push_id(node_index, |ui| {
                ui.put(node.rect, &mut frame);
            });
            frame.show(ui, |ui| node.widget.ui(ui));
            
            // draw input pins
            let painter = ui.painter();
            for (pin_index, pin) in node.inputs.iter().enumerate() {
                let center = pin_position(&node.rect, pin_index, PinDirection::Input);
                let radius = 8.0;
                painter.circle_filled(center, radius, Color32::LIGHT_BLUE);
                
                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                let pin_id = PinId { node_index, pin_index, direction: PinDirection::Input};
                let response = ui.interact(pin_rect, pin_id.id(ui), Sense::drag());
                if response.drag_started() {
                    self.link_from = Some(pin_id);
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
                    if let Some(pointer_pos) = pointer {
                        // check if dropped into any of the output nodes
                        /*for (node_index, node) in self.nodes.iter().enumerate() {
                            for (pin_index, pin) in node.outputs.iter().enumerate() {
                                let center = pin_position(&node.rect, pin_index, PinDirection::Output);
                                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                                if pin_rect.contains(pointer_pos) {
                                    self.links.push((PinId { node_index, pin_index, direction: PinDirection::Output}, self.link_from.unwrap()));
                                }
                            }
                        }*/
                    }
                    self.link_from = None;
                }
            }
            // draw output pins
            for (pin_index, pin) in node.outputs.iter().enumerate() {
                let center = pin_position(&node.rect, pin_index, PinDirection::Output);
                let radius = 8.0;
                painter.circle_filled(center, radius, Color32::LIGHT_BLUE);
                
                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                let pin_id = PinId { node_index, pin_index, direction: PinDirection::Output};
                let response = ui.interact(pin_rect, pin_id.id(ui), Sense::drag());
                if response.drag_started() {
                    self.link_from = Some(pin_id);
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
                    if let Some(pointer_pos) = pointer {
                        // check if dropped into any of the input nodes
                        /*for (node_index, node) in self.nodes.iter().enumerate() {
                            for (pin_index, pin) in node.inputs.iter().enumerate() {
                                let center = pin_position(&node.rect, pin_index, PinDirection::Input);
                                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                                if pin_rect.contains(pointer_pos) {
                                    self.links.push((self.link_from.unwrap(), PinId { node_index, pin_index, direction: PinDirection::Input}));
                                }
                            }
                        }*/
                    }
                    self.link_from = None;
                }
            }
        }
        for (node_index, node) in self.nodes.iter_mut().enumerate() {
            let response = ui.interact(node.rect, ui.id().with(node_index), sense);
            if response.dragged() {
                node.rect = node.rect.translate(response.drag_delta());
            }
        }
        response
    }
}
use egui::{Color32, Context, Id, Pos2, Rect, Response, Sense, Stroke, Vec2, Widget};

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
pub struct PinId {
    pub node_index: usize,
    pub pin_index: usize,
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
    fn in_pins(&self) -> Vec<Pin>;
    fn out_pins(&self) -> Vec<Pin>;
    fn ui(&mut self, ui: &mut egui::Ui) -> Response;
}

#[derive(Debug)]
pub struct Node<W: NodeWidget> {
    pub widget: W,
}

impl<W: NodeWidget> Node<W> {
    pub(crate) fn new(widget: W) -> Self {
        Self {
            widget,
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
    pub fn show(&mut self, ctx: &Context, ui: &mut egui::Ui) -> egui::Response {
        let sense = Sense::drag();
        let (rect, response) = ui.allocate_at_least(ui.available_size(), sense);

        let mut node_rects = Vec::new();
        for (node_index, node) in self.nodes.iter_mut().enumerate() {
            let area = egui::Area::new(Id::new(node_index)).movable(true);
            let response = area.show(ctx, |ui| {
                let frame = egui::Frame::group(ui.style()).fill(ui.style().visuals.panel_fill);
                frame.show(ui, |ui| {
                    ui.set_min_size(Vec2::new(48.0, 64.0));
                    node.widget.ui(ui);
                });
            }).response;
            let node_rect = response.rect;
            node_rects.push(node_rect);
        }

        // draw links        
        for (from, to) in &self.links {
            let from_rect = &node_rects[from.node_index];
            let from_center = pin_position(from_rect, from.pin_index, from.direction);

            let to_rect = &node_rects[to.node_index];
            let to_center = pin_position(to_rect, to.pin_index, to.direction);

            let mut lines = Vec::new();
            lines.push(from_center);
            lines.push(to_center);
            let painter = ui.painter();
            painter.line(lines, Stroke::new(2.0, Color32::WHITE));
        }

        // pre-calculate all inputs and outputs to avoid mutable borrow woes
        let radius = 8.0;
        let mut output_pins = Vec::new();
        let mut input_pins = Vec::new();
        for (node_index, (node, node_rect)) in self.nodes.iter().zip(node_rects.iter()).enumerate() {
            for (pin_index, pin) in node.widget.out_pins().iter().enumerate() {
                let center = pin_position(&node_rect, pin_index, PinDirection::Output);
                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                output_pins.push((node_index, pin_index, pin_rect));
            }
            for (pin_index, pin) in node.widget.in_pins().iter().enumerate() {
                let center = pin_position(&node_rect, pin_index, PinDirection::Input);
                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                input_pins.push((node_index, pin_index, pin_rect));
            }
        }

        // draw pins
        for (node_index, (node, node_rect)) in self.nodes.iter().zip(node_rects.iter()).enumerate() {
            // draw input pins
            let painter = ui.painter();
            for (pin_index, pin) in node.widget.in_pins().iter().enumerate() {
                let center = pin_position(&node_rect, pin_index, PinDirection::Input);
                painter.circle_filled(center, radius, Color32::LIGHT_BLUE);
                
                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                let pin_id = PinId { node_index, pin_index, direction: PinDirection::Input};
                let response = ui.interact(pin_rect, pin_id.id(ui), Sense::drag());
                if response.drag_started() {
                    self.link_from = Some(pin_id);
                }
                if response.dragged() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let mut lines = Vec::new();
                        lines.push(center);
                        lines.push(pointer);
                        painter.line(lines, Stroke::new(2.0, Color32::WHITE));
                    }
                }
                if response.drag_stopped() {
                    println!("drag stopped");
                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                        // check if dropped into any of the output nodes
                        for (node_index, pin_index, pin_rect) in &output_pins {
                            println!("{} {}", pin_rect, pointer_pos);
                            if pin_rect.contains(pointer_pos) {
                                self.links.push((PinId { node_index: *node_index, pin_index: *pin_index, direction: PinDirection::Output}, self.link_from.unwrap()));
                            }
                        }
                    }
                    self.link_from = None;
                }
            }
            
            // draw output pins
            for (pin_index, pin) in node.widget.out_pins().iter().enumerate() {
                let center = pin_position(node_rect, pin_index, PinDirection::Output);
                let radius = 8.0;
                painter.circle_filled(center, radius, Color32::LIGHT_BLUE);
                
                let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
                let pin_id = PinId { node_index, pin_index, direction: PinDirection::Output};
                let response = ui.interact(pin_rect, pin_id.id(ui), Sense::drag());
                if response.drag_started() {
                    self.link_from = Some(pin_id);
                }
                if response.dragged() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let mut lines = Vec::new();
                        lines.push(center);
                        lines.push(pointer);
                        painter.line(lines, Stroke::new(2.0, Color32::WHITE));
                    }
                }
                if response.drag_stopped() {
                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                        // check if dropped into any of the input nodes
                        for (node_index, pin_index, pin_rect) in &input_pins {
                            if pin_rect.contains(pointer_pos) {
                                self.links.push((PinId { node_index: *node_index, pin_index: *pin_index, direction: PinDirection::Input}, self.link_from.unwrap()));
                            }
                        }
                    }
                    self.link_from = None;
                }
            }
        }
        response
    }

    // Finds all PinIds linking to the specified node_index
    pub fn inputs_for(&self, node_index: usize) -> Vec<PinId> {
        self.links
            .iter()
            .filter_map(|(from, to)| (to.node_index == node_index).then_some(*from))
            .collect()
    }
}
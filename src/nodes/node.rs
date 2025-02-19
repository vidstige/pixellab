use egui::{Color32, Context, Id, Pos2, Rect, Response, Sense, Stroke, Vec2};

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum PinDirection {
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct PinId {
    pub node_index: usize,
    pub pin_index: usize,
    pub direction: PinDirection,
}
impl PinId {
    fn id(&self, ui: &egui::Ui) -> Id {
        ui.id().with(self.node_index).with(self.pin_index).with(self.direction)
    }
    fn link(self, other: PinId) -> (PinId, PinId) {
        match &self.direction {
            &PinDirection::Input => (other, self),
            &PinDirection::Output => (self, other),
        }
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
    fn title(&self) -> String;
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

pub struct Graph<W: NodeWidget> {
    pub nodes: Vec<Node<W>>,
    pub links: Vec<(PinId, PinId)>,
}

fn disconnect_pin(links: &mut Vec<(PinId, PinId)>, pin_id: &PinId) -> bool {
    let before = links.len();
    links.retain(|(from, to)| from != pin_id && to != pin_id);
    let after = links.len();
    after < before
}

fn pins_ui(pins: &Vec<Pin>, direction: PinDirection, links: &mut Vec<(PinId, PinId)>, node_index: usize, node_rect: &Rect, ui: &egui::Ui, radius: f32) {
    let painter = ui.painter();
    for (pin_index, pin) in pins.iter().enumerate() {
        let center = pin_position(node_rect, pin_index, direction);
        painter.circle_filled(center, radius, Color32::LIGHT_BLUE);
        
        let pin_rect = Rect::from_center_size(center, Vec2::splat(2.0 * radius));
        let pin_id = PinId { node_index, pin_index, direction};
        let response = ui.interact(pin_rect, pin_id.id(ui), Sense::drag());
        
        if response.drag_started() {
            // remove pin if it exists
            if !disconnect_pin(links, &pin_id) {
                response.dnd_set_drag_payload(pin_id);
            }
        }

        if response.dragged() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let mut lines = Vec::new();
                lines.push(center);
                lines.push(pointer);
                painter.line(lines, Stroke::new(2.0, Color32::WHITE));
            }
        }
        if let Some(link_from) = response.dnd_release_payload() {
            links.push(pin_id.link(*link_from));
        }
    }
}

impl<W: NodeWidget> Graph<W> {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), links: Vec::new() }
    }
    pub fn show(&mut self, ctx: &Context, ui: &mut egui::Ui) -> egui::Response {
        let sense = Sense::drag();
        let (rect, response) = ui.allocate_at_least(ui.available_size(), sense);

        let mut node_rects = Vec::new();
        let mut closed_indices = Vec::new();
        for (node_index, node) in self.nodes.iter_mut().enumerate() {
            let frame = egui::Frame::group(ui.style()).fill(ui.style().visuals.panel_fill);
            let window = egui::Window::new(node.widget.title())
                .id(Id::new(node_index))
                .frame(frame)
                .resizable(false);
            let mut is_open = true;
            let maybe_response = window.open(&mut is_open).show(ctx, |ui| {
                ui.set_min_size(Vec2::new(48.0, 64.0));
                node.widget.ui(ui);
            });
            if is_open {
                let node_rect = maybe_response.unwrap().response.rect;
                node_rects.push(node_rect);
            } else {
                closed_indices.push(node_index)
            } 
        }
        closed_indices.reverse();
        for index in closed_indices {
            self.remove_node(index);
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
            pins_ui(&node.widget.in_pins(), PinDirection::Input, &mut self.links, node_index, &node_rect, ui, radius);
            pins_ui(&node.widget.out_pins(), PinDirection::Output, &mut self.links, node_index, &node_rect, ui, radius);
        }
        response
    }

    // Finds all PinIds linking to the specified node_index
    pub fn inputs_for(&self, node_index: usize) -> Vec<PinId> {
        let mut links: Vec<_> = self.links
            .iter()
            .filter(|(_, to)| (to.node_index == node_index))
            .collect();
        links.sort_by_key(|(_, to)| to.pin_index);
        links.iter().map(|(from, _)| *from).collect()
    }
    
    fn remove_node<>(&mut self, index: usize) {
        // fist update all links referencing a node after this
        for (from, to) in self.links.iter_mut() {
            if from.node_index > index {
                from.node_index -= 1;
            }
            if to.node_index > index {
                to.node_index -= 1;
            }
        }
        // remove all links referencing this node
        self.links.retain(|(from, to)| from.node_index != index && to.node_index != index);
        // finally actully remove node
        self.nodes.remove(index);
    }
}
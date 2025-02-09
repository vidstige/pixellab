use egui::{response, Color32, Pos2, Rect, Response, Sense, Vec2, Widget};

#[derive(Debug)]
pub struct Node {
    rect: Rect,
}
impl Node {
    pub(crate) fn new() -> Self {
        Self { rect: Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::new(100.0, 100.0))}
    }
}

pub struct Nodes {
    pub nodes: Vec<Node>,
    pub links: Vec<(usize, usize)>,
}

impl Nodes {
    pub fn new() -> Self {
        Self { nodes: Vec::new(), links: Vec::new(), }
    }
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let sense = Sense::DRAG;
        let (rect, response) = ui.allocate_at_least(ui.available_size(), sense);
        //let canvas_origin_screen_space = rect.min.to_vec2();
        
        let painter = ui.painter();
        //painter.debug_rect(rect, Color32::KHAKI, "rect");
        for node in &mut self.nodes {
            painter.rect_filled(node.rect, 4.0, Color32::DARK_GREEN);
            let response = ui.interact(node.rect, ui.id(), sense);
            if response.dragged() {
                node.rect = node.rect.translate(response.drag_delta());
            }
        }
        
        response
    }
}
use tiny_skia::{FillRule, Paint, Path, PathBuilder, Pixmap, Point, Rect, Transform};

fn hex_tile(size: f32) -> Path {
    // pointy top
    let w = 3.0_f32.sqrt() / 2.0 * size;
    let mut pb = PathBuilder::with_capacity(7, 6);
    pb.move_to(0.0, -size); // top
    pb.line_to(w, - 0.5 * size); // top right
    pb.line_to(w, 0.5 * size); // bottom right
    pb.line_to(0.0, size); // bottom
    pb.line_to(- w, 0.5 * size); // bottom left
    pb.line_to(- w, -0.5 * size); // top left
    pb.close();
    pb.finish().unwrap()
}

pub struct HexGrid {
    spacing: f32,
    size: f32,
    transform: Transform,
}
impl HexGrid {
    pub fn new(spacing: f32, size: f32, transform: Transform) -> Self {
        Self { spacing, size, transform }
    }
    fn position(&self, q: i32, r: i32) -> Point {
        let x = self.spacing * 3.0_f32.sqrt() * (q as f32 + 0.5 * (r & 1) as f32);
        let y = self.spacing * 3.0/2.0 * r as f32;
        Point { x, y }
    }
}

fn bounds_for(pixmap: &Pixmap) -> Rect {
    Rect::from_xywh(0.0, 0.0, pixmap.width() as f32, pixmap.height() as f32).unwrap()
}

pub fn draw_hex_grid<'a>(pixmap: &mut Pixmap, paint: &Paint<'a>, grid: &HexGrid) {
    let screen = bounds_for(pixmap);
    let rect = screen.transform(grid.transform.invert().unwrap()).unwrap();
    let hex_tile = hex_tile(grid.size);
    let (x0, y0) = (rect.left() as i32, rect.top() as i32);
    let (x1, y1) = (rect.right() as i32, rect.bottom() as i32);
    for r in y0..y1 {
        for q in x0..x1 {
            let p = grid.position(q, r);
            pixmap.fill_path(
                &hex_tile,
                &paint,
                FillRule::Winding,
                grid.transform.pre_translate(p.x, p.y),
                None,
            );
        }
    }
}
use tiny_skia::Point;

// represnts a field that can be evaluated a specific point, e.g. color field, scalar field, vector field
pub(crate) trait Field2<T> {
    fn at(&self, position: Point) -> T;
}

pub(crate) struct ConstantField<T: Clone> {
    value: T,
}
impl<T: Clone> ConstantField<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}
impl<T: Clone> Field2<T> for ConstantField<T> {
    fn at(&self, _position: Point) -> T {
        self.value.clone()
    }
}

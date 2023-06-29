use crate::{Color, SpanGenerator};
//--------------------------------------------------------------span_solid
pub struct SpanSolid<C: Color> {
    pub color: C,
}

impl<C: Color> SpanSolid<C> {
    pub fn new(color: C) -> Self {
        Self { color }
    }

    pub fn color(&self) -> C {
        self.color
    }
}

impl<C: Color> SpanGenerator for SpanSolid<C> {
	type C = C;
    fn prepare(&mut self) {}

    fn generate(&mut self, span: &mut [C], _x: i32, _y: i32, len: u32) {
        for i in 0..len {
            span[i as usize] = self.color;
        }
    }
}

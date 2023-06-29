use crate::basics::{is_end_poly, PathFlag};
use crate::{Equiv, VertexSource};

pub struct ConvUnclosePolygon<'a, VS: VertexSource> {
    source: Equiv<'a, VS>,
}

impl<'a, VS: VertexSource> ConvUnclosePolygon<'a, VS> {
    pub fn new_owned(source: VS) -> Self {
        ConvUnclosePolygon {
            source: Equiv::Own(source),
        }
    }

    pub fn new_borrowed(source: &'a mut VS) -> Self {
        ConvUnclosePolygon {
            source: Equiv::Brw(source),
        }
    }

    pub fn set_source_owned(&mut self, source: VS) {
        self.source = Equiv::Own(source);
    }

    pub fn set_source_borrowed(&mut self, source: &'a mut VS) {
        self.source = Equiv::Brw(source);
    }
}

impl<'a, VS: VertexSource> VertexSource for ConvUnclosePolygon<'a, VS> {
    fn rewind(&mut self, path_id: u32) {
        self.source.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let cmd = self.source.vertex(x, y);
        if is_end_poly(cmd) {
            cmd & !(PathFlag::Close as u32)
        } else {
            cmd
        }
    }
}

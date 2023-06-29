use crate::basics::{is_stop, PathCmd};
use crate::{Equiv, VertexSource};

//NOT TESTED

//=============================================================ConvConcat
// Concatenation of two paths. Usually used to combine lines or curves
// with markers such as arrowheads
pub struct ConvConcat<'a, VS1: VertexSource, VS2: VertexSource> {
    source1: Equiv<'a, VS1>,
    source2: Equiv<'a, VS2>,
    status: i32,
}

impl<'a, VS1: VertexSource, VS2: VertexSource> ConvConcat<'a, VS1, VS2> {
    pub fn new_owned(source1: VS1, source2: VS2) -> Self {
        ConvConcat {
            source1: Equiv::Own(source1),
            source2: Equiv::Own(source2),
            status: 2,
        }
    }

    pub fn new_borrowed(source1: &'a mut VS1, source2: &'a mut VS2) -> Self {
        ConvConcat {
            source1: Equiv::Brw(source1),
            source2: Equiv::Brw(source2),
            status: 2,
        }
    }
}

impl<'a, VS1: VertexSource, VS2: VertexSource> VertexSource for ConvConcat<'a, VS1, VS2> {
    fn rewind(&mut self, path_id: u32) {
        self.source1.rewind(path_id);
        self.source2.rewind(0);
        self.status = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd;
        if self.status == 0 {
            cmd = self.source1.vertex(x, y);
            if !is_stop(cmd) {
                return cmd;
            }
            self.status = 1;
        }
        if self.status == 1 {
            cmd = self.source2.vertex(x, y);
            if !is_stop(cmd) {
                return cmd;
            }
            self.status = 2;
        }
        return PathCmd::Stop as u32;
    }
}

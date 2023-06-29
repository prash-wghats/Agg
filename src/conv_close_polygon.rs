use crate::basics::{PathCmd, PathFlag, *};
use crate::{Equiv, VertexSource};

//NOT TESTED

//======================================================ConvClosePolygon
pub struct ConvClosePolygon<'a, VS: VertexSource> {
    source: Equiv<'a, VS>,
    cmd: [u32; 2],
    x: [f64; 2],
    y: [f64; 2],
    vertex: u32,
    line_to: bool,
}

impl<'a, VS: VertexSource> ConvClosePolygon<'a, VS> {
    pub fn new_borrowed(source: &'a mut VS) -> Self {
        ConvClosePolygon {
            source: Equiv::Brw(source),
            cmd: [0, 0],
            x: [0.0, 0.0],
            y: [0.0, 0.0],
            vertex: 2,
            line_to: false,
        }
    }

    pub fn new_owned(source: VS) -> Self {
        ConvClosePolygon {
            source: Equiv::Own(source),
            cmd: [0, 0],
            x: [0.0, 0.0],
            y: [0.0, 0.0],
            vertex: 2,
            line_to: false,
        }
    }

    pub fn set_source_owned(&mut self, source: VS) {
        self.source = Equiv::Own(source);
    }

    pub fn set_source_borrowed(&mut self, source: &'a mut VS) {
        self.source = Equiv::Brw(source);
    }
}

impl<'a, VS: VertexSource> VertexSource for ConvClosePolygon<'a, VS> {
    fn rewind(&mut self, path_id: u32) {
        self.source.rewind(path_id);
        self.vertex = 2;
        self.line_to = false;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd;
        loop {
            if self.vertex < 2 {
                *x = self.x[self.vertex as usize];
                *y = self.y[self.vertex as usize];
                cmd = self.cmd[self.vertex as usize];
                self.vertex += 1;
                break;
            }

            cmd = self.source.vertex(x, y);

            if is_end_poly(cmd) {
                cmd |= PathFlag::Close as u32;
                break;
            }

            if is_stop(cmd) {
                if self.line_to {
                    self.cmd[0] = PathCmd::EndPoly as u32 | PathFlag::Close as u32;
                    self.cmd[1] = PathCmd::Stop as u32;
                    self.vertex = 0;
                    self.line_to = false;
                    continue;
                }
                break;
            }

            if is_move_to(cmd) {
                if self.line_to {
                    self.x[0] = 0.0;
                    self.y[0] = 0.0;
                    self.cmd[0] = PathCmd::EndPoly as u32 | PathFlag::Close as u32;
                    self.x[1] = *x;
                    self.y[1] = *y;
                    self.cmd[1] = cmd;
                    self.vertex = 0;
                    self.line_to = false;
                    continue;
                }
                break;
            }

            if is_vertex(cmd) {
                self.line_to = true;
                break;
            }
        }
        cmd
    }
}

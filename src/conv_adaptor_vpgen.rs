use crate::basics::{is_closed, is_end_poly, is_move_to, is_stop, is_vertex, PathCmd, PathFlag};
use crate::{Equiv, VertexSource, VpGenerator};

//======================================================conv_adaptor_vpgen
pub struct ConvAdaptorVpgen<'a, VS: VertexSource, VG: VpGenerator> {
    source: Equiv<'a, VS>,
    vpgen: VG,
    start_x: f64,
    start_y: f64,
    poly_flags: u32,
    vertices: i32,
}

impl<'a, VS: VertexSource, VG: VpGenerator> ConvAdaptorVpgen<'a, VS, VG> {
    pub fn new_owned(source: VS) -> Self {
        ConvAdaptorVpgen {
            source: Equiv::Own(source),
            vpgen: <VG as VpGenerator>::new(),
            start_x: 0.0,
            start_y: 0.0,
            poly_flags: 0,
            vertices: 0,
        }
    }

    pub fn new_borrowed(source: &'a mut VS) -> Self {
        ConvAdaptorVpgen {
            source: Equiv::Brw(source),
            vpgen: <VG as VpGenerator>::new(),
            start_x: 0.0,
            start_y: 0.0,
            poly_flags: 0,
            vertices: 0,
        }
    }

    pub fn set_source_owned(&mut self, source: VS) {
        self.source = Equiv::Own(source);
    }

    pub fn set_source_borrowed(&mut self, source: &'a mut VS) {
        self.source = Equiv::Brw(source);
    }

    pub fn source_mut(&mut self) -> &mut VS {
        &mut self.source
    }

    pub fn source(&self) -> &VS {
        &self.source
    }

    pub fn vpgen_mut(&mut self) -> &mut VG {
        &mut self.vpgen
    }

    pub fn vpgen(&self) -> &VG {
        &self.vpgen
    }
}

impl<'a, VS: VertexSource, VG: VpGenerator> VertexSource for ConvAdaptorVpgen<'a, VS, VG> {
    fn rewind(&mut self, path_id: u32) {
        self.source.rewind(path_id);
        self.vpgen.reset();
        self.start_x = 0.0;
        self.start_y = 0.0;
        self.poly_flags = 0;
        self.vertices = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd; // = PathCmd::Stop as u32;
        loop {
            cmd = self.vpgen.vertex(x, y);
            if !is_stop(cmd) {
                break;
            }

            if self.poly_flags != 0 && !self.vpgen.auto_unclose() {
                *x = 0.0;
                *y = 0.0;
                cmd = self.poly_flags;
                self.poly_flags = 0;
                break;
            }

            if self.vertices < 0 {
                if self.vertices < -1 {
                    self.vertices = 0;
                    return PathCmd::Stop as u32;
                }
                self.vpgen.move_to(self.start_x, self.start_y);
                self.vertices = 1;
                continue;
            }

            let mut tx: f64 = 0.0;
            let mut ty: f64 = 0.0;
            cmd = self.source.vertex(&mut tx, &mut ty);
            if is_vertex(cmd) {
                if is_move_to(cmd) {
                    if self.vpgen.auto_close() && self.vertices > 2 {
                        self.vpgen.line_to(self.start_x, self.start_y);
                        self.poly_flags = PathCmd::EndPoly as u32 | PathFlag::Close as u32;
                        self.start_x = tx;
                        self.start_y = ty;
                        self.vertices = -1;
                        continue;
                    }
                    self.vpgen.move_to(tx, ty);
                    self.start_x = tx;
                    self.start_y = ty;
                    self.vertices = 1;
                } else {
                    self.vpgen.line_to(tx, ty);
                    self.vertices += 1;
                }
            } else {
                if is_end_poly(cmd) {
                    self.poly_flags = cmd;
                    if is_closed(cmd) || self.vpgen.auto_close() {
                        if self.vpgen.auto_close() {
                            self.poly_flags |= PathFlag::Close as u32;
                        }
                        if self.vertices > 2 {
                            self.vpgen.line_to(self.start_x, self.start_y);
                        }
                        self.vertices = 0;
                    }
                } else {
                    // PathCmd::Stop
                    if self.vpgen.auto_close() && self.vertices > 2 {
                        self.vpgen.line_to(self.start_x, self.start_y);
                        self.poly_flags = PathCmd::EndPoly as u32 | PathFlag::Close as u32;
                        self.vertices = -2;
                        continue;
                    }
                    break;
                }
            }
        }
        cmd
    }
}

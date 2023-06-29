use crate::array::{PodBVector};
use crate::basics::{get_close_flag, is_move_to, is_stop, is_vertex, PathCmd};
use crate::vertex_sequence::{VecSequence, VertexDist};
use crate::{Generator, VertexSource, VertexSequence};

/// XXXX TESTING

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusE {
    Initial,
    Ready,
    Polygon,
    CtrlB,
    CtrlE,
    Ctrl1,
    Ctrl2,
    EndPoly,
    Stop,
}
//======================================================VcgenSmoothPoly1
pub type VertexStorage = VecSequence<VertexDist>;

pub struct VcgenSmoothPoly1 {
    src_vertices: VertexStorage,
    smooth_value: f64,
    closed: u32,
    status: StatusE,
    src_vertex: usize,
    ctrl1_x: f64,
    ctrl1_y: f64,
    ctrl2_x: f64,
    ctrl2_y: f64,
}

impl Generator for VcgenSmoothPoly1 {
    fn new() -> VcgenSmoothPoly1 {
        VcgenSmoothPoly1 {
            src_vertices: VertexStorage::new(),
            smooth_value: 0.5,
            closed: 0,
            status: StatusE::Initial,
            src_vertex: 0,
            ctrl1_x: 0.0,
            ctrl1_y: 0.0,
            ctrl2_x: 0.0,
            ctrl2_y: 0.0,
        }
    }

    fn remove_all(&mut self) {
        self.src_vertices.remove_all();
        self.closed = 0;
        self.status = StatusE::Initial;
    }

    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        self.status = StatusE::Initial;
        if is_move_to(cmd) {
            self.src_vertices.modify_last(VertexDist::new(x, y));
        } else {
            if is_vertex(cmd) {
                self.src_vertices.add(VertexDist::new(x, y));
            } else {
                self.closed = get_close_flag(cmd);
            }
        }
    }
}

impl VertexSource for VcgenSmoothPoly1 {
    fn rewind(&mut self, _: u32) {
        if self.status == StatusE::Initial {
            self.src_vertices.close(self.closed != 0);
        }
        self.status = StatusE::Ready;
        self.src_vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::LineTo as u32;
        while !is_stop(cmd) {
            match self.status {
                StatusE::Initial => {
                    self.rewind(0);
                }
                StatusE::Ready => {
                    if self.src_vertices.size() < 2 {
                        cmd = PathCmd::Stop as u32;
                        break;
                    }
                    if self.src_vertices.size() == 2 {
                        *x = self.src_vertices[self.src_vertex].x;
                        *y = self.src_vertices[self.src_vertex].y;
                        self.src_vertex += 1;
                        if self.src_vertex == 1 {
                            return PathCmd::MoveTo as u32;
                        }
                        if self.src_vertex == 2 {
                            return PathCmd::LineTo as u32;
                        }
                        cmd = PathCmd::Stop as u32;
                        break;
                    }
                    cmd = PathCmd::MoveTo as u32;
                    self.status = StatusE::Polygon;
                    self.src_vertex = 0;
                }
                StatusE::Polygon => {
                    if self.closed != 0 {
                        if self.src_vertex >= self.src_vertices.size() {
                            *x = self.src_vertices[0].x;
                            *y = self.src_vertices[0].y;
                            self.status = StatusE::EndPoly;
                            return PathCmd::Curve4 as u32;
                        }
                    } else {
                        if self.src_vertex >= self.src_vertices.size() - 1 {
                            *x = self.src_vertices[self.src_vertices.size() - 1].x;
                            *y = self.src_vertices[self.src_vertices.size() - 1].y;
                            self.status = StatusE::EndPoly;
                            return PathCmd::Curve3 as u32;
                        }
                    }

                    self.calculate();
                    *x = self.src_vertices[self.src_vertex].x;
                    *y = self.src_vertices[self.src_vertex].y;
                    self.src_vertex += 1;
                    if self.closed != 0 {
                        self.status = StatusE::Ctrl1;
                        return if self.src_vertex == 1 {
                            PathCmd::MoveTo as u32
                        } else {
                            PathCmd::Curve4 as u32
                        };
                    } else {
                        if self.src_vertex == 1 {
                            self.status = StatusE::CtrlB;
                            return PathCmd::MoveTo as u32;
                        }
                        if self.src_vertex >= self.src_vertices.size() - 1 {
                            self.status = StatusE::CtrlE;
                            return PathCmd::Curve3 as u32;
                        }
                        self.status = StatusE::Ctrl1;
                        return PathCmd::Curve4 as u32;
                    }
                }
                StatusE::CtrlB => {
                    *x = self.ctrl2_x;
                    *y = self.ctrl2_y;
                    self.status = StatusE::Polygon;
                    return PathCmd::Curve3 as u32;
                }
                StatusE::CtrlE => {
                    *x = self.ctrl1_x;
                    *y = self.ctrl1_y;
                    self.status = StatusE::Polygon;
                    return PathCmd::Curve3 as u32;
                }
                StatusE::Ctrl1 => {
                    *x = self.ctrl1_x;
                    *y = self.ctrl1_y;
                    self.status = StatusE::Ctrl2;
                    return PathCmd::Curve4 as u32;
                }
                StatusE::Ctrl2 => {
                    *x = self.ctrl2_x;
                    *y = self.ctrl2_y;
                    self.status = StatusE::Polygon;
                    return PathCmd::Curve4 as u32;
                }
                StatusE::EndPoly => {
                    self.status = StatusE::Stop;
                    return PathCmd::EndPoly as u32 | self.closed;
                }
                StatusE::Stop => {
                    return PathCmd::Stop as u32;
                }
            }
        }
        cmd
    }
}

impl VcgenSmoothPoly1 {
    pub fn calculate(&mut self) {
        let v0 = self.src_vertices.prev(self.src_vertex);
        let v1 = self.src_vertices.curr(self.src_vertex);
        let v2 = self.src_vertices.next(self.src_vertex);
        let v3 = self.src_vertices.next(self.src_vertex + 1);
        let k1 = v0.dist / (v0.dist + v1.dist);
        let k2 = v1.dist / (v1.dist + v2.dist);

        let xm1 = v0.x + (v2.x - v0.x) * k1;
        let ym1 = v0.y + (v2.y - v0.y) * k1;
        let xm2 = v1.x + (v3.x - v1.x) * k2;
        let ym2 = v1.y + (v3.y - v1.y) * k2;

        self.ctrl1_x = v1.x + self.smooth_value * (v2.x - xm1);
        self.ctrl1_y = v1.y + self.smooth_value * (v2.y - ym1);
        self.ctrl2_x = v2.x + self.smooth_value * (v1.x - xm2);
        self.ctrl2_y = v2.y + self.smooth_value * (v1.y - ym2);
    }

    pub fn set_smooth_value(&mut self, v: f64) {
        self.smooth_value = v * 0.5;
    }

    pub fn smooth_value(&self) -> f64 {
        self.smooth_value * 2.0
    }
}

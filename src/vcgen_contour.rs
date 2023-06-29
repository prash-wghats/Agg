use crate::array::{PodBVector, VecPodB};
use crate::basics::{
    get_close_flag, get_orientation, is_ccw, is_end_poly, is_move_to, is_oriented, is_stop,
    is_vertex, PathCmd, PathFlag, PointD,
};
use crate::math::calc_polygon_area;
use crate::math_stroke::{InnerJoin, LineCap, LineJoin, MathStroke};
use crate::vertex_sequence::{VecSequence, VertexDist};
use crate::{Generator, VertexSequence, VertexSource};

/// XXXX TESTING

#[derive(Clone, Copy, PartialEq, Eq)]
enum Staus {
    Initial,
    Ready,
    Outline,
    OutVertices,
    EndPoly,
    Stop,
}

pub type VertexStorage = VecSequence<VertexDist>;
pub type CoordStorage = VecPodB<PointD>;

//----------------------------------------------------------VcgenContour
pub struct VcgenContour {
    m_stroker: MathStroke<CoordStorage>,
    m_width: f64,
    m_src_vertices: VertexStorage,
    m_out_vertices: CoordStorage,
    m_status: Staus,
    m_src_vertex: usize,
    m_out_vertex: usize,
    m_closed: bool,
    m_orientation: u32,
    m_auto_detect: bool,
}

impl Generator for VcgenContour {
    fn new() -> VcgenContour {
        VcgenContour {
            m_stroker: MathStroke::new(),
            m_width: 1.0,
            m_src_vertices: VertexStorage::new(),
            m_out_vertices: CoordStorage::new(),
            m_status: Staus::Initial,
            m_src_vertex: 0,
            m_out_vertex: 0,
            m_closed: false,
            m_orientation: PathFlag::None as u32,
            m_auto_detect: false,
        }
    }

    fn remove_all(&mut self) {
        self.m_src_vertices.remove_all();
        self.m_closed = false;
        self.m_orientation = 0;
        self.m_status = Staus::Initial;
    }

    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        self.m_status = Staus::Initial;
        if is_move_to(cmd) {
            self.m_src_vertices.modify_last(VertexDist::new(x, y));
        } else {
            if is_vertex(cmd) {
                self.m_src_vertices.add(VertexDist::new(x, y));
            } else {
                if is_end_poly(cmd) {
                    self.m_closed = get_close_flag(cmd) != 0;
                    if self.m_orientation == PathFlag::None as u32 {
                        self.m_orientation = get_orientation(cmd);
                    }
                }
            }
        }
    }
}

impl VertexSource for VcgenContour {
    fn rewind(&mut self, _: u32) {
        if self.m_status == Staus::Initial {
            self.m_src_vertices.close(true);
            if self.m_auto_detect {
                if !is_oriented(self.m_orientation) {
                    self.m_orientation = if calc_polygon_area(&self.m_src_vertices) > 0.0 {
                        PathFlag::Ccw as u32
                    } else {
                        PathFlag::Cw as u32
                    };
                }
            }
            if is_oriented(self.m_orientation) {
                self.m_stroker.set_width(if is_ccw(self.m_orientation) {
                    self.m_width
                } else {
                    -self.m_width
                });
            }
        }
        self.m_status = Staus::Ready;
        self.m_src_vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::LineTo as u32;
        while !is_stop(cmd) {
            loop {
                match self.m_status {
                    Staus::Initial => {
                        self.rewind(0);
                        self.m_status = Staus::Ready;
                    }
                    Staus::Ready => {
                        if self.m_src_vertices.size() < 2 + self.m_closed as usize {
                            cmd = PathCmd::Stop as u32;
                            break;
                        }
                        self.m_status = Staus::Outline;
                        cmd = PathCmd::MoveTo as u32;
                        self.m_src_vertex = 0;
                        self.m_out_vertex = 0;
                    }
                    Staus::Outline => {
                        if self.m_src_vertex >= self.m_src_vertices.size() {
                            self.m_status = Staus::EndPoly;
                            break;
                        }
                        self.m_stroker.calc_join(
                            &mut self.m_out_vertices,
                            &self.m_src_vertices.prev(self.m_src_vertex),
                            &self.m_src_vertices.curr(self.m_src_vertex),
                            &self.m_src_vertices.next(self.m_src_vertex),
                            self.m_src_vertices.prev(self.m_src_vertex).dist,
                            self.m_src_vertices.curr(self.m_src_vertex).dist,
                        );
                        self.m_src_vertex += 1;
                        self.m_status = Staus::OutVertices;
                        self.m_out_vertex = 0;
                    }
                    Staus::OutVertices => {
                        if self.m_out_vertex >= self.m_out_vertices.len() {
                            self.m_status = Staus::Outline;
                        } else {
                            let c = self.m_out_vertices[self.m_out_vertex];
                            self.m_out_vertex += 1;
                            *x = c.x;
                            *y = c.y;
                            return cmd;
                        }
                    }
                    Staus::EndPoly => {
                        if !self.m_closed {
                            return PathCmd::Stop as u32;
                        }
                        self.m_status = Staus::Stop;
                        return PathCmd::EndPoly as u32
                            | PathFlag::Close as u32
                            | PathFlag::Ccw as u32;
                    }
                    Staus::Stop => {
                        return PathCmd::Stop as u32;
                    }
                }
            }
        }
        cmd
    }
}

impl VcgenContour {
    pub fn set_line_cap(&mut self, lc: LineCap) {
        self.m_stroker.set_line_cap(lc);
    }
    pub fn set_line_join(&mut self, lj: LineJoin) {
        self.m_stroker.set_line_join(lj);
    }
    pub fn set_inner_join(&mut self, ij: InnerJoin) {
        self.m_stroker.set_inner_join(ij);
    }

    pub fn line_cap(&self) -> LineCap {
        self.m_stroker.line_cap()
    }
    pub fn line_join(&self) -> LineJoin {
        self.m_stroker.line_join()
    }
    pub fn inner_join(&self) -> InnerJoin {
        self.m_stroker.inner_join()
    }

	pub fn width(&self) -> f64 {
        self.m_width
    }

    pub fn set_width(&mut self, w: f64) {
        self.m_width = w;
        self.m_stroker.set_width(w);
    }
    pub fn set_miter_limit(&mut self, ml: f64) {
        self.m_stroker.set_miter_limit(ml);
    }
	pub fn miter_limit(&self) -> f64 {
        self.m_stroker.miter_limit()
    }

    pub fn set_miter_limit_theta(&mut self, t: f64) {
        self.m_stroker.set_miter_limit_theta(t);
    }

	pub fn inner_miter_limit(&self) -> f64 {
        self.m_stroker.inner_miter_limit()
    }

    pub fn set_inner_miter_limit(&mut self, ml: f64) {
        self.m_stroker.set_inner_miter_limit(ml);
    }

    pub fn approximation_scale(&self) -> f64 {
        self.m_stroker.approximation_scale()
    }

	pub fn set_approximation_scale(&mut self, a: f64) {
        self.m_stroker.set_approximation_scale(a);
    }

    pub fn set_auto_detect_orientation(&mut self, v: bool) {
        self.m_auto_detect = v;
    }
    pub fn auto_detect_orientation(&self) -> bool {
        return self.m_auto_detect;
    }
}

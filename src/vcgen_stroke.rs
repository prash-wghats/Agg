//----------------------------------------------------------------------------
// Anti-Grain Geometry - Version 2.4
// Copyright (C) 2002-2005 Maxim Shemanarev (http://www.antigrain.com)
//
// Permission to copy, use, modify, sell and distribute this software
// is granted provided this copyright notice appears in all copies.
// This software is provided "as is" without express or implied
// warranty, and with no claim as to its suitability for any purpose.
//
//----------------------------------------------------------------------------
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------
use crate::array::{PodBVector, VecPodB};
use crate::basics::{get_close_flag, is_move_to, is_stop, is_vertex, PathCmd, PathFlag, PointD};
use crate::math_stroke::{InnerJoin, LineCap, LineJoin, MathStroke};
use crate::shorten_path::shorten_path;
use crate::vertex_sequence::{VecSequence, VertexDist};
use crate::{Generator, VertexSequence, VertexSource};

//============================================================VcgenStroke

#[derive(Clone, Copy, PartialEq, Eq)]
enum StatusE {
    Initial,
    Ready,
    Cap1,
    Cap2,
    Outline1,
    CloseFirst,
    Outline2,
    OutVertices,
    EndPoly1,
    EndPoly2,
    Stop,
}
pub type VertexStorage = VecSequence<VertexDist>;
pub type CoordStorage = VecPodB<PointD>;

pub struct VcgenStroke {
    m_stroker: MathStroke<CoordStorage>,
    m_src_vertices: VertexStorage,
    m_out_vertices: CoordStorage,
    m_shorten: f64,
    m_closed: u32,
    m_status: StatusE,
    m_prev_status: StatusE,
    m_src_vertex: usize,
    m_out_vertex: usize,
}

impl Generator for VcgenStroke {
	fn new() -> VcgenStroke {
        VcgenStroke {
            m_stroker: MathStroke::new(),
            m_src_vertices: VecSequence::new(),
            m_out_vertices: VecPodB::new(),
            m_shorten: 0.0,
            m_closed: 0,
            m_status: StatusE::Initial,
            m_prev_status: StatusE::Initial,
            m_src_vertex: 0,
            m_out_vertex: 0,
        }
    }
	
    // Vertex Generator Interface
    fn remove_all(&mut self) {
        self.m_src_vertices.remove_all();
        self.m_closed = 0;
        self.m_status = StatusE::Initial;
    }
	
    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        self.m_status = StatusE::Initial;
        if is_move_to(cmd) {
            self.m_src_vertices.modify_last(VertexDist::new(x, y));
        } else {
            if is_vertex(cmd) {
                self.m_src_vertices.add(VertexDist::new(x, y));
            } else {
                self.m_closed = get_close_flag(cmd);
            }
        }
    }
}

impl VertexSource for VcgenStroke {
    // Vertex Source Interface
    fn rewind(&mut self, _: u32) {
        if self.m_status == StatusE::Initial {
            self.m_src_vertices.close(self.m_closed != 0);
            shorten_path(&mut self.m_src_vertices, self.m_shorten, self.m_closed);
            if self.m_src_vertices.size() < 3 {
                self.m_closed = 0;
            }
        }
        self.m_status = StatusE::Ready;
        self.m_src_vertex = 0;
        self.m_out_vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::LineTo;
        while !is_stop(cmd as u32) {
            match self.m_status {
                StatusE::Initial => {
                    self.rewind(0);
                    self.m_status = StatusE::Ready;
                }
                StatusE::Ready => {
                    if self.m_src_vertices.size() < 2 + (self.m_closed != 0) as usize {
                        cmd = PathCmd::Stop;
                    } else {
                        self.m_status = if self.m_closed != 0 {
                            StatusE::Outline1
                        } else {
                            StatusE::Cap1
                        };
                        cmd = PathCmd::MoveTo;
                        self.m_src_vertex = 0;
                        self.m_out_vertex = 0;
                    }
                }
                StatusE::Cap1 => {
                    self.m_stroker.calc_cap(
                        &mut self.m_out_vertices,
                        &self.m_src_vertices[0],
                        &self.m_src_vertices[1],
                        self.m_src_vertices[0].dist,
                    );
                    self.m_src_vertex = 1;
                    self.m_prev_status = StatusE::Outline1;
                    self.m_status = StatusE::OutVertices;
                    self.m_out_vertex = 0;
                }
                StatusE::Cap2 => {
                    self.m_stroker.calc_cap(
                        &mut self.m_out_vertices,
                        &self.m_src_vertices[self.m_src_vertices.size() - 1],
                        &self.m_src_vertices[self.m_src_vertices.size() - 2],
                        self.m_src_vertices[self.m_src_vertices.size() - 2].dist,
                    );
                    self.m_prev_status = StatusE::Outline2;
                    self.m_status = StatusE::OutVertices;
                    self.m_out_vertex = 0;
                }
                StatusE::Outline1 => {
                    if self.m_closed != 0 && self.m_src_vertex >= self.m_src_vertices.size() {
                        self.m_prev_status = StatusE::CloseFirst;
                        self.m_status = StatusE::EndPoly1;
                    } else if self.m_closed == 0
                        && self.m_src_vertex >= self.m_src_vertices.size() - 1
                    {
                        self.m_status = StatusE::Cap2;
                    } else {
                        self.m_stroker.calc_join(
                            &mut self.m_out_vertices,
                            self.m_src_vertices.prev(self.m_src_vertex),
                            self.m_src_vertices.curr(self.m_src_vertex),
                            self.m_src_vertices.next(self.m_src_vertex),
                            self.m_src_vertices.prev(self.m_src_vertex).dist,
                            self.m_src_vertices.curr(self.m_src_vertex).dist,
                        );
                        self.m_src_vertex += 1;
                        self.m_prev_status = self.m_status;
                        self.m_status = StatusE::OutVertices;
                        self.m_out_vertex = 0;
                    }
                }
                StatusE::CloseFirst => {
                    self.m_status = StatusE::Outline2;
                    cmd = PathCmd::MoveTo;
                }
                StatusE::Outline2 => {
                    if self.m_src_vertex <= (self.m_closed == 0) as usize {
                        self.m_status = StatusE::EndPoly2;
                        self.m_prev_status = StatusE::Stop;
                    } else {
                        self.m_src_vertex -= 1;
                        self.m_stroker.calc_join(
                            &mut self.m_out_vertices,
                            self.m_src_vertices.next(self.m_src_vertex),
                            self.m_src_vertices.curr(self.m_src_vertex),
                            self.m_src_vertices.prev(self.m_src_vertex),
                            self.m_src_vertices.curr(self.m_src_vertex).dist,
                            self.m_src_vertices.prev(self.m_src_vertex).dist,
                        );
                        self.m_prev_status = self.m_status;
                        self.m_status = StatusE::OutVertices;
                        self.m_out_vertex = 0;
                    }
                }
                StatusE::OutVertices => {
                    if self.m_out_vertex as usize >= self.m_out_vertices.len() {
                        self.m_status = self.m_prev_status;
                    } else {
                        let c = self.m_out_vertices[self.m_out_vertex];
                        self.m_out_vertex += 1;
                        *x = c.x;
                        *y = c.y;
                        return cmd as u32;
                    }
                }
                StatusE::EndPoly1 => {
                    self.m_status = self.m_prev_status;
                    return PathCmd::EndPoly as u32 | PathFlag::Close as u32 | PathFlag::Ccw as u32;
                }
                StatusE::EndPoly2 => {
                    self.m_status = self.m_prev_status;
                    return PathCmd::EndPoly as u32 | PathFlag::Close as u32 | PathFlag::Cw as u32;
                }
                StatusE::Stop => {
                    cmd = PathCmd::Stop;
                }
            }
        }
        cmd as u32
    }
}

impl VcgenStroke {
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
    pub fn set_width(&mut self, w: f64) {
        self.m_stroker.set_width(w);
    }
    pub fn set_miter_limit(&mut self, ml: f64) {
        self.m_stroker.set_miter_limit(ml);
    }
    pub fn set_miter_limit_theta(&mut self, t: f64) {
        self.m_stroker.set_miter_limit_theta(t);
    }
    pub fn set_inner_miter_limit(&mut self, ml: f64) {
        self.m_stroker.set_inner_miter_limit(ml);
    }
    pub fn set_approximation_scale(&mut self, a: f64) {
        self.m_stroker.set_approximation_scale(a);
    }
    pub fn width(&self) -> f64 {
        self.m_stroker.width()
    }
    pub fn miter_limit(&self) -> f64 {
        self.m_stroker.miter_limit()
    }
    pub fn inner_miter_limit(&self) -> f64 {
        self.m_stroker.inner_miter_limit()
    }
    pub fn approximation_scale(&self) -> f64 {
        self.m_stroker.approximation_scale()
    }
    pub fn set_shorten(&mut self, s: f64) {
        self.m_shorten = s;
    }
    pub fn shorten(&self) -> f64 {
        self.m_shorten
    }
}

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
//
// Line dash generator
//
//----------------------------------------------------------------------------
use crate::array::{PodBVector, VecPodB};
use crate::basics::{get_close_flag, is_move_to, is_stop, is_vertex, PathCmd, PointD};
use crate::math_stroke::{InnerJoin, LineCap, LineJoin};
use crate::shorten_path::shorten_path;
use crate::vertex_sequence::{VecSequence, VertexDist};
use crate::{Generator, VertexSequence, VertexSource};

const MAX_DASHES: usize = 32;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Initial,
    Ready,
    Polyline,
    Stop,
}
pub type VertexStorage = VecSequence<VertexDist>;
pub type CoordStorage = VecPodB<PointD>;

pub struct VcgenDash {
    m_dashes: [f64; MAX_DASHES],
    m_total_dash_len: f64,
    m_num_dashes: usize,
    m_dash_start: f64,
    m_shorten: f64,
    m_curr_dash_start: f64,
    m_curr_rest: f64,
    m_curr_dash: usize,
    m_src_vertices: VertexStorage,
    m_closed: u32,
    m_status: Status,
    m_src_vertex: u32,
    m_v1: *const VertexDist,
    m_v2: *const VertexDist,
}

impl VcgenDash {
    pub fn remove_all_dashes(&mut self) {
        self.m_total_dash_len = 0.0;
        self.m_num_dashes = 0;
        self.m_curr_dash_start = 0.0;
        self.m_curr_dash = 0;
    }
    //------------------------------------------------------------------------
    pub fn add_dash(&mut self, dash_len: f64, gap_len: f64) {
        if self.m_num_dashes < MAX_DASHES {
            self.m_total_dash_len += dash_len + gap_len;
            self.m_dashes[self.m_num_dashes] = dash_len;
            self.m_num_dashes += 1;
            self.m_dashes[self.m_num_dashes] = gap_len;
            self.m_num_dashes += 1;
        }
    }
    //------------------------------------------------------------------------
    pub fn dash_start(&mut self, ds: f64) {
        self.m_dash_start = ds;
        self.calc_dash_start(ds.abs());
    }
    //------------------------------------------------------------------------
    pub fn calc_dash_start(&mut self, ds_: f64) {
        let mut ds = ds_;
        self.m_curr_dash = 0;
        self.m_curr_dash_start = 0.0;
        while ds > 0.0 {
            if ds > self.m_dashes[self.m_curr_dash] {
                ds -= self.m_dashes[self.m_curr_dash];
                self.m_curr_dash += 1;
                self.m_curr_dash_start = 0.0;
                if self.m_curr_dash >= self.m_num_dashes {
                    self.m_curr_dash = 0;
                }
            } else {
                self.m_curr_dash_start = ds;
                ds = 0.0;
            }
        }
    }

    pub fn get_line_cap(&self) -> LineCap {
        LineCap::Round
    }
    pub fn get_line_join(&self) -> LineJoin {
        LineJoin::Miter
    }
    pub fn get_inner_join(&self) -> InnerJoin {
        InnerJoin::Miter
    }

    pub fn shorten(&mut self, s: f64) {
        self.m_shorten = s;
    }
    pub fn get_shorten(&self) -> f64 {
        self.m_shorten
    }
}

impl Generator for VcgenDash {
    fn new() -> VcgenDash {
        VcgenDash {
            m_dashes: [0.0; MAX_DASHES],
            m_total_dash_len: 0.0,
            m_num_dashes: 0,
            m_dash_start: 0.0,
            m_shorten: 0.0,
            m_curr_dash_start: 0.0,
            m_curr_rest: 0.0,
            m_curr_dash: 0,
            m_src_vertices: VecSequence::new(),
            m_closed: 0,
            m_status: Status::Initial,
            m_src_vertex: 0,
            m_v1: std::ptr::null(),
            m_v2: std::ptr::null(),
        }
    }

    // Vertex Generator Interface
    fn remove_all(&mut self) {
        self.m_src_vertices.remove_all();
        self.m_closed = 0;
        self.m_status = Status::Initial;
    }
    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        self.m_status = Status::Initial;
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

impl VertexSource for VcgenDash {
    fn rewind(&mut self, _: u32) {
        if self.m_status == Status::Initial {
            self.m_src_vertices.close(self.m_closed != 0);
            shorten_path(&mut self.m_src_vertices, self.m_shorten, self.m_closed);
        }
        self.m_status = Status::Ready;
        self.m_src_vertex = 0;
    }
    //------------------------------------------------------------------------
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::MoveTo;
        while !is_stop(cmd as u32) {
            match self.m_status {
                Status::Initial => {
                    self.rewind(0);
                }
                Status::Ready => {
                    if self.m_num_dashes < 2 || self.m_src_vertices.size() < 2 {
                        cmd = PathCmd::Stop;
                        break;
                    }
                    self.m_status = Status::Polyline;
                    self.m_src_vertex = 1;
                    self.m_v1 = &self.m_src_vertices[0];
                    self.m_v2 = &self.m_src_vertices[1];
                    unsafe {
                        self.m_curr_rest = (*self.m_v1).dist;
                        *x = (*self.m_v1).x;
                        *y = (*self.m_v1).y;
                    }
                    if self.m_dash_start >= 0.0 {
                        self.calc_dash_start(self.m_dash_start);
                    }
                    return PathCmd::MoveTo as u32;
                }
                Status::Polyline => {
                    let dash_rest = self.m_dashes[self.m_curr_dash] - self.m_curr_dash_start;
                    let cmd = if self.m_curr_dash & 1 == 1 {
                        PathCmd::MoveTo
                    } else {
                        PathCmd::LineTo
                    };
                    if self.m_curr_rest > dash_rest {
                        self.m_curr_rest -= dash_rest;
                        self.m_curr_dash += 1;
                        if self.m_curr_dash >= self.m_num_dashes {
                            self.m_curr_dash = 0;
                        }
                        self.m_curr_dash_start = 0.0;
                        unsafe {
                            *x = (*self.m_v2).x
                                - ((*self.m_v2).x - (*self.m_v1).x) * self.m_curr_rest
                                    / (*self.m_v1).dist;
                            *y = (*self.m_v2).y
                                - ((*self.m_v2).y - (*self.m_v1).y) * self.m_curr_rest
                                    / (*self.m_v1).dist;
                        }
                    } else {
                        self.m_curr_dash_start += self.m_curr_rest;
                        unsafe {
                            *x = (*self.m_v2).x;
                            *y = (*self.m_v2).y;
                            self.m_src_vertex += 1;
                            self.m_v1 = self.m_v2;
                            self.m_curr_rest = (*self.m_v1).dist;
                        }
                        if self.m_closed != 0 {
                            if self.m_src_vertex as usize > self.m_src_vertices.size() {
                                self.m_status = Status::Stop;
                            } else {
                                let ind =
                                    if self.m_src_vertex as usize >= self.m_src_vertices.size() {
                                        0
                                    } else {
                                        self.m_src_vertex
                                    };
                                self.m_v2 = &self.m_src_vertices[ind as usize];
                            }
                        } else {
                            if self.m_src_vertex as usize >= self.m_src_vertices.size() {
                                self.m_status = Status::Stop;
                            } else {
                                self.m_v2 = &self.m_src_vertices[self.m_src_vertex as usize];
                            }
                        }
                    }
                    return cmd as u32;
                }
                Status::Stop => {
                    cmd = PathCmd::Stop;
                }
            }
        }
        cmd as u32 //PathCmd::Stop as u32
    }
}

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
// Stroke math
//
//----------------------------------------------------------------------------

use std::marker::PhantomData;

use crate::math::{calc_distance, calc_intersection, cross_product};
use crate::vertex_sequence::VertexDist;
use crate::{Point, VertexConsumer};

//-------------------------------------------------------------LineCap
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    Butt,
    Square,
    Round,
}

//------------------------------------------------------------LineJoin
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    Miter = 0,
    MiterRevert = 1,
    Round = 2,
    Bevel = 3,
    MiterRound = 4,
}

//-----------------------------------------------------------InnerJoin
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InnerJoin {
    Bevel,
    Miter,
    Jag,
    Round,
}

//------------------------------------------------------------MathStroke
pub struct MathStroke<VC>
where
    VC: VertexConsumer,
    VC::ValueType: Point<T = f64>,
{
    m_line_cap: LineCap,
    m_line_join: LineJoin,
    m_inner_join: InnerJoin,
    m_miter_limit: f64,
    m_inner_miter_limit: f64,
    m_approx_scale: f64,
    m_width: f64,
    m_width_abs: f64,
    m_width_eps: f64,
    m_width_sign: i32,
    m_dum: PhantomData<VC>,
}

impl<VC> MathStroke<VC>
where
    VC: VertexConsumer,
    VC::ValueType: Point<T = f64>,
{
    pub fn new() -> Self {
        MathStroke {
            m_line_cap: LineCap::Butt,
            m_line_join: LineJoin::Miter,
            m_inner_join: InnerJoin::Miter,
            m_miter_limit: 4.0,
            m_inner_miter_limit: 1.01,
            m_approx_scale: 1.0,
            m_width: 0.5,
            m_width_abs: 0.5,
            m_width_eps: 0.5/1024.0,
            m_width_sign: 1,
            m_dum: PhantomData,
        }
    }
    pub fn set_line_cap(&mut self, lc: LineCap) {
        self.m_line_cap = lc;
    }
    pub fn set_line_join(&mut self, lj: LineJoin) {
        self.m_line_join = lj;
    }
    pub fn set_inner_join(&mut self, ij: InnerJoin) {
        self.m_inner_join = ij;
    }
    pub fn line_cap(&self) -> LineCap {
        self.m_line_cap
    }
    pub fn line_join(&self) -> LineJoin {
        self.m_line_join
    }
    pub fn inner_join(&self) -> InnerJoin {
        self.m_inner_join
    }
    pub fn set_miter_limit(&mut self, ml: f64) {
        self.m_miter_limit = ml;
    }
    pub fn set_inner_miter_limit(&mut self, ml: f64) {
        self.m_inner_miter_limit = ml;
    }
    pub fn set_approximation_scale(&mut self, a: f64) {
        self.m_approx_scale = a;
    }
    pub fn width(&self) -> f64 {
        self.m_width * 2.0
    }
    pub fn miter_limit(&self) -> f64 {
        self.m_miter_limit
    }
    pub fn inner_miter_limit(&self) -> f64 {
        self.m_inner_miter_limit
    }
    pub fn approximation_scale(&self) -> f64 {
        self.m_approx_scale
    }
    #[inline]
    fn add_vertex(&self, vc: &mut VC, x: f64, y: f64) {
        vc.add(VC::ValueType::new(x, y));
    }

    pub fn set_width(&mut self, w: f64) {
        self.m_width = w * 0.5;
        if self.m_width < 0.0 {
            self.m_width_abs = -self.m_width;
            self.m_width_sign = -1;
        } else {
            self.m_width_abs = self.m_width;
            self.m_width_sign = 1;
        }
        self.m_width_eps = self.m_width / 1024.0;
    }

    pub fn set_miter_limit_theta(&mut self, t: f64) {
        self.m_miter_limit = 1.0 / (t * 0.5).sin();
    }

    pub fn calc_arc(&self, vc: &mut VC, x: f64, y: f64, dx1: f64, dy1: f64, dx2: f64, dy2: f64) {
        let mut a1 = (dy1 * self.m_width_sign as f64).atan2(dx1 * self.m_width_sign as f64);
        let mut a2 = (dy2 * self.m_width_sign as f64).atan2(dx2 * self.m_width_sign as f64);
        let mut da;
        let n: i32;

        da = (self.m_width_abs / (self.m_width_abs + 0.125 / self.m_approx_scale)).acos() * 2.0;

        self.add_vertex(vc, x + dx1, y + dy1);
        if self.m_width_sign > 0 {
            if a1 > a2 {
                a2 += 2.0 * std::f64::consts::PI;
            }
            n = ((a2 - a1) / da) as i32;
            da = (a2 - a1) / (n + 1) as f64;
            a1 += da;
            for _ in 0..n {
                self.add_vertex(vc, x + a1.cos() * self.m_width, y + a1.sin() * self.m_width);
                a1 += da;
            }
        } else {
            if a1 < a2 {
                a2 -= 2.0 * std::f64::consts::PI;
            }
            n = ((a1 - a2) / da) as i32;
            da = (a1 - a2) / (n + 1) as f64;
            a1 -= da;
            for _ in 0..n {
                self.add_vertex(vc, x + a1.cos() * self.m_width, y + a1.sin() * self.m_width);
                a1 -= da;
            }
        }
        self.add_vertex(vc, x + dx2, y + dy2);
    }

    pub fn calc_miter(
        &mut self, vc: &mut VC, v0: &VertexDist, v1: &VertexDist, v2: &VertexDist, dx1: f64,
        dy1: f64, dx2: f64, dy2: f64, lj: LineJoin, mlimit: f64, dbevel: f64,
    ) {
        let mut xi = v1.x;
        let mut yi = v1.y;
        let mut di = 1.0;
        let lim = self.m_width_abs * mlimit;
        let mut miter_limit_exceeded = true; // Assume the worst
        let mut intersection_failed = true; // Assume the worst

        if calc_intersection(
            v0.x + dx1,
            v0.y - dy1,
            v1.x + dx1,
            v1.y - dy1,
            v1.x + dx2,
            v1.y - dy2,
            v2.x + dx2,
            v2.y - dy2,
            &mut xi,
            &mut yi,
        ) {
            // Calculation of the intersection succeeded
            di = calc_distance(v1.x, v1.y, xi, yi);
            if di <= lim {
                // Inside the miter limit
                //---------------------
                self.add_vertex(vc, xi, yi);
                miter_limit_exceeded = false;
            }
            intersection_failed = false;
        } else {
            // Calculation of the intersection failed, most probably
            // the three points lie one straight line.
            // First check if v0 and v2 lie on the opposite sides of vector:
            // (v1.x, v1.y) -> (v1.x+dx1, v1.y-dy1), that is, the perpendicular
            // to the line determined by vertices v0 and v1.
            // This condition determines whether the next line segments continues
            // the previous one or goes back.
            let x2 = v1.x + dx1;
            let y2 = v1.y - dy1;
            if (cross_product(v0.x, v0.y, v1.x, v1.y, x2, y2) < 0.0)
                == (cross_product(v1.x, v1.y, v2.x, v2.y, x2, y2) < 0.0)
            {
                self.add_vertex(vc, v1.x + dx1, v1.y - dy1);
                miter_limit_exceeded = false;
            }
        }

        if miter_limit_exceeded {
            // Miter limit exceeded
            //------------------------
            match lj {
                LineJoin::MiterRevert => {
                    // For the compatibility with SVG, PDF, etc,
                    // we use a simple bevel join instead of
                    // "smart" bevel
                    //-------------------
                    self.add_vertex(vc, v1.x + dx1, v1.y - dy1);
                    self.add_vertex(vc, v1.x + dx2, v1.y - dy2);
                }
                LineJoin::MiterRound => {
                    self.calc_arc(vc, v1.x, v1.y, dx1, -dy1, dx2, -dy2);
                }
                _ => {
                    // If no miter-revert, calculate new dx1, dy1, dx2, dy2
                    //----------------
                    if intersection_failed {
                        let mlimit = mlimit * self.m_width_sign as f64;
                        self.add_vertex(vc, v1.x + dx1 + dy1 * mlimit, v1.y - dy1 + dx1 * mlimit);
                        self.add_vertex(vc, v1.x + dx2 - dy2 * mlimit, v1.y - dy2 - dx2 * mlimit);
                    } else {
                        let x1 = v1.x + dx1;
                        let y1 = v1.y - dy1;
                        let x2 = v1.x + dx2;
                        let y2 = v1.y - dy2;
                        di = (lim - dbevel) / (di - dbevel);
                        self.add_vertex(vc, x1 + (xi - x1) * di, y1 + (yi - y1) * di);
                        self.add_vertex(vc, x2 + (xi - x2) * di, y2 + (yi - y2) * di);
                    }
                }
            }
        }
    }

    pub fn calc_cap(&self, vc: &mut VC, v0: &VertexDist, v1: &VertexDist, len: f64) {
        vc.remove_all();

        let dx1 = (v1.y - v0.y) / len;
        let dy1 = (v1.x - v0.x) / len;
        let mut dx2 = 0.0;
        let mut dy2 = 0.0;

        let dx1 = dx1 * self.m_width;
        let dy1 = dy1 * self.m_width;

        if self.m_line_cap != LineCap::Round {
            if self.m_line_cap == LineCap::Square {
                dx2 = dy1 * self.m_width_sign as f64;
                dy2 = dx1 * self.m_width_sign as f64;
            }
            self.add_vertex(vc, v0.x - dx1 - dx2, v0.y + dy1 - dy2);
            self.add_vertex(vc, v0.x + dx1 - dx2, v0.y - dy1 - dy2);
        } else {
            let da =
                (self.m_width_abs / (self.m_width_abs + 0.125 / self.m_approx_scale)).acos() * 2.0;
            let mut a1: f64;
            let n = (std::f64::consts::PI / da) as i32;

            let da = std::f64::consts::PI / (n + 1) as f64;
            self.add_vertex(vc, v0.x - dx1, v0.y + dy1);
            if self.m_width_sign > 0 {
                a1 = dy1.atan2(-dx1);
                a1 += da;
                for _ in 0..n {
                    self.add_vertex(
                        vc,
                        v0.x + a1.cos() * self.m_width,
                        v0.y + a1.sin() * self.m_width,
                    );
                    a1 += da;
                }
            } else {
                a1 = -dy1.atan2(dx1);
                a1 -= da;
                for _ in 0..n {
                    self.add_vertex(
                        vc,
                        v0.x + a1.cos() * self.m_width,
                        v0.y + a1.sin() * self.m_width,
                    );
                    a1 -= da;
                }
            }
            self.add_vertex(vc, v0.x + dx1, v0.y - dy1);
        }
    }

    pub fn calc_join(
        &mut self, vc: &mut VC, v0: &VertexDist, v1: &VertexDist, v2: &VertexDist, len1: f64,
        len2: f64,
    ) {
        let dx1 = self.m_width * (v1.y - v0.y) / len1;
        let dy1 = self.m_width * (v1.x - v0.x) / len1;
        let dx2 = self.m_width * (v2.y - v1.y) / len2;
        let dy2 = self.m_width * (v2.x - v1.x) / len2;

        vc.remove_all();

        let mut cp = cross_product(v0.x, v0.y, v1.x, v1.y, v2.x, v2.y);
        if cp != 0.0 && (cp > 0.0) == (self.m_width > 0.0) {
            // Inner join
            //---------------
            let mut limit = if len1 < len2 { len1 } else { len2 } / self.m_width_abs;
            if limit < self.m_inner_miter_limit {
                limit = self.m_inner_miter_limit;
            }

            match self.m_inner_join {
                InnerJoin::Bevel => {
                    self.add_vertex(vc, v1.x + dx1, v1.y - dy1);
                    self.add_vertex(vc, v1.x + dx2, v1.y - dy2);
                }
                InnerJoin::Miter => {
                    self.calc_miter(
                        vc,
                        v0,
                        v1,
                        v2,
                        dx1,
                        dy1,
                        dx2,
                        dy2,
                        LineJoin::MiterRevert,
                        limit,
                        0.0,
                    );
                }
                InnerJoin::Jag | InnerJoin::Round => {
                    cp = (dx1 - dx2) * (dx1 - dx2) + (dy1 - dy2) * (dy1 - dy2);
                    if cp < len1 * len1 && cp < len2 * len2 {
                        self.calc_miter(
                            vc,
                            v0,
                            v1,
                            v2,
                            dx1,
                            dy1,
                            dx2,
                            dy2,
                            LineJoin::MiterRevert,
                            limit,
                            0.,
                        );
                    } else {
                        if self.m_inner_join == InnerJoin::Jag {
                            self.add_vertex(vc, v1.x + dx1, v1.y - dy1);
                            self.add_vertex(vc, v1.x, v1.y);
                            self.add_vertex(vc, v1.x + dx2, v1.y - dy2);
                        } else {
                            self.add_vertex(vc, v1.x + dx1, v1.y - dy1);
                            self.add_vertex(vc, v1.x, v1.y);
                            self.calc_arc(vc, v1.x, v1.y, dx2, -dy2, dx1, -dy1);
                            self.add_vertex(vc, v1.x, v1.y);
                            self.add_vertex(vc, v1.x + dx2, v1.y - dy2);
                        }
                    }
                }
            }
        } else {
			// Outer join
            //---------------

            // Calculate the distance between v1 and 
            // the central point of the bevel line segment
            let mut dx = (dx1 + dx2) / 2.0;
            let mut dy = (dy1 + dy2) / 2.0;
            let dbevel = (dx * dx + dy * dy).sqrt();

            if self.m_line_join == LineJoin::Round || self.m_line_join == LineJoin::Bevel {
                // This is an optimization that reduces the number of points 
                // in cases of almost collinear segments. If there's no
                // visible difference between bevel and miter joins we'd rather
                // use miter join because it adds only one point instead of two. 
                //
                // Here we calculate the middle point between the bevel points 
                // and then, the distance between v1 and this middle point. 
                // At outer joins this distance always less than stroke width, 
                // because it's actually the height of an isosceles triangle of
                // v1 and its two bevel points. If the difference between this
                // width and this value is small (no visible bevel) we can 
                // add just one point. 
                //
                // The constant in the expression makes the result approximately 
                // the same as in round joins and caps. You can safely comment 
                // out this entire "if".

				if self.m_approx_scale * (self.m_width_abs - dbevel) < self.m_width_eps {
                    if calc_intersection(
                        v0.x + dx1,
                        v0.y - dy1,
                        v1.x + dx1,
                        v1.y - dy1,
                        v1.x + dx2,
                        v1.y - dy2,
                        v2.x + dx2,
                        v2.y - dy2,
                        &mut dx,
                        &mut dy,
                    ) {
                        self.add_vertex(vc, dx, dy);
                    } else {
                        self.add_vertex(vc, v1.x + dx1, v1.y - dy1);
                    }
                    return;
                }
            }

            match self.m_line_join {
                LineJoin::Miter | LineJoin::MiterRevert | LineJoin::MiterRound => {
                    self.calc_miter(
                        vc,
                        v0,
                        v1,
                        v2,
                        dx1,
                        dy1,
                        dx2,
                        dy2,
                        self.m_line_join,
                        self.m_miter_limit,
                        dbevel,
                    );
                }
                
                LineJoin::Round => {
                    self.calc_arc(vc, v1.x, v1.y, dx1, -dy1, dx2, -dy2);
                }
                LineJoin::Bevel => {
                    self.add_vertex(vc, v1.x + dx1, v1.y - dy1);
                    self.add_vertex(vc, v1.x + dx2, v1.y - dy2);
                }
            }
        }
    }
}

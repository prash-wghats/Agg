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
// class ellipse
//
//----------------------------------------------------------------------------

use crate::basics::PathCmd;
use crate::trans_affine::TransAffine;
use crate::{Transformer, VertexSource};
//----------------------------------------------------------------------------
//
// Arc generator. Produces at most 4 consecutive cubic bezier curves, i.e.,
// 4, 7, 10, or 13 vertices.
//
//----------------------------------------------------------------------------

// This epsilon is used to prevent us from adding degenerate curves
// (converging to a single point).
// The value isn't very critical. Function arc_to_bezier() has a limit
// of the sweep_angle. If fabs(sweep_angle) exceeds std::f64::consts::PI/2 the curve
// becomes inaccurate. But slight exceeding is quite appropriate.
//-------------------------------------------------ANGLE_EPSILON
const ANGLE_EPSILON: f64 = 0.01;

//------------------------------------------------------------arc_to_bezier
fn arc_to_bezier(
    cx: f64, cy: f64, rx: f64, ry: f64, start_angle: f64, sweep_angle: f64, curve: &mut [f64],
) {
    let x0 = sweep_angle.cos() / 2.0;
    let y0 = sweep_angle.sin() / 2.0;
    let tx = (1.0 - x0) * 4.0 / 3.0;
    let ty = y0 - tx * x0 / y0;
    let px = [x0, x0 + tx, x0 + tx, x0];
    let py = [-y0, -ty, ty, y0];

    let sn = (start_angle + sweep_angle / 2.0).sin();
    let cs = (start_angle + sweep_angle / 2.0).cos();

    for i in 0..4 {
        curve[i * 2] = cx + rx * (px[i] * cs - py[i] * sn);
        curve[i * 2 + 1] = cy + ry * (px[i] * sn + py[i] * cs);
    }
}

//==============================================================BezierArc
pub struct BezierArc {
    pub m_vertex: u32,
    pub m_num_vertices: u32,
    pub m_cmd: u32,
    pub m_vertices: [f64; 26],
}

impl VertexSource for BezierArc {
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.m_vertex >= self.m_num_vertices {
            return PathCmd::Stop as u32;
        }
        *x = self.m_vertices[self.m_vertex as usize];
        *y = self.m_vertices[(self.m_vertex + 1) as usize];
        self.m_vertex += 2;
        if self.m_vertex == 2 {
            PathCmd::MoveTo as u32
        } else {
            self.m_cmd
        }
    }

    fn rewind(&mut self, _: u32) {
        self.m_vertex = 0;
    }
}

impl BezierArc {
    pub fn new() -> BezierArc {
        BezierArc {
            m_vertex: 26,
            m_num_vertices: 0,
            m_cmd: PathCmd::LineTo as u32,
            m_vertices: [0.0; 26],
        }
    }
    pub fn new_with_params(
        x: f64, y: f64, rx: f64, ry: f64, start_angle: f64, sweep_angle: f64,
    ) -> BezierArc {
        let mut b = BezierArc::new();
        b.init(x, y, rx, ry, start_angle, sweep_angle);
        b
    }
    pub fn num_vertices(&self) -> u32 {
        self.m_num_vertices
    }
    pub fn vertices(&self) -> &[f64] {
        &self.m_vertices
    }
    fn vertices_mut(&mut self) -> &mut [f64] {
        &mut self.m_vertices
    }

    // Supplemantary functions. num_vertices() actually returns doubled
    // number of vertices. That is, for 1 vertex it returns 2.
    //------------------------------------------------------------------------
    fn init(&mut self, x: f64, y: f64, rx: f64, ry: f64, start_angle: f64, sweep_angle: f64) {
        let mut start_angle = start_angle % (2.0 * std::f64::consts::PI);
        let mut sweep_angle = sweep_angle;
        if sweep_angle >= 2.0 * std::f64::consts::PI {
            sweep_angle = 2.0 * std::f64::consts::PI;
        }
        if sweep_angle <= -2.0 * std::f64::consts::PI {
            sweep_angle = -2.0 * std::f64::consts::PI;
        }

        if sweep_angle.abs() < 1e-10 {
            self.m_num_vertices = 4;
            self.m_cmd = PathCmd::LineTo as u32;
            self.m_vertices[0] = x + rx * start_angle.cos();
            self.m_vertices[1] = y + ry * start_angle.sin();
            self.m_vertices[2] = x + rx * (start_angle + sweep_angle).cos();
            self.m_vertices[3] = y + ry * (start_angle + sweep_angle).sin();
            return;
        }

        let mut total_sweep = 0.0;
        let mut local_sweep;
        let mut prev_sweep;
        self.m_num_vertices = 2;
        self.m_cmd = PathCmd::Curve4 as u32;
        let mut done = false;
        loop {
            if sweep_angle < 0.0 {
                prev_sweep = total_sweep;
                local_sweep = -std::f64::consts::PI * 0.5;
                total_sweep -= std::f64::consts::PI * 0.5;
                if total_sweep <= sweep_angle + ANGLE_EPSILON {
                    local_sweep = sweep_angle - prev_sweep;
                    done = true;
                }
            } else {
                prev_sweep = total_sweep;
                local_sweep = std::f64::consts::PI * 0.5;
                total_sweep += std::f64::consts::PI * 0.5;
                if total_sweep >= sweep_angle - ANGLE_EPSILON {
                    local_sweep = sweep_angle - prev_sweep;
                    done = true;
                }
            }
            let len = self.m_vertices.len();
            arc_to_bezier(
                x,
                y,
                rx,
                ry,
                start_angle,
                local_sweep,
                &mut self.m_vertices[(self.m_num_vertices - 2) as usize..len],
            );

            self.m_num_vertices += 6;
            start_angle += local_sweep;
            if done || self.m_num_vertices >= 26 {
                break;
            }
        }
    }
}

//==========================================================BezierArcSvg
// Compute an SVG-style bezier arc.
//
// Computes an elliptical arc from (x1, y1) to (x2, y2). The size and
// orientation of the ellipse are defined by two radii (rx, ry)
// and an x-axis-rotation, which indicates how the ellipse as a whole
// is rotated relative to the current coordinate system. The center
// (cx, cy) of the ellipse is calculated automatically to satisfy the
// constraints imposed by the other parameters.
// large-arc-flag and sweep-flag contribute to the automatic calculations
// and help determine how the arc is drawn.
pub struct BezierArcSvg {
    pub m_arc: BezierArc,
    pub m_radii_ok: bool,
}
impl VertexSource for BezierArcSvg {
    fn rewind(&mut self, _: u32) {
        self.m_arc.rewind(0);
    }
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.m_arc.vertex(x, y)
    }
}
impl BezierArcSvg {
    pub fn new() -> BezierArcSvg {
        BezierArcSvg {
            m_arc: BezierArc::new(),
            m_radii_ok: false,
        }
    }
    pub fn new_with_params(
        x1: f64, y1: f64, rx: f64, ry: f64, angle: f64, large_arc_flag: bool, sweep_flag: bool,
        x2: f64, y2: f64,
    ) -> BezierArcSvg {
        let mut b = BezierArcSvg {
            m_arc: BezierArc::new(),
            m_radii_ok: false,
        };
        b.init(x1, y1, rx, ry, angle, large_arc_flag, sweep_flag, x2, y2);
        b
    }
    pub fn radii_ok(&self) -> bool {
        self.m_radii_ok
    }

    // Supplemantary functions. num_vertices() actually returns doubled
    // number of vertices. That is, for 1 vertex it returns 2.
    //--------------------------------------------------------------------
    pub fn num_vertices(&self) -> u32 {
        self.m_arc.num_vertices()
    }
    pub fn vertices(&self) -> &[f64] {
        self.m_arc.vertices()
    }
    pub fn vertices_mut(&mut self) -> &mut [f64] {
        self.m_arc.vertices_mut()
    }

    pub fn init(
        &mut self, x0: f64, y0: f64, rx_: f64, ry_: f64, angle: f64, large_arc_flag: bool,
        sweep_flag: bool, x2: f64, y2: f64,
    ) {
        self.m_radii_ok = true;
        let (mut rx, mut ry) = (rx_, ry_);
        if rx < 0.0 {
            rx = -rx;
        }
        if ry < 0.0 {
            ry = -rx;
        }

        // Calculate the middle point between
        // the current and the final points
        //------------------------
        let dx2 = (x0 - x2) / 2.0;
        let dy2 = (y0 - y2) / 2.0;

        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Calculate (x1, y1)
        //------------------------
        let x1 = cos_a * dx2 + sin_a * dy2;
        let y1 = -sin_a * dx2 + cos_a * dy2;

        // Ensure radii are large enough
        //------------------------
        let mut prx = rx * rx;
        let mut pry = ry * ry;
        let px1 = x1 * x1;
        let py1 = y1 * y1;

        // Check that radii are large enough
        //------------------------
        let radii_check = px1 / prx + py1 / pry;
        if radii_check > 1.0 {
            rx = (radii_check * rx).sqrt();
            ry = (radii_check * ry).sqrt();
            prx = rx * rx;
            pry = ry * ry;
            if radii_check > 10.0 {
                self.m_radii_ok = false;
            }
        }

        // Calculate (cx1, cy1)
        //------------------------
        let sign = if large_arc_flag == sweep_flag {
            -1.0
        } else {
            1.0
        };
        let sq = (prx * pry - prx * py1 - pry * px1) / (prx * py1 + pry * px1);
        let coef = sign * (if sq < 0.0 { 0.0 } else { sq }).sqrt();
        let cx1 = coef * ((rx * y1) / ry);
        let cy1 = coef * -((ry * x1) / rx);

        //
        // Calculate (cx, cy) from (cx1, cy1)
        //------------------------
        let sx2 = (x0 + x2) / 2.0;
        let sy2 = (y0 + y2) / 2.0;
        let cx = sx2 + (cos_a * cx1 - sin_a * cy1);
        let cy = sy2 + (sin_a * cx1 + cos_a * cy1);

        // Calculate the start_angle (angle1) and the sweep_angle (dangle)
        //------------------------
        let ux = (x1 - cx1) / rx;
        let uy = (y1 - cy1) / ry;
        let vx = (-x1 - cx1) / rx;
        let vy = (-y1 - cy1) / ry;
        let (mut p, mut n);

        // Calculate the angle start
        //------------------------
        n = (ux * ux + uy * uy).sqrt();
        p = ux; // (1 * ux) + (0 * uy)
        let sign = if uy < 0.0 { -1.0 } else { 1.0 };
        let mut v = p / n;
        if v < -1.0 {
            v = -1.0;
        }
        if v > 1.0 {
            v = 1.0;
        }
        let start_angle = sign * v.acos();

        // Calculate the sweep angle
        //------------------------
        n = ((ux * ux + uy * uy) * (vx * vx + vy * vy)).sqrt();
        p = ux * vx + uy * vy;
        let sign = if ux * vy - uy * vx < 0.0 { -1.0 } else { 1.0 };
        v = p / n;
        if v < -1.0 {
            v = -1.0;
        }
        if v > 1.0 {
            v = 1.0;
        }
        let mut sweep_angle = sign * v.acos();
        if !sweep_flag && sweep_angle > 0.0 {
            sweep_angle -= std::f64::consts::PI * 2.0;
        } else if sweep_flag && sweep_angle < 0.0 {
            sweep_angle += std::f64::consts::PI * 2.0;
        }

        // We can now build and transform the resulting arc
        //------------------------
        self.m_arc.init(0.0, 0.0, rx, ry, start_angle, sweep_angle);
        let mut mtx = TransAffine::trans_affine_rotation(angle);
        mtx *= TransAffine::trans_affine_translation(cx, cy);

        let len = self.m_arc.num_vertices();
        let aref = self.m_arc.vertices_mut();
        let (mut x, mut y);
        for i in 2..len as usize - 2 {
            x = aref[i];
            y = aref[i + 1];
            mtx.transform(&mut x, &mut y);
        }

        // We must make sure that the starting and ending points
        // exactly coincide with the initial (x0,y0) and (x2,y2)
        aref[0] = x0;
        aref[1] = y0;
        if len > 2 {
            aref[len as usize - 2] = x2;
            aref[len as usize - 1] = y2;
        }
    }
}

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
// class Ellipse
//
//----------------------------------------------------------------------------

use crate::basics::{uround, PathCmd, PathFlag};
use crate::VertexSource;

//----------------------------------------------------------------Ellipse
pub struct Ellipse {
    m_x: f64,
    m_y: f64,
    m_rx: f64,
    m_ry: f64,
    m_scale: f64,
    m_num: u32,
    m_step: u32,
    m_cw: bool,
}

impl Ellipse {
    pub fn new() -> Ellipse {
        Ellipse {
            m_x: 0.0,
            m_y: 0.0,
            m_rx: 1.0,
            m_ry: 1.0,
            m_scale: 1.0,
            m_num: 4,
            m_step: 0,
            m_cw: false,
        }
    }
    pub fn new_ellipse(x: f64, y: f64, rx: f64, ry: f64, num_steps: u32, cw: bool) -> Ellipse {
        let mut e = Ellipse {
            m_x: x,
            m_y: y,
            m_rx: rx,
            m_ry: ry,
            m_scale: 1.0,
            m_num: num_steps,
            m_step: 0,
            m_cw: cw,
        };
        if num_steps == 0 {
            e.calc_num_steps();
        }
        e
    }
    pub fn init(&mut self, x: f64, y: f64, rx: f64, ry: f64, num_steps: u32, cw: bool) {
        self.m_x = x;
        self.m_y = y;
        self.m_rx = rx;
        self.m_ry = ry;
        self.m_num = num_steps;
        self.m_step = 0;
        self.m_cw = cw;
        if self.m_num == 0 {
            self.calc_num_steps();
        }
    }

    pub fn approximation_scale(&mut self, scale: f64) {
        self.m_scale = scale;
        self.calc_num_steps();
    }

    pub fn calc_num_steps(&mut self) {
        let ra = (self.m_rx.abs() + self.m_ry.abs()) / 2.0;
        let da = (ra / (ra + 0.125 / self.m_scale)).acos() * 2.0;
        self.m_num = uround(2. * std::f64::consts::PI / da) as u32;
    }
}

impl VertexSource for Ellipse {
    fn rewind(&mut self, _: u32) {
        self.m_step = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.m_step == self.m_num {
            self.m_step += 1;
            return PathCmd::EndPoly as u32 | PathFlag::Close as u32 | PathFlag::Ccw as u32;
        }
        if self.m_step > self.m_num {
            return PathCmd::Stop as u32;
        }
        let mut angle = self.m_step as f64 / self.m_num as f64 * 2.0 * std::f64::consts::PI;
        if self.m_cw {
            angle = 2.0 * std::f64::consts::PI - angle;
        }
        *x = self.m_x + angle.cos() * self.m_rx;
        *y = self.m_y + angle.sin() * self.m_ry;
        self.m_step += 1;
        return if self.m_step == 1 {
            PathCmd::MoveTo as u32
        } else {
            PathCmd::LineTo as u32
        };
    }
}

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
// classes Cbox, Cbox
//
//----------------------------------------------------------------------------

use std::ops::{Deref, DerefMut};

use crate::ctrl::CtrlBase;
use agg::basics::{is_stop, PathCmd};
use agg::color_rgba::Rgba;
use agg::math_stroke::{LineCap, LineJoin};
use agg::{Color, VertexSource};
use agg::{ConvStroke, GsvText};

use super::{Ctrl, CtrlColor};
//----------------------------------------------------------Cbox
pub struct Cbox<'a, C: Color> {
    ctrl: CtrlBase,
    text_thickness: f64,
    text_height: f64,
    text_width: f64,
    label: String,
    status: bool,
    vx: [f64; 32],
    vy: [f64; 32],
    //text: Rc<RefCell<GsvText>>,
    text_poly: ConvStroke<'a, GsvText>,
    idx: u32,
    vertex: u32,
    pub text_color: C,
    pub inactive_color: C,
    pub active_color: C,
}

//----------------------------------------------------------Cbox
impl<'a, C: Color> Cbox<'a, C> {
    pub fn new(x: f64, y: f64, l: &str, flip_y: bool) -> Self {
        Self {
            ctrl: CtrlBase::new(x, y, x + 9.0 * 1.5, y + 9.0 * 1.5, flip_y),
            text_thickness: 1.5,
            text_height: 9.0,
            text_width: 0.0,
            status: false,
            //text: mt.clone(),
            text_poly: ConvStroke::new_owned(GsvText::new()),
            label: l.to_string(),
            text_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            inactive_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            active_color: C::new_from_rgba(&Rgba::new_params(0.4, 0.0, 0.0, 1.0)),
            vx: [0.; 32],
            vy: [0.; 32],
            idx: 0,
            vertex: 0,
        }
    }

    pub fn set_text_thickness(&mut self, t: f64) {
        self.text_thickness = t;
    }

    pub fn set_text_size(&mut self, h: f64, w: f64) {
        self.text_height = h;
        self.text_width = w;
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn status(&self) -> bool {
        self.status
    }

    pub fn set_status(&mut self, st: bool) {
        self.status = st;
    }

    pub fn set_label(&mut self, l: &str) {
        if l.len() > 126 {
            self.label = l[0..126].to_string() + "\u{0}";
        } else {
            self.label = l.to_string() + "\u{0}";
        }
    }

    pub fn set_text_color(&mut self, c: C) {
        self.text_color = c;
    }

    pub fn set_inactive_color(&mut self, c: C) {
        self.inactive_color = c;
    }

    pub fn set_active_color(&mut self, c: C) {
        self.active_color = c;
    }
}

impl<'a, C: Color> Deref for Cbox<'a, C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<'a, C: Color> DerefMut for Cbox<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

impl<'a, C: Color> CtrlColor for Cbox<'a, C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            0 => self.inactive_color,
            1 => self.text_color,
            _ => self.active_color,
        }
    }
}

impl<'a, C: Color> Ctrl for Cbox<'a, C> {
    fn num_paths(&self) -> u32 {
        3
    }

    fn set_transform(&mut self, mtx: &agg::TransAffine) {
        self.ctrl.set_transform(&mtx);
    }

    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
        let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        if x >= self.x1 && y >= self.y1 && x <= self.x2 && y <= self.y2 {
            self.status = !self.status;
            return true;
        }
        return false;
    }

    fn on_mouse_move(&mut self, _: f64, _: f64, _: bool) -> bool {
        return false;
    }

    fn in_rect(&self, x: f64, y: f64) -> bool {
        let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        return x >= self.x1 && y >= self.y1 && x <= self.x2 && y <= self.y2;
    }

    fn on_mouse_button_up(&mut self, _: f64, _: f64) -> bool {
        return false;
    }

    fn on_arrow_keys(&mut self, _: bool, _: bool, _: bool, _: bool) -> bool {
        return false;
    }
}

impl<'a, C: Color> VertexSource for Cbox<'a, C> {
    fn rewind(&mut self, idx: u32) {
        self.idx = idx;

        match idx {
            0 => {
                // Border
                self.vertex = 0;
                self.vx[0] = self.x1;
                self.vy[0] = self.y1;
                self.vx[1] = self.x2;
                self.vy[1] = self.y1;
                self.vx[2] = self.x2;
                self.vy[2] = self.y2;
                self.vx[3] = self.x1;
                self.vy[3] = self.y2;
                self.vx[4] = self.x1 + self.text_thickness;
                self.vy[4] = self.y1 + self.text_thickness;
                self.vx[5] = self.x1 + self.text_thickness;
                self.vy[5] = self.y2 - self.text_thickness;
                self.vx[6] = self.x2 - self.text_thickness;
                self.vy[6] = self.y2 - self.text_thickness;
                self.vx[7] = self.x2 - self.text_thickness;
                self.vy[7] = self.y1 + self.text_thickness;
            }
            1 => {
                // Text
                self.text_poly.source_mut().set_text(&self.label);
				let (x, y) = (self.x1, self.y1);
                self.text_poly.source_mut().set_start_point(
                    x + self.text_height * 2.0,
                    y + self.text_height / 5.0,
                );
                self.text_poly
                    .source_mut()
                    .set_size(self.text_height, self.text_width);
                self.text_poly.set_width(self.text_thickness);
                self.text_poly.set_line_join(LineJoin::Round);
                self.text_poly.set_line_cap(LineCap::Round);
                self.text_poly.rewind(0);
            }
            2 => {
                // Active item
                self.vertex = 0;
                let d2 = (self.y2 - self.y1) / 2.0;
                let t = self.text_thickness * 1.5;
                self.vx[0] = self.x1 + self.text_thickness;
                self.vy[0] = self.y1 + self.text_thickness;
                self.vx[1] = self.x1 + d2;
                self.vy[1] = self.y1 + d2 - t;
                self.vx[2] = self.x2 - self.text_thickness;
                self.vy[2] = self.y1 + self.text_thickness;
                self.vx[3] = self.x1 + d2 + t;
                self.vy[3] = self.y1 + d2;
                self.vx[4] = self.x2 - self.text_thickness;
                self.vy[4] = self.y2 - self.text_thickness;
                self.vx[5] = self.x1 + d2;
                self.vy[5] = self.y1 + d2 + t;
                self.vx[6] = self.x1 + self.text_thickness;
                self.vy[6] = self.y2 - self.text_thickness;
                self.vx[7] = self.x1 + d2 - t;
                self.vy[7] = self.y1 + d2;
            }
            _ => {}
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::LineTo as u32;
        match self.idx {
            0 => {
                if self.vertex == 0 || self.vertex == 4 {
                    cmd = PathCmd::MoveTo as u32;
                }
                if self.vertex >= 8 {
                    cmd = PathCmd::Stop as u32;
                }
                *x = self.vx[self.vertex as usize];
                *y = self.vy[self.vertex as usize];
                self.vertex += 1;
            }
            1 => {
                cmd = self.text_poly.vertex(x, y);
            }
            2 => {
                if self.status {
                    if self.vertex == 0 {
                        cmd = PathCmd::MoveTo as u32;
                    }
                    if self.vertex >= 8 {
                        cmd = PathCmd::Stop as u32;
                    }
                    *x = self.vx[self.vertex as usize];
                    *y = self.vy[self.vertex as usize];
                    self.vertex += 1;
                } else {
                    cmd = PathCmd::Stop as u32;
                }
            }
            _ => {
                cmd = PathCmd::Stop as u32;
            }
        }
        if !is_stop(cmd) {
            self.transform_xy(x, y);
        }
        cmd
    }
}

//----------------------------------------------------------------------------
// Anti-Grain Geometry - Version 2.4
// Copyright (ColorT) 2002-2005 Maxim Shemanarev (http://www.antigrain.com)
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
// classes Slider, Slider
//
//----------------------------------------------------------------------------
use std::ops::{Deref, DerefMut};

use crate::ctrl::CtrlBase;
use agg::basics::{is_stop, PathCmd};
use agg::color_rgba::Rgba;
use agg::math::calc_distance;
use agg::math_stroke::{LineCap, LineJoin};
use agg::{Color, VertexSource};
use sprintf::sprintf;

use super::{Ctrl, CtrlColor};

pub struct Slider<'a, C: Color> {
    base: CtrlBase,
    border_width: f64,
    border_extra: f64,
    text_thickness: f64,
    value: f64,
    preview_value: f64,
    min: f64,
    max: f64,
    num_steps: u32,
    descending: bool,
    label: String,
    xs1: f64,
    ys1: f64,
    xs2: f64,
    ys2: f64,
    pdx: f64,
    mouse_move: bool,
    vx: [f64; 32],
    vy: [f64; 32],
    ellipse: agg::Ellipse,
    idx: u32,
    vertex: u32,
    text_poly: agg::ConvStroke<'a, agg::GsvText>,
    storage: agg::PathStorage,
    background_color: C,
    triangle_color: C,
    text_color: C,
    pointer_preview_color: C,
    pointer_color: C,
}

impl<'a,C: Color> Slider<'a,C> {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, flip_y: bool) -> Self {
        let mut s = Self {
            base: CtrlBase::new(x1, y1, x2, y2, flip_y),
            border_width: 1.0,
            border_extra: (y2 - y1) / 2.0,
            text_thickness: 1.0,
            pdx: 0.0,
            mouse_move: false,
            value: 0.5,
            preview_value: 0.5,
            min: 0.0,
            max: 1.0,
            num_steps: 0,
            descending: false,
            text_poly: agg::ConvStroke::new_owned(agg::GsvText::new()),
            label: "".to_string(),
            background_color: C::new_from_rgba(&Rgba::new_params(1.0, 0.9, 0.8, 1.0)),
            triangle_color: C::new_from_rgba(&Rgba::new_params(0.7, 0.6, 0.6, 1.0)),
            text_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            pointer_preview_color: C::new_from_rgba(&Rgba::new_params(0.6, 0.4, 0.4, 0.4)),
            pointer_color: C::new_from_rgba(&Rgba::new_params(0.8, 0.0, 0.0, 0.6)),

            xs1: 0.,
            ys1: 0.,
            xs2: 0.,
            ys2: 0.,

            vx: [0.; 32],
            vy: [0.; 32],
            idx: 0,
            vertex: 0,
            //m_text: mt,
            ellipse: agg::Ellipse::new(),
            storage: agg::PathStorage::new(),
        };
        s.calc_box();
        s
    }
    pub fn set_range(&mut self, min: f64, max: f64) {
        self.min = min;
        self.max = max;
    }

    pub fn set_num_steps(&mut self, num: u32) {
        self.num_steps = num;
    }

    pub fn set_text_thickness(&mut self, t: f64) {
        self.text_thickness = t;
    }

    pub fn descending(&self) -> bool {
        self.descending
    }

    pub fn set_descending(&mut self, v: bool) {
        self.descending = v;
    }

    pub fn value(&self) -> f64 {
        self.value * (self.max - self.min) + self.min
    }

    pub fn num_paths(&self) -> u32 {
        6
    }

    pub fn set_background_color(&mut self, c: &C) {
        self.background_color = *c;
    }

    pub fn set_pointer_color(&mut self, c: &C) {
        self.pointer_color = *c;
    }

    pub fn calc_box(&mut self) {
        self.xs1 = self.x1 + self.border_width;
        self.ys1 = self.y1 + self.border_width;
        self.xs2 = self.x2 - self.border_width;
        self.ys2 = self.y2 - self.border_width;
    }

    pub fn set_border_width(&mut self, t: f64, extra: f64) {
        self.border_width = t;
        self.border_extra = extra;
        self.calc_box();
    }

    fn normalize_value(&mut self, preview_value_flag: bool) -> bool {
        let mut ret = true;
        if self.num_steps != 0 {
            let step = (self.preview_value * self.num_steps as f64 + 0.5) as i32;
            ret = self.value != step as f64 / self.num_steps as f64;
            self.value = step as f64 / self.num_steps as f64;
        } else {
            self.value = self.preview_value;
        }

        if preview_value_flag {
            self.preview_value = self.value;
        }
        ret
    }

    pub fn set_value(&mut self, value: f64) {
        self.preview_value = (value - self.min) / (self.max - self.min);
        if self.preview_value > 1.0 {
            self.preview_value = 1.0;
        }
        if self.preview_value < 0.0 {
            self.preview_value = 0.0;
        }
        self.normalize_value(true);
    }

    pub fn set_label(&mut self, fmt: &str) {
        if fmt.len() > 63 {
            self.label = fmt[0..63].to_string();
        } else {
            self.label = fmt.to_string();
        }
    }
}

impl<'a,C: Color> Deref for Slider<'a,C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a,C: Color> DerefMut for Slider<'a,C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<'a,C: Color> CtrlColor for Slider<'a,C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            0 => self.background_color,
            1 => self.triangle_color,
            2 => self.text_color,
            3 => self.pointer_preview_color,
            4 => self.pointer_color,
            _ => self.text_color,
        }
    }
}

impl<'a,C: Color> Ctrl for Slider<'a,C> {
    fn num_paths(&self) -> u32 {
        6
    }
    fn set_transform(&mut self, mtx: &agg::TransAffine) {
        self.base.set_transform(&mtx);
    }
    fn in_rect(&self, x: f64, y: f64) -> bool {
        let mut x = x;
        let mut y = y;
        self.inverse_transform_xy(&mut x, &mut y);
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
        let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);

        let xp = self.xs1 + (self.xs2 - self.xs1) * self.value;
        let yp = (self.ys1 + self.ys2) / 2.0;

        if calc_distance(x, y, xp, yp) <= self.y2 - self.y1 {
            self.pdx = xp - x;
            self.mouse_move = true;
            return true;
        }
        return false;
    }

    fn on_mouse_move(&mut self, x: f64, y: f64, button_flag: bool) -> bool {
        let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        if !button_flag {
            self.on_mouse_button_up(x, y);
            return false;
        }

        if self.mouse_move {
            let xp = x + self.pdx;
            self.preview_value = (xp - self.xs1) / (self.xs2 - self.xs1);
            if self.preview_value < 0.0 {
                self.preview_value = 0.0;
            }
            if self.preview_value > 1.0 {
                self.preview_value = 1.0;
            }
            return true;
        }
        return false;
    }

    fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        self.mouse_move = false;
        self.normalize_value(true);
        return true;
    }

    fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool {
        let mut d = 0.005;
        if self.num_steps != 0 {
            d = 1.0 / self.num_steps as f64;
        }

        if right || up {
            self.preview_value += d;
            if self.preview_value > 1.0 {
                self.preview_value = 1.0;
            }
            self.normalize_value(true);
            return true;
        }

        if left || down {
            self.preview_value -= d;
            if self.preview_value < 0.0 {
                self.preview_value = 0.0;
            }
            self.normalize_value(true);
            return true;
        }
        return false;
    }
}

impl<'a,C: Color> VertexSource for Slider<'a,C> {
    fn rewind(&mut self, idx: u32) {
        self.idx = idx;

        match idx {
            0 => {
                self.vertex = 0;
                self.vx[0] = self.x1 - self.border_extra;
                self.vy[0] = self.y1 - self.border_extra;
                self.vx[1] = self.x2 + self.border_extra;
                self.vy[1] = self.y1 - self.border_extra;
                self.vx[2] = self.x2 + self.border_extra;
                self.vy[2] = self.y2 + self.border_extra;
                self.vx[3] = self.x1 - self.border_extra;
                self.vy[3] = self.y2 + self.border_extra;
            }
            1 => {
                self.vertex = 0;
                if self.descending {
                    self.vx[0] = self.x1;
                    self.vy[0] = self.y1;
                    self.vx[1] = self.x2;
                    self.vy[1] = self.y1;
                    self.vx[2] = self.x1;
                    self.vy[2] = self.y2;
                    self.vx[3] = self.x1;
                    self.vy[3] = self.y1;
                } else {
                    self.vx[0] = self.x1;
                    self.vy[0] = self.y1;
                    self.vx[1] = self.x2;
                    self.vy[1] = self.y1;
                    self.vx[2] = self.x2;
                    self.vy[2] = self.y2;
                    self.vx[3] = self.x1;
                    self.vy[3] = self.y1;
                }
            }
            2 => {
                self.text_poly.source_mut().set_text(&self.label);
                if !self.label.is_empty() {
                    let v = self.value();
                    let mut s0 = sprintf!(&self.label, v).unwrap();
                    s0 = s0 + "\u{0}";
                    self.text_poly.source_mut().set_text(&s0);
                }
                let (tx1, ty1, ty2) = (self.x1, self.y1, self.y2);
                self.text_poly.source_mut().set_start_point(tx1, ty1);

                self.text_poly
                    .source_mut()
                    .set_size((ty2 - ty1) * 1.2, ty2 - ty1);
                self.text_poly.set_width(self.text_thickness);
                self.text_poly.set_line_join(LineJoin::Round);
                self.text_poly.set_line_cap(LineCap::Round);
                self.text_poly.rewind(0);
            }
            3 => {
                self.ellipse.init(
                    self.xs1 + (self.xs2 - self.xs1) * self.preview_value,
                    (self.ys1 + self.ys2) / 2.0,
                    self.y2 - self.y1,
                    self.y2 - self.y1,
                    32,
                    false,
                );
            }
            4 => {
                self.normalize_value(false);
                self.ellipse.init(
                    self.xs1 + (self.xs2 - self.xs1) * self.value,
                    (self.ys1 + self.ys2) / 2.0,
                    self.y2 - self.y1,
                    self.y2 - self.y1,
                    32,
                    false,
                );
                self.ellipse.rewind(0);
            }
            5 => {
                self.storage.remove_all();
                if self.num_steps != 0 {
                    let mut d = (self.xs2 - self.xs1) / self.num_steps as f64;
                    if d > 0.004 {
                        d = 0.004;
                    }
                    for i in 0..self.num_steps + 1 {
                        let x = self.xs1 + (self.xs2 - self.xs1) * i as f64 / self.num_steps as f64;
                        self.storage.move_to(x, self.y1);
                        self.storage
                            .line_to(x - d * (self.x2 - self.x1), self.y1 - self.border_extra);
                        self.storage
                            .line_to(x + d * (self.x2 - self.x1), self.y1 - self.border_extra);
                    }
                }
            }
            _ => (),
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::LineTo as u32;
        match self.idx {
            0 => {
                if self.vertex == 0 {
                    cmd = PathCmd::MoveTo as u32;
                }
                if self.vertex >= 4 {
                    cmd = PathCmd::Stop as u32;
                }
                *x = self.vx[self.vertex as usize];
                *y = self.vy[self.vertex as usize];
                self.vertex += 1;
            }
            1 => {
                if self.vertex == 0 {
                    cmd = PathCmd::MoveTo as u32;
                }
                if self.vertex >= 4 {
                    cmd = PathCmd::Stop as u32;
                }
                *x = self.vx[self.vertex as usize];
                *y = self.vy[self.vertex as usize];
                self.vertex += 1;
            }
            2 => {
                cmd = self.text_poly.vertex(x, y);
            }
            3 | 4 => {
                cmd = self.ellipse.vertex(x, y);
            }
            5 => {
                cmd = self.storage.vertex(x, y);
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

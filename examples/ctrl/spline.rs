use std::ops::{Deref, DerefMut};

use crate::ctrl::CtrlBase;
use agg::basics::{is_stop, PathCmd};
use agg::color_rgba::Rgba;
use agg::math::calc_distance;
use agg::{Color, VertexSource};

use super::{Ctrl, CtrlColor};

use agg::bspline::*;
use agg::conv_stroke::ConvStroke;
use agg::ellipse::Ellipse;
use agg::path_storage::PathStorage;
use agg::trans_affine::TransAffine;
//------------------------------------------------------------------------
// Class that can be used to create an interactive control to set up
// gamma arrays.
//------------------------------------------------------------------------
pub struct Spline<'a, C: Color> {
    ctrl: CtrlBase,
    num_pnt: u32,
    xp: [f64; 32],
    yp: [f64; 32],
    spline: Bspline,
    spline_values: [f64; 256],
    spline_values8: [u8; 256],
    border_width: f64,
    border_extra: f64,
    curve_width: f64,
    point_size: f64,
    xs1: f64,
    ys1: f64,
    xs2: f64,
    ys2: f64,
    curve_pnt: PathStorage,
    curve_poly: ConvStroke<'a, PathStorage>,
    ellipse: Ellipse,
    idx: u32,
    vertex: u32,
    vx: [f64; 32],
    vy: [f64; 32],
    active_pnt: i32,
    move_pnt: i32,
    pdx: f64,
    pdy: f64,
    background_color: C,
    border_color: C,
    curve_color: C,
    inactive_pnt_color: C,
    active_pnt_color: C,
    mtx: Option<TransAffine>,
}

impl<'a, C: Color> Spline<'a, C> {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, num_pnt: u32, flip_y: bool) -> Self {
        let mut ctrl = Self {
            ctrl: CtrlBase::new(x1, y1, x2, y2, flip_y),
            num_pnt: num_pnt,
            border_width: 1.0,
            border_extra: 0.0,
            curve_width: 1.0,
            point_size: 3.0,
            curve_pnt: PathStorage::new(),
            curve_poly: ConvStroke::new_owned(PathStorage::new()),
            ellipse: Ellipse::new(),
            idx: 0,
            vertex: 0,
            active_pnt: -1,
            move_pnt: -1,
            pdx: 0.0,
            pdy: 0.0,
            xp: [0.0; 32],
            yp: [0.0; 32],
            spline: Bspline::new(),
            spline_values: [0.0; 256],
            spline_values8: [0; 256],
            xs1: 0.0,
            ys1: 0.0,
            xs2: 0.0,
            ys2: 0.0,
            vx: [0.0; 32],
            vy: [0.0; 32],
            background_color: C::new_from_rgba(&Rgba::new_params(1.0, 1.0, 0.9, 1.0)),
            border_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            curve_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            inactive_pnt_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            active_pnt_color: C::new_from_rgba(&Rgba::new_params(1.8, 0.0, 0.0, 1.0)),

            mtx: None,
        };
        if ctrl.num_pnt < 4 {
            ctrl.num_pnt = 4;
        }
        if ctrl.num_pnt > 32 {
            ctrl.num_pnt = 32;
        }
        for i in 0..ctrl.num_pnt as usize {
            ctrl.xp[i] = (i as f64) / ((ctrl.num_pnt - 1) as f64);
            ctrl.yp[i] = 0.5;
        }
        ctrl.calc_spline_box();
        ctrl.update_spline();
        ctrl
    }

    pub fn set_curve_width(&mut self, t: f64) {
        self.curve_width = t;
    }

    pub fn set_point_size(&mut self, s: f64) {
        self.point_size = s;
    }

    pub fn spline(&self) -> &[f64] {
        &self.spline_values
    }

    pub fn spline8(&self) -> &[u8] {
        &self.spline_values8
    }

    pub fn set_x(&mut self, idx: u32, x: f64) {
        self.xp[idx as usize] = x;
    }

    pub fn set_y(&mut self, idx: u32, y: f64) {
        self.yp[idx as usize] = y;
    }

    pub fn x(&self, idx: u32) -> f64 {
        self.xp[idx as usize]
    }

    pub fn y(&self, idx: u32) -> f64 {
        self.yp[idx as usize]
    }

    pub fn set_border_width(&mut self, t: f64, extra: f64) {
        self.border_width = t;
        self.border_extra = extra;
        self.calc_spline_box();
    }

    pub fn calc_spline_box(&mut self) {
        self.xs1 = self.ctrl.x1 + self.border_width;
        self.ys1 = self.ctrl.y1 + self.border_width;
        self.xs2 = self.ctrl.x2 - self.border_width;
        self.ys2 = self.ctrl.y2 - self.border_width;
    }

    pub fn set_background_color(&mut self, c: &C) {
        self.background_color = *c;
    }

    pub fn update_spline(&mut self) {
        self.spline
            .init_with_points(self.num_pnt as usize, &self.xp, &self.yp);
        for i in 0..256 {
            self.spline_values[i] = self.spline.get(i as f64 / 255.0);
            if self.spline_values[i] < 0.0 {
                self.spline_values[i] = 0.0;
            }
            if self.spline_values[i] > 1.0 {
                self.spline_values[i] = 1.0;
            }
            self.spline_values8[i] = (self.spline_values[i] * 255.0) as u8;
        }
    }

    pub fn calc_curve(&mut self) {
        self.curve_poly.source_mut().remove_all();
        self.curve_poly.source_mut().move_to(
            self.xs1,
            self.ys1 + (self.ys2 - self.ys1) * self.spline_values[0],
        );
        for i in 1..256 {
            self.curve_poly.source_mut().line_to(
                self.xs1 + (self.xs2 - self.xs1) * i as f64 / 255.0,
                self.ys1 + (self.ys2 - self.ys1) * self.spline_values[i],
            );
        }
    }

    pub fn calc_xp(&self, idx: u32) -> f64 {
        self.xs1 + (self.xs2 - self.xs1) * self.xp[idx as usize]
    }

    pub fn calc_yp(&self, idx: u32) -> f64 {
        self.ys1 + (self.ys2 - self.ys1) * self.yp[idx as usize]
    }

    pub fn set_xp(&mut self, idx: u32, val: f64) {
        let mut val = val;

        if val < 0.0 {
            val = 0.0;
        }
        if val > 1.0 {
            val = 1.0;
        }
        if idx == 0 {
            val = 0.0;
        } else if idx == self.num_pnt - 1 {
            val = 1.0;
        } else {
            if val < self.xp[(idx - 1) as usize] + 0.001 {
                val = self.xp[(idx - 1) as usize] + 0.001;
            }
            if val > self.xp[(idx + 1) as usize] - 0.001 {
                val = self.xp[(idx + 1) as usize] - 0.001;
            }
        }
        self.xp[idx as usize] = val;
    }

    pub fn set_yp(&mut self, idx: u32, val: f64) {
        let mut val = val;

        if val < 0.0 {
            val = 0.0;
        }
        if val > 1.0 {
            val = 1.0;
        }
        self.yp[idx as usize] = val;
    }

    pub fn set_point(&mut self, idx: u32, x: f64, y: f64) {
        if idx < self.num_pnt {
            self.set_xp(idx, x);
            self.set_yp(idx, y);
        }
    }

    pub fn set_value(&mut self, idx: u32, y: f64) {
        if idx < self.num_pnt {
            self.set_yp(idx, y);
        }
    }

    pub fn value(&self, x: f64) -> f64 {
        let x = self.spline.get(x);
        if x < 0.0 {
            0.0
        } else if x > 1.0 {
            1.0
        } else {
            x
        }
    }

    fn active_point(&mut self, i: i32) {
        self.active_pnt = i;
    }
}

impl<'a, C: Color> Deref for Spline<'a, C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<'a, C: Color> DerefMut for Spline<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

impl<'a, C: Color> CtrlColor for Spline<'a, C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            0 => self.background_color,
            1 => self.border_color,
            2 => self.curve_color,
            3 => self.inactive_pnt_color,
            _ => self.active_pnt_color,
        }
    }
}

impl<'a, C: Color> Ctrl for Spline<'a, C> {
    fn num_paths(&self) -> u32 {
        5
    }

    fn set_transform(&mut self, mtx: &agg::TransAffine) {
        self.ctrl.set_transform(&mtx);
    }

    fn in_rect(&self, x: f64, y: f64) -> bool {
        let mut x = x;
        let mut y = y;
        self.inverse_transform_xy(&mut x, &mut y);
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }

    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
        let mut x = x;
        let mut y = y;
        self.inverse_transform_xy(&mut x, &mut y);

        for i in 0..self.num_pnt {
            let xp = self.calc_xp(i);
            let yp = self.calc_yp(i);
            if calc_distance(x, y, xp, yp) <= self.point_size + 1.0 {
                self.pdx = xp - x;
                self.pdy = yp - y;
                self.active_pnt = i as i32;
                self.move_pnt = i as i32;
                return true;
            }
        }
        false
    }

    fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        if self.move_pnt >= 0 {
            self.move_pnt = -1;
            return true;
        }
        false
    }

    fn on_mouse_move(&mut self, x: f64, y: f64, button_flag: bool) -> bool {
        let mut x = x;
        let mut y = y;
        self.inverse_transform_xy(&mut x, &mut y);
        if !button_flag {
            return self.on_mouse_button_up(x, y);
        }

        if self.move_pnt >= 0 {
            let xp = x + self.pdx;
            let yp = y + self.pdy;

            self.set_xp(
                self.move_pnt as u32,
                (xp - self.xs1) / (self.xs2 - self.xs1),
            );
            self.set_yp(
                self.move_pnt as u32,
                (yp - self.ys1) / (self.ys2 - self.ys1),
            );

            self.update_spline();
            return true;
        }
        false
    }

    fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool {
        let mut kx = 0.0;
        let mut ky = 0.0;
        let mut ret = false;
        if self.active_pnt >= 0 {
            kx = self.xp[self.active_pnt as usize];
            ky = self.yp[self.active_pnt as usize];
            if left {
                kx -= 0.001;
                ret = true;
            }
            if right {
                kx += 0.001;
                ret = true;
            }
            if down {
                ky -= 0.001;
                ret = true;
            }
            if up {
                ky += 0.001;
                ret = true;
            }
        }
        if ret {
            self.set_xp(self.active_pnt as u32, kx);
            self.set_yp(self.active_pnt as u32, ky);
            self.update_spline();
        }
        ret
    }
}

impl<'a, C: Color> VertexSource for Spline<'a, C> {
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
                self.vx[0] = self.x1;
                self.vy[0] = self.y1;
                self.vx[1] = self.x2;
                self.vy[1] = self.y1;
                self.vx[2] = self.x2;
                self.vy[2] = self.y2;
                self.vx[3] = self.x1;
                self.vy[3] = self.y2;
                self.vx[4] = self.x1 + self.border_width;
                self.vy[4] = self.y1 + self.border_width;
                self.vx[5] = self.x1 + self.border_width;
                self.vy[5] = self.y2 - self.border_width;
                self.vx[6] = self.x2 - self.border_width;
                self.vy[6] = self.y2 - self.border_width;
                self.vx[7] = self.x2 - self.border_width;
                self.vy[7] = self.y1 + self.border_width;
            }
            2 => {
                self.calc_curve();
                self.curve_poly.set_width(self.curve_width);
                self.curve_poly.rewind(0);
            }
            3 => {
                self.curve_poly.source_mut().remove_all();
                for i in 0..self.num_pnt {
                    if i as i32 != self.active_pnt {
                        self.ellipse.init(
                            self.calc_xp(i),
                            self.calc_yp(i),
                            self.point_size,
                            self.point_size,
                            32,
                            false,
                        );
                        self.curve_pnt.concat_path(&mut self.ellipse, 0);
                    }
                }
                self.curve_poly.rewind(0);
            }
            4 => {
                self.curve_poly.source_mut().remove_all();
                if self.active_pnt >= 0 {
                    self.ellipse.init(
                        self.calc_xp(self.active_pnt as u32),
                        self.calc_yp(self.active_pnt as u32),
                        self.point_size,
                        self.point_size,
                        32,
                        false,
                    );

                    self.curve_pnt.concat_path(&mut self.ellipse, 0);
                }
                self.curve_poly.rewind(0);
            }
            _ => {}
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
            2 => {
                cmd = self.curve_poly.vertex(x, y);
            }
            3 | 4 => {
                cmd = self.curve_pnt.vertex(x, y);
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

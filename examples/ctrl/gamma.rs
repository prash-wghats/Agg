use crate::ctrl::*;
use agg::basics::{PathCmd, *};
use agg::color_rgba::*;
use agg::conv_stroke::*;
use agg::ellipse::*;
use agg::gsv_text::*;
use agg::math::calc_distance;
use agg::math_stroke::{LineCap, LineJoin};
use agg::{Color, VertexSource};
use std::ops::{Deref, DerefMut};

pub struct Gamma<'a, C: Color> {
    ctrl: CtrlBase,
    border_width: f64,
    border_extra: f64,
    curve_width: f64,
    grid_width: f64,
    text_thickness: f64,
    point_size: f64,
    text_height: f64,
    text_width: f64,
    xc1: f64,
    yc1: f64,
    xc2: f64,
    yc2: f64,
    xs1: f64,
    ys1: f64,
    xs2: f64,
    ys2: f64,
    xt1: f64,
    yt1: f64,
    xt2: f64,
    yt2: f64,
    curve_poly: ConvStroke<'a, gamma_spline::GammaSpline>,
    ellipse: Ellipse,
    text: GsvText,
    text_poly: ConvStroke<'a, GsvText>,
    idx: u32,
    vertex: usize,
    vx: [f64; 32],
    vy: [f64; 32],
    xp1: f64,
    yp1: f64,
    xp2: f64,
    yp2: f64,
    p1_active: bool,
    mouse_point: u32,
    pdx: f64,
    pdy: f64,
    background_color: C,
    border_color: C,
    curve_color: C,
    grid_color: C,
    inactive_pnt_color: C,
    active_pnt_color: C,
    text_color: C,
}

impl<'a, C: Color> Gamma<'a, C> {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, flip_y: bool) -> Self {
        let mut gamma = Self {
            ctrl: CtrlBase::new(x1, y1, x2, y2, flip_y),
            border_width: 2.0,
            border_extra: 0.0,
            curve_width: 2.0,
            grid_width: 0.2,
            text_thickness: 1.5,
            point_size: 5.0,
            text_height: 9.0,
            text_width: 0.0,
            xc1: x1,
            yc1: y1,
            xc2: x2,
            yc2: y2 - 9.0 * 2.0,
            xs1: 0.0,
            ys1: 0.0,
            xs2: 0.0,
            ys2: 0.0,
            xt1: x1,
            yt1: y2 - 9.0 * 2.0,
            xt2: x2,
            yt2: y2,
            curve_poly: ConvStroke::new_owned(gamma_spline::GammaSpline::new()),
            ellipse: Ellipse::new(),
            text: GsvText::new(),
            text_poly: ConvStroke::new_owned(GsvText::new()),
            idx: 0,
            vertex: 0,
            vx: [0.0; 32],
            vy: [0.0; 32],
            xp1: 0.0,
            yp1: 0.0,
            xp2: 0.0,
            yp2: 0.0,
            p1_active: true,
            mouse_point: 0,
            pdx: 0.0,
            pdy: 0.0,
            background_color: C::new_from_rgba(&Rgba::new_params(1.0, 1., 0.9, 1.0)),
            border_color: C::new_from_rgba(&Rgba::new_params(0., 0., 0., 1.0)),
            text_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            curve_color: C::new_from_rgba(&Rgba::new_params(0., 0., 0., 1.)),
            grid_color: C::new_from_rgba(&Rgba::new_params(0.2, 0.2, 0.0, 1.)),
            inactive_pnt_color: C::new_from_rgba(&Rgba::new_params(0., 0., 0., 1.)),
            active_pnt_color: C::new_from_rgba(&Rgba::new_params(1.0, 0.0, 0.0, 1.)),
        };
        gamma.calc_spline_box();
        gamma
    }

    fn calc_spline_box(&mut self) {
        self.xs1 = self.xc1 + self.border_width;
        self.ys1 = self.yc1 + self.border_width;
        self.xs2 = self.xc2 - self.border_width;
        self.ys2 = self.yc2 - self.border_width * 0.5;
    }

    fn calc_points(&mut self) {
        let mut kx1: f64 = 0.0;
        let mut ky1: f64 = 0.0;
        let mut kx2: f64 = 0.0;
        let mut ky2: f64 = 0.0;
        self.curve_poly.source_mut()
            .values(&mut kx1, &mut ky1, &mut kx2, &mut ky2);
        self.xp1 = self.xs1 + (self.xs2 - self.xs1) * kx1 * 0.25;
        self.yp1 = self.ys1 + (self.ys2 - self.ys1) * ky1 * 0.25;
        self.xp2 = self.xs2 - (self.xs2 - self.xs1) * kx2 * 0.25;
        self.yp2 = self.ys2 - (self.ys2 - self.ys1) * ky2 * 0.25;
    }

    fn calc_values(&mut self) {
        self.curve_poly.source_mut().set_values(
            (self.xp1 - self.xs1) * 4.0 / (self.xs2 - self.xs1),
            (self.yp1 - self.ys1) * 4.0 / (self.ys2 - self.ys1),
            (self.xs2 - self.xp2) * 4.0 / (self.xs2 - self.xs1),
            (self.ys2 - self.yp2) * 4.0 / (self.ys2 - self.ys1),
        );
    }

    pub fn set_text_size(&mut self, h: f64, w: f64) {
        self.text_width = w;
        self.text_height = h;
        self.yc2 = self.y2 - self.text_height * 2.0;
        self.yt1 = self.y2 - self.text_height * 2.0;
        self.calc_spline_box();
    }

    pub fn set_border_width(&mut self, t: f64, extra: f64) {
        self.border_width = t;
        self.border_extra = extra;
        self.calc_spline_box();
    }

    pub fn set_values(&mut self, kx1: f64, ky1: f64, kx2: f64, ky2: f64) {
        self.curve_poly.source_mut().set_values(kx1, ky1, kx2, ky2);
    }

    pub fn values(&self, kx1: &mut f64, ky1: &mut f64, kx2: &mut f64, ky2: &mut f64) {
        self.curve_poly.source().values(kx1, ky1, kx2, ky2);
    }

    pub fn set_curve_width(&mut self, t: f64) {
        self.curve_width = t;
    }

    pub fn set_grid_width(&mut self, t: f64) {
        self.grid_width = t;
    }

    pub fn set_text_thickness(&mut self, t: f64) {
        self.text_thickness = t;
    }

    pub fn set_point_size(&mut self, s: f64) {
        self.point_size = s;
    }

    pub fn gamma(&self) -> &[u8] {
        self.curve_poly.source().gamma()
    }

    pub fn y(&self, x: f64) -> f64 {
        self.curve_poly.source().y(x)
    }

    pub fn gamma_spline(&self) -> &gamma_spline::GammaSpline {
        &self.curve_poly.source()
    }

    fn change_active_point(&mut self) {
        self.p1_active = if self.p1_active { false } else { true };
    }
}

impl<'a, C: Color> agg::GammaFn for Gamma<'a, C> {
	fn call(&self, x: f64) -> f64 {
		self.curve_poly.source().y(x)
	}
}

impl<'a, C: Color> Deref for Gamma<'a, C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<'a, C: Color> DerefMut for Gamma<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

impl<'a, C: Color> CtrlColor for Gamma<'a, C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            0 => self.background_color,
            1 => self.border_color,
            2 => self.curve_color,
            3 => self.grid_color,
            4 => self.inactive_pnt_color,
            5 => self.active_pnt_color,
            _ => self.text_color,
        }
    }
}

impl<'a, C: Color> Ctrl for Gamma<'a, C> {
    fn num_paths(&self) -> u32 {
        7
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
        self.calc_points();
        if calc_distance(x, y, self.xp1, self.yp1) <= self.point_size + 1.0 {
            self.mouse_point = 1;
            self.pdx = self.xp1 - x;
            self.pdy = self.yp1 - y;
            self.p1_active = true;
            return true;
        }
        if calc_distance(x, y, self.xp2, self.yp2) <= self.point_size + 1.0 {
            self.mouse_point = 2;
            self.pdx = self.xp2 - x;
            self.pdy = self.yp2 - y;
            self.p1_active = false;
            return true;
        }
        false
    }

    fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        if self.mouse_point != 0 {
            self.mouse_point = 0;
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
        if self.mouse_point == 1 {
            self.xp1 = x + self.pdx;
            self.yp1 = y + self.pdy;
            self.calc_values();
            return true;
        }
        if self.mouse_point == 2 {
            self.xp2 = x + self.pdx;
            self.yp2 = y + self.pdy;
            self.calc_values();
            return true;
        }
        false
    }

    fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool {
        let mut kx1 = 0.0;
        let mut ky1 = 0.0;
        let mut kx2 = 0.0;
        let mut ky2 = 0.0;
        self.curve_poly.source_mut()
            .values(&mut kx1, &mut ky1, &mut kx2, &mut ky2);
        let mut ret = false;
        if self.p1_active {
            if left {
                kx1 -= 0.005;
                ret = true;
            }
            if right {
                kx1 += 0.005;
                ret = true;
            }
            if down {
                ky1 -= 0.005;
                ret = true;
            }
            if up {
                ky1 += 0.005;
                ret = true;
            }
        } else {
            if left {
                kx2 += 0.005;
                ret = true;
            }
            if right {
                kx2 -= 0.005;
                ret = true;
            }
            if down {
                ky2 += 0.005;
                ret = true;
            }
            if up {
                ky2 -= 0.005;
                ret = true;
            }
        }
        if ret {
            self.curve_poly.source_mut().set_values(kx1, ky1, kx2, ky2);
        }
        ret
    }
}

impl<'a, C: Color> VertexSource for Gamma<'a, C> {
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
                *x = self.vx[self.vertex];
                *y = self.vy[self.vertex];
                self.vertex += 1;
            }
            1 => {
                if self.vertex == 0 || self.vertex == 4 || self.vertex == 8 {
                    cmd = PathCmd::MoveTo as u32;
                }
                if self.vertex >= 12 {
                    cmd = PathCmd::Stop as u32;
                }
                *x = self.vx[self.vertex];
                *y = self.vy[self.vertex];
                self.vertex += 1;
            }
            2 => {
                cmd = self.curve_poly.vertex(x, y);
            }
            3 => {
                if self.vertex == 0 || self.vertex == 4 || self.vertex == 8 || self.vertex == 14 {
                    cmd = PathCmd::MoveTo as u32;
                }
                if self.vertex >= 20 {
                    cmd = PathCmd::Stop as u32;
                }
                *x = self.vx[self.vertex];
                *y = self.vy[self.vertex];
                self.vertex += 1;
            }
            4 | 5 => {
                cmd = self.ellipse.vertex(x, y);
            }
            6 => {
                cmd = self.text_poly.vertex(x, y);
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

    fn rewind(&mut self, idx: u32) {
        let mut kx1: f64 = 0.0;
        let mut ky1: f64 = 0.0;
        let mut kx2: f64 = 0.0;
        let mut ky2: f64 = 0.0;

        self.idx = idx;

        match idx {
            0 => {
                // Background
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
                self.vx[4] = self.x1 + self.border_width;
                self.vy[4] = self.y1 + self.border_width;
                self.vx[5] = self.x1 + self.border_width;
                self.vy[5] = self.y2 - self.border_width;
                self.vx[6] = self.x2 - self.border_width;
                self.vy[6] = self.y2 - self.border_width;
                self.vx[7] = self.x2 - self.border_width;
                self.vy[7] = self.y1 + self.border_width;
                self.vx[8] = self.xc1 + self.border_width;
                self.vy[8] = self.yc2 - self.border_width * 0.5;
                self.vx[9] = self.xc2 - self.border_width;
                self.vy[9] = self.yc2 - self.border_width * 0.5;
                self.vx[10] = self.xc2 - self.border_width;
                self.vy[10] = self.yc2 + self.border_width * 0.5;
                self.vx[11] = self.xc1 + self.border_width;
                self.vy[11] = self.yc2 + self.border_width * 0.5;
            }
            2 => {
                // Curve
                self.curve_poly.source_mut()
                    .set_box(self.xs1, self.ys1, self.xs2, self.ys2);
                self.curve_poly.set_width(self.curve_width);
                self.curve_poly.rewind(0);
            }
            3 => {
                // Grid
                self.vertex = 0;
                self.vx[0] = self.xs1;
                self.vy[0] = (self.ys1 + self.ys2) * 0.5 - self.grid_width * 0.5;
                self.vx[1] = self.xs2;
                self.vy[1] = (self.ys1 + self.ys2) * 0.5 - self.grid_width * 0.5;
                self.vx[2] = self.xs2;
                self.vy[2] = (self.ys1 + self.ys2) * 0.5 + self.grid_width * 0.5;
                self.vx[3] = self.xs1;
                self.vy[3] = (self.ys1 + self.ys2) * 0.5 + self.grid_width * 0.5;
                self.vx[4] = (self.xs1 + self.xs2) * 0.5 - self.grid_width * 0.5;
                self.vy[4] = self.ys1;
                self.vx[5] = (self.xs1 + self.xs2) * 0.5 - self.grid_width * 0.5;
                self.vy[5] = self.ys2;
                self.vx[6] = (self.xs1 + self.xs2) * 0.5 + self.grid_width * 0.5;
                self.vy[6] = self.ys2;
                self.vx[7] = (self.xs1 + self.xs2) * 0.5 + self.grid_width * 0.5;
                self.vy[7] = self.ys1;
                self.calc_points();
                self.vx[8] = self.xs1;
                self.vy[8] = self.yp1 - self.grid_width * 0.5;
                self.vx[9] = self.xp1 - self.grid_width * 0.5;
                self.vy[9] = self.yp1 - self.grid_width * 0.5;
                self.vx[10] = self.xp1 - self.grid_width * 0.5;
                self.vy[10] = self.ys1;
                self.vx[11] = self.xp1 + self.grid_width * 0.5;
                self.vy[11] = self.ys1;
                self.vx[12] = self.xp1 + self.grid_width * 0.5;
                self.vy[12] = self.yp1 + self.grid_width * 0.5;
                self.vx[13] = self.xs1;
                self.vy[13] = self.yp1 + self.grid_width * 0.5;
                self.vx[14] = self.xs2;
                self.vy[14] = self.yp2 + self.grid_width * 0.5;
                self.vx[15] = self.xp2 + self.grid_width * 0.5;
                self.vy[15] = self.yp2 + self.grid_width * 0.5;
                self.vx[16] = self.xp2 + self.grid_width * 0.5;
                self.vy[16] = self.ys2;
                self.vx[17] = self.xp2 - self.grid_width * 0.5;
                self.vy[17] = self.ys2;
                self.vx[18] = self.xp2 - self.grid_width * 0.5;
                self.vy[18] = self.yp2 - self.grid_width * 0.5;
                self.vx[19] = self.xs2;
                self.vy[19] = self.yp2 - self.grid_width * 0.5;
            }
            4 => {
                // Point1
                self.calc_points();
                if self.p1_active {
                    self.ellipse.init(
                        self.xp2,
                        self.yp2,
                        self.point_size,
                        self.point_size,
                        32,
                        false,
                    );
                } else {
                    self.ellipse.init(
                        self.xp1,
                        self.yp1,
                        self.point_size,
                        self.point_size,
                        32,
                        false,
                    );
                }
            }
            5 => {
                // Point2
                self.calc_points();
                if self.p1_active {
                    self.ellipse.init(
                        self.xp1,
                        self.yp1,
                        self.point_size,
                        self.point_size,
                        32,
                        false,
                    );
                } else {
                    self.ellipse.init(
                        self.xp2,
                        self.yp2,
                        self.point_size,
                        self.point_size,
                        32,
                        false,
                    );
                }
            }
            6 => {
                // Text
                self.curve_poly.source_mut()
                    .values(&mut kx1, &mut ky1, &mut kx2, &mut ky2);
                let mut tbuf = format!("{:5.3} {:5.3} {:5.3} {:5.3}", kx1, ky1, kx2, ky2);
                tbuf = tbuf + "\u{0}";
                self.text_poly.source_mut().set_text(&tbuf);
                self.text_poly
                    .source_mut()
                    .set_size(self.text_height, self.text_width);
                self.text_poly.source_mut().set_start_point(
                    self.xt1 + self.border_width * 2.0,
                    (self.yt1 + self.yt2) * 0.5 - self.text_height * 0.5,
                );
                self.text_poly.set_width(self.text_thickness);
                self.text_poly.set_line_join(LineJoin::Round);
                self.text_poly.set_line_cap(LineCap::Round);
                self.text_poly.rewind(0);
            }
            _ => {}
        }
    }
}

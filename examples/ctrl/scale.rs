use crate::ctrl::*;
use agg::basics::{PathCmd, *};
use agg::color_rgba::*;
use agg::ellipse::*;
use agg::math::calc_distance;
use agg::{Color, VertexSource};
use std::ops::{Deref, DerefMut};

enum Move {
    Nothing,
    Value1,
    Value2,
    Slider,
}

pub struct Scale<C: Color> {
    ctrl: CtrlBase,
    border_thickness: f64,
    border_extra: f64,
    pdx: f64,
    pdy: f64,
    move_what: Move,
    value1: f64,
    value2: f64,
    min_d: f64,
    xs1: f64,
    ys1: f64,
    xs2: f64,
    ys2: f64,
    ellipse: agg::Ellipse,
    idx: u32,
    vertex: usize,
    vx: [f64; 32],
    vy: [f64; 32],
    background_color: C,
    border_color: C,
    pointers_color: C,
    slider_color: C,
}

impl<C: Color> Scale<C> {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, flip_y: bool) -> Self {
        let mut ctrl = Self {
            ctrl: CtrlBase::new(x1, y1, x2, y2, flip_y),
            border_thickness: 1.0,
            border_extra: if (x2 - x1).abs() > (y2 - y1).abs() {
                (y2 - y1) / 2.0
            } else {
                (x2 - x1) / 2.0
            },
            pdx: 0.0,
            pdy: 0.0,
            move_what: Move::Nothing,
            value1: 0.3,
            value2: 0.7,
            min_d: 0.01,
            xs1: x1 + 1.0,
            ys1: y1 + 1.0,
            xs2: x2 - 1.0,
            ys2: y2 - 1.0,
            ellipse: Ellipse::new(),
            idx: 0,
            vertex: 0,
            vx: [0.0; 32],
            vy: [0.0; 32],
            background_color: C::new_from_rgba(&Rgba::new_params(1.0, 0.9, 0.8, 1.0)),
            border_color: C::new_from_rgba(&Rgba::new_params(0., 0., 0., 1.0)),
            pointers_color: C::new_from_rgba(&Rgba::new_params(0.8, 0.0, 0.0, 0.8)),
            slider_color: C::new_from_rgba(&Rgba::new_params(0.2, 0.1, 0., 0.6)),
        };
        ctrl.calc_box();
        ctrl
    }

    pub fn calc_box(&mut self) {
        self.xs1 = self.ctrl.x1 + self.border_thickness;
        self.ys1 = self.ctrl.y1 + self.border_thickness;
        self.xs2 = self.ctrl.x2 - self.border_thickness;
        self.ys2 = self.ctrl.y2 - self.border_thickness;
    }

    pub fn border_thickness(&mut self, t: f64, extra: f64) {
        self.border_thickness = t;
        self.border_extra = extra;
        self.calc_box();
    }

    pub fn resize(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.ctrl.x1 = x1;
        self.ctrl.y1 = y1;
        self.ctrl.x2 = x2;
        self.ctrl.y2 = y2;
        self.calc_box();
        self.border_extra = if (x2 - x1).abs() > (y2 - y1).abs() {
            (y2 - y1) / 2.0
        } else {
            (x2 - x1) / 2.0
        };
    }

    pub fn value1(&self) -> f64 {
        self.value1
    }

    pub fn value2(&self) -> f64 {
        self.value2
    }

    pub fn set_value1(&mut self, value: f64) {
        let mut value = value;

        if value < 0.0 {
            value = 0.0;
        }
        if value > 1.0 {
            value = 1.0;
        }
        if self.value2 - value < self.min_d {
            value = self.value2 - self.min_d;
        }
        self.value1 = value;
    }

    pub fn set_value2(&mut self, value: f64) {
        let mut value = value;

        if value < 0.0 {
            value = 0.0;
        }
        if value > 1.0 {
            value = 1.0;
        }
        if self.value1 + value < self.min_d {
            value = self.value1 + self.min_d;
        }
        self.value2 = value;
    }

    pub fn move_(&mut self, d: f64) {
        self.value1 += d;
        self.value2 += d;
        if self.value1 < 0.0 {
            self.value2 -= self.value1;
            self.value1 = 0.0;
        }
        if self.value2 > 1.0 {
            self.value1 -= self.value2 - 1.0;
            self.value2 = 1.0;
        }
    }
}

impl<C: Color> Deref for Scale<C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<C: Color> DerefMut for Scale<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

impl<C: Color> CtrlColor for Scale<C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            0 => self.background_color,
            1 => self.border_color,
            2 => self.pointers_color,
            _ => self.slider_color,
        }
    }
}

impl<C: Color> Ctrl for Scale<C> {
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

    fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        self.move_what = Move::Nothing;
        false
    }
    fn on_arrow_keys(&mut self, _left: bool, _right: bool, _down: bool, _up: bool) -> bool {
        false
    }

    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
        let (mut x, mut y) = (x, y);

        self.inverse_transform_xy(&mut x, &mut y);

        let xp1: f64;
        let xp2: f64;
        let ys1: f64;
        let ys2: f64;
        let xp: f64;
        let yp: f64;

        if (self.x2 - self.x1).abs() > (self.y2 - self.y1).abs() {
            xp1 = self.xs1 + (self.xs2 - self.xs1) * self.value1;
            xp2 = self.xs1 + (self.xs2 - self.xs1) * self.value2;
            ys1 = self.y1 - self.border_extra / 2.0;
            ys2 = self.y2 + self.border_extra / 2.0;
            yp = (self.ys1 + self.ys2) / 2.0;

            if x > xp1 && y > ys1 && x < xp2 && y < ys2 {
                self.pdx = xp1 - x;
                self.move_what = Move::Slider;
                return true;
            }

            //if(x < xp1 && calc_distance(x, y, xp1, yp) <= m_y2 - m_y1)
            if calc_distance(x, y, xp1, yp) <= self.y2 - self.y1 {
                self.pdx = xp1 - x;
                self.move_what = Move::Value1;
                return true;
            }

            //if(x > xp2 && calc_distance(x, y, xp2, yp) <= m_y2 - m_y1)
            if calc_distance(x, y, xp2, yp) <= self.y2 - self.y1 {
                self.pdx = xp2 - x;
                self.move_what = Move::Value2;
                return true;
            }
        } else {
            xp1 = self.x1 - self.border_extra / 2.0;
            xp2 = self.x2 + self.border_extra / 2.0;
            ys1 = self.ys1 + (self.ys2 - self.ys1) * self.value1;
            ys2 = self.ys1 + (self.ys2 - self.ys1) * self.value2;
            xp = (self.xs1 + self.xs2) / 2.0;

            if x > xp1 && y > ys1 && x < xp2 && y < ys2 {
                self.pdy = ys1 - y;
                self.move_what = Move::Slider;
                return true;
            }

            //if(y < ys1 && calc_distance(x, y, xp, ys1) <= m_x2 - m_x1)
            if calc_distance(x, y, xp, ys1) <= self.x2 - self.x1 {
                self.pdy = ys1 - y;
                self.move_what = Move::Value1;
                return true;
            }

            //if(y > ys2 && calc_distance(x, y, xp, ys2) <= m_x2 - m_x1)
            if calc_distance(x, y, xp, ys2) <= self.x2 - self.x1 {
                self.pdy = ys2 - y;
                self.move_what = Move::Value2;
                return true;
            }
        }

        false
    }

    fn on_mouse_move(&mut self, x: f64, y: f64, button_flag: bool) -> bool {
		let (mut x, mut y) = (x, y);
		
        self.inverse_transform_xy(&mut x, &mut y);
        if !button_flag {
            return self.on_mouse_button_up(x, y);
        }

        let xp = x + self.pdx;
        let yp = y + self.pdy;
        let mut dv: f64;

        match self.move_what {
            Move::Value1 => {
                if (self.x2 - self.x1).abs() > (self.y2 - self.y1).abs() {
                    self.value1 = (xp - self.xs1) / (self.xs2 - self.xs1);
                } else {
                    self.value1 = (yp - self.ys1) / (self.ys2 - self.ys1);
                }
                if self.value1 < 0.0 {
                    self.value1 = 0.0;
                }
                if self.value1 > self.value2 - self.min_d {
                    self.value1 = self.value2 - self.min_d;
                }
                return true;
            }
            Move::Value2 => {
                if (self.x2 - self.x1).abs() > (self.y2 - self.y1).abs() {
                    self.value2 = (xp - self.xs1) / (self.xs2 - self.xs1);
                } else {
                    self.value2 = (yp - self.ys1) / (self.ys2 - self.ys1);
                }
                if self.value2 > 1.0 {
                    self.value2 = 1.0;
                }
                if self.value2 < self.value1 + self.min_d {
                    self.value2 = self.value1 + self.min_d;
                }
                return true;
            }
            Move::Slider => {
                dv = self.value2 - self.value1;
                if (self.x2 - self.x1).abs() > (self.y2 - self.y1).abs() {
                    self.value1 = (xp - self.xs1) / (self.xs2 - self.xs1);
                } else {
                    self.value1 = (yp - self.ys1) / (self.ys2 - self.ys1);
                }
                self.value2 = self.value1 + dv;
                if self.value1 < 0.0 {
                    dv = self.value2 - self.value1;
                    self.value1 = 0.0;
                    self.value2 = self.value1 + dv;
                }
                if self.value2 > 1.0 {
                    dv = self.value2 - self.value1;
                    self.value2 = 1.0;
                    self.value1 = self.value2 - dv;
                }
                return true;
            }
            _ => {}
        }
        return false;
    }
}

impl<C: Color> VertexSource for Scale<C> {
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
                self.vx[4] = self.x1 + self.border_thickness;
                self.vy[4] = self.y1 + self.border_thickness;
                self.vx[5] = self.x1 + self.border_thickness;
                self.vy[5] = self.y2 - self.border_thickness;
                self.vx[6] = self.x2 - self.border_thickness;
                self.vy[6] = self.y2 - self.border_thickness;
                self.vx[7] = self.x2 - self.border_thickness;
                self.vy[7] = self.y1 + self.border_thickness;
            }
            2 => {
                if (self.x2 - self.x1).abs() > (self.y2 - self.y1).abs() {
                    self.ellipse.init(
                        self.xs1 + (self.xs2 - self.xs1) * self.value1,
                        (self.ys1 + self.ys2) / 2.0,
                        self.y2 - self.y1,
                        self.y2 - self.y1,
                        32,
                        false,
                    );
                } else {
                    self.ellipse.init(
                        (self.xs1 + self.xs2) / 2.0,
                        self.ys1 + (self.ys2 - self.ys1) * self.value1,
                        self.x2 - self.x1,
                        self.x2 - self.x1,
                        32,
                        false,
                    );
                }
                self.ellipse.rewind(0);
            }
            3 => {
                if (self.x2 - self.x1).abs() > (self.y2 - self.y1).abs() {
                    self.ellipse.init(
                        self.xs1 + (self.xs2 - self.xs1) * self.value2,
                        (self.ys1 + self.ys2) / 2.0,
                        self.y2 - self.y1,
                        self.y2 - self.y1,
                        32,
                        false,
                    );
                } else {
                    self.ellipse.init(
                        (self.xs1 + self.xs2) / 2.0,
                        self.ys1 + (self.ys2 - self.ys1) * self.value2,
                        self.x2 - self.x1,
                        self.x2 - self.x1,
                        32,
                        false,
                    );
                }
                self.ellipse.rewind(0);
            }
            4 => {
                self.vertex = 0;
                if (self.x2 - self.x1).abs() > (self.y2 - self.y1).abs() {
                    self.vx[0] = self.xs1 + (self.xs2 - self.xs1) * self.value1;
                    self.vy[0] = self.y1 - self.border_extra / 2.0;
                    self.vx[1] = self.xs1 + (self.xs2 - self.xs1) * self.value2;
                    self.vy[1] = self.vy[0];
                    self.vx[2] = self.vx[1];
                    self.vy[2] = self.y2 + self.border_extra / 2.0;
                    self.vx[3] = self.vx[0];
                    self.vy[3] = self.vy[2];
                } else {
                    self.vx[0] = self.x1 - self.border_extra / 2.0;
                    self.vy[0] = self.ys1 + (self.ys2 - self.ys1) * self.value1;
                    self.vx[1] = self.vx[0];
                    self.vy[1] = self.ys1 + (self.ys2 - self.ys1) * self.value2;
                    self.vx[2] = self.x2 + self.border_extra / 2.0;
                    self.vy[2] = self.vy[1];
                    self.vx[3] = self.vx[2];
                    self.vy[3] = self.vy[0];
                }
            }
            _ => (),
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::LineTo as u32;
        match self.idx {
            0 | 4 => {
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
                if self.vertex == 0 || self.vertex == 4 {
                    cmd = PathCmd::MoveTo as u32;
                }
                if self.vertex >= 8 {
                    cmd = PathCmd::Stop as u32;
                }
                *x = self.vx[self.vertex];
                *y = self.vy[self.vertex];
                self.vertex += 1;
            }
            2 | 3 => {
                cmd = self.ellipse.vertex(x, y);
            }
            _ => {
                cmd = PathCmd::Stop as u32;
            }
        }
        if !is_stop(cmd) {
            self.ctrl.transform_xy(x, y);
        }
        cmd
    }
}

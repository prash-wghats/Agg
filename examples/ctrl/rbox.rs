use std::ops::{Deref, DerefMut};

use crate::ctrl::CtrlBase;
use agg::basics::{is_stop, PathCmd};
use agg::color_rgba::Rgba;
use agg::math::calc_distance;
use agg::math_stroke::{LineCap, LineJoin};
use agg::{Color, VertexSource};

use super::{Ctrl, CtrlColor};

//------------------------------------------------------------------------
pub struct Rbox<'a, C: Color> {
    ctrl: CtrlBase,
    border_width: f64,
    border_extra: f64,
    text_thickness: f64,
    text_height: f64,
    text_width: f64,
    items: [String; 32],
    num_items: u32,
    cur_item: i32,

    xs1: f64,
    ys1: f64,
    xs2: f64,
    ys2: f64,

    vx: [f64; 32],
    vy: [f64; 32],
    draw_item: u32,
    dy: f64,

    //ellipse: agg::Ellipse,
    ellipse_poly: agg::ConvStroke<'a, agg::Ellipse>,
    //text: Rc<RefCell<gsv_text>>,
    text_poly: agg::ConvStroke<'a, agg::GsvText>,

    idx: u32,
    vertex: u32,

    background_color: C,
    border_color: C,
    text_color: C,
    inactive_color: C,
    active_color: C,
}

impl<'a, C: Color> Rbox<'a, C> {
    pub fn set_background_color(&mut self, c: C) {
        self.background_color = c;
    }

    pub fn set_border_color(&mut self, c: C) {
        self.border_color = c;
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

    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, flip_y: bool) -> Self {
        const EMPTY_STRING: String = String::new();
        let mut ctrl = Self {
            ctrl: CtrlBase::new(x1, y1, x2, y2, flip_y),
            border_width: 1.0,
            border_extra: 0.0,
            text_thickness: 1.5,
            text_height: 9.0,
            text_width: 0.0,
            num_items: 0,
            cur_item: -1,
            //ellipse: el.clone(),
            ellipse_poly: agg::ConvStroke::new_owned(agg::Ellipse::new()),
            //text: mt.clone(),
            text_poly: agg::ConvStroke::new_owned(agg::GsvText::new()),
            idx: 0,
            vertex: 0,
            items: [EMPTY_STRING; 32],
            xs1: 0.,
            xs2: 0.,
            ys1: 0.,
            ys2: 0.,
            vx: [0.; 32],
            vy: [0.; 32],
            draw_item: 0,
            dy: 0.,
            background_color: C::new_from_rgba(&Rgba::new_params(1.0, 1.0, 0.9, 1.0)),
            border_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            text_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
            inactive_color: C::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.)),
            active_color: C::new_from_rgba(&Rgba::new_params(0.4, 0.0, 0.0, 1.)),
        };
        ctrl.calc_rbox();
        ctrl
    }

    pub fn calc_rbox(&mut self) {
        self.xs1 = self.ctrl.x1 + self.border_width;
        self.ys1 = self.ctrl.y1 + self.border_width;
        self.xs2 = self.ctrl.x2 - self.border_width;
        self.ys2 = self.ctrl.y2 - self.border_width;
    }

    pub fn set_cur_item(&mut self, i: i32) {
        self.cur_item = i;
    }

    pub fn cur_item(&self) -> i32 {
        self.cur_item
    }

    pub fn add_item(&mut self, text: &str) {
        if self.num_items < 32 {
            self.items[self.num_items as usize] = String::from(text);
            self.num_items += 1;
        }
    }

    pub fn set_border_width(&mut self, t: f64, extra: f64) {
        self.border_width = t;
        self.border_extra = extra;
        self.calc_rbox();
    }

    pub fn set_text_size(&mut self, h: f64, w: f64) {
        self.text_width = w;
        self.text_height = h;
    }

	pub fn set_text_thickness(&mut self, t: f64) {
        self.text_thickness = t;
    }
}

impl<'a, C: Color> Deref for Rbox<'a, C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<'a, C: Color> DerefMut for Rbox<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

impl<'a, C: Color> CtrlColor for Rbox<'a, C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            0 => self.background_color,
            1 => self.border_color,
            2 => self.text_color,
            3 => self.inactive_color,
            4 => self.active_color,
            _ => self.text_color,
        }
    }
}

impl<'a, C: Color> Ctrl for Rbox<'a, C> {
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
        let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        for i in 0..self.num_items {
            let xp = self.xs1 + self.dy / 1.3;
            let yp = self.ys1 + self.dy * i as f64 + self.dy / 1.3;
            if calc_distance(x, y, xp, yp) <= self.text_height / 1.5 {
                self.cur_item = i as i32;
                return true;
            }
        }
        false
    }

    fn on_mouse_move(&mut self, _x: f64, _y: f64, _left_button: bool) -> bool {
        false
    }

    fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        false
    }

    fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool {
        if self.cur_item >= 0 {
            if up || right {
                self.cur_item += 1;
                if self.cur_item >= self.num_items as i32 {
                    self.cur_item = 0;
                }
                return true;
            }

            if down || left {
                self.cur_item -= 1;
                if self.cur_item < 0 {
                    self.cur_item = self.num_items as i32 - 1;
                }
                return true;
            }
        }
        false
    }
}

impl<'a, C: Color> VertexSource for Rbox<'a, C> {
    fn rewind(&mut self, idx: u32) {
        self.idx = idx;
        self.dy = self.text_height * 2.0;
        self.draw_item = 0;

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
                self.text_poly.source_mut().set_text(&self.items[0]);
                self.text_poly.source_mut()
                    .set_start_point(self.xs1 + self.dy * 1.5, self.ys1 + self.dy / 2.0);
                self.text_poly.source_mut()
                    .set_size(self.text_height, self.text_width);
                self.text_poly.set_width(self.text_thickness);
                self.text_poly.set_line_join(LineJoin::Round);
                self.text_poly.set_line_cap(LineCap::Round);
                self.text_poly.rewind(0);
            }
            3 => {
                self.ellipse_poly.source_mut().init(
                    self.xs1 + self.dy / 1.3,
                    self.ys1 + self.dy / 1.3,
                    self.text_height / 1.5,
                    self.text_height / 1.5,
                    32,
                    false,
                );
                self.ellipse_poly.set_width(self.text_thickness);
                self.ellipse_poly.rewind(0);
            }
            4 => {
                if self.cur_item >= 0 {
                    self.ellipse_poly.source_mut().init(
                        self.xs1 + self.dy / 1.3,
                        self.ys1 + self.dy * self.cur_item as f64 + self.dy / 1.3,
                        self.text_height / 2.0,
                        self.text_height / 2.0,
                        32,
                        false,
                    );
                    self.ellipse_poly.source_mut().rewind(0);
                }
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
                cmd = self.text_poly.vertex(x, y);
                if is_stop(cmd) {
                    self.draw_item += 1;
                    if self.draw_item >= self.num_items {
                    } else {
                        self.text_poly.source_mut()
                            .set_text(&self.items[self.draw_item as usize]);
                        self.text_poly.source_mut().set_start_point(
                            self.xs1 + self.dy * 1.5,
                            self.ys1 + self.dy * (self.draw_item + 1) as f64 - self.dy / 2.0,
                        );

                        self.text_poly.rewind(0);
                        cmd = self.text_poly.vertex(x, y);
                    }
                }
            }
            3 => {
                cmd = self.ellipse_poly.vertex(x, y);
                if is_stop(cmd) {
                    self.draw_item += 1;
                    if self.draw_item >= self.num_items {
                    } else {
                        self.ellipse_poly.source_mut().init(
                            self.xs1 + self.dy / 1.3,
                            self.ys1 + self.dy * self.draw_item as f64 + self.dy / 1.3,
                            self.text_height / 1.5,
                            self.text_height / 1.5,
                            32,
                            false,
                        );
                        self.ellipse_poly.rewind(0);
                        cmd = self.ellipse_poly.vertex(x, y);
                    }
                }
            }

            4 => {
                if self.cur_item >= 0 {
                    cmd = self.ellipse_poly.source_mut().vertex(x, y);
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

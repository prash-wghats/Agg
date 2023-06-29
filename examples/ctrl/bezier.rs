use std::ops::{Deref, DerefMut};

use crate::ctrl::CtrlBase;
use agg::basics::{PathCmd, is_stop};
use agg::color_rgba::Rgba;
use agg::conv_stroke::ConvStroke;
use agg::ellipse::Ellipse;
use agg::{Color, VertexSource, CurveBase, CurveType3, CurveType4};
use agg::curves::{Curve4, Curve3};

use super::polygon::Polygon;
use super::{Ctrl, CtrlColor};




//--------------------------------------------------------bezier_ctrl_impl
pub struct Bezier<'a, C: Color> {
    poly: Polygon<'a, C>,
    //curve: Curve4,
    ellipse: Ellipse,
    stroke: ConvStroke<'a, Curve4>,
    idx: u32,
    line_color: C,
    ctrl: CtrlBase,
}

impl<'a, C: Color> Bezier<'a, C> {
    pub fn new() -> Self {
        let mut ctrl = Self {
            poly: Polygon::<C>::new(4, 5.0),
            //curve: Curve4::new(),
            ellipse: Ellipse::new(),
            stroke: ConvStroke::new_owned(Curve4::new()),
            idx: 0,
            line_color: C::new_from_rgba(&Rgba::new_params(1.0, 1.0, 0.9, 1.0)),
            ctrl: CtrlBase::new(0., 0., 1., 1., false),
        };
        ctrl.poly.set_polygon_check(false);
        *ctrl.poly.xn_mut(0) = 100.0;
        *ctrl.poly.yn_mut(0) = 0.0;
        *ctrl.poly.xn_mut(1) = 100.0;
        *ctrl.poly.yn_mut(1) = 50.0;
        *ctrl.poly.xn_mut(2) = 50.0;
        *ctrl.poly.yn_mut(2) = 100.0;
        *ctrl.poly.xn_mut(3) = 0.0;
        *ctrl.poly.yn_mut(3) = 100.0;
        ctrl
    }
    pub fn x1(&self) -> f64 {
        self.poly.xn(0)
    }
    pub fn y1(&self) -> f64 {
        self.poly.yn(0)
    }
    pub fn x2(&self) -> f64 {
        self.poly.xn(1)
    }
    pub fn y2(&self) -> f64 {
        self.poly.yn(1)
    }
    pub fn x3(&self) -> f64 {
        self.poly.xn(2)
    }
    pub fn y3(&self) -> f64 {
        self.poly.yn(2)
    }
    pub fn x4(&self) -> f64 {
        self.poly.xn(3)
    }
    pub fn y4(&self) -> f64 {
        self.poly.yn(3)
    }
    pub fn x1_mut(&mut self, x: f64) {
        *self.poly.xn_mut(0) = x;
    }
    pub fn y1_mut(&mut self, y: f64) {
        *self.poly.yn_mut(0) = y;
    }
    pub fn x2_mut(&mut self, x: f64) {
        *self.poly.xn_mut(1) = x;
    }
    pub fn y2_mut(&mut self, y: f64) {
        *self.poly.yn_mut(1) = y;
    }
    pub fn x3_mut(&mut self, x: f64) {
        *self.poly.xn_mut(2) = x;
    }
    pub fn y3_mut(&mut self, y: f64) {
        *self.poly.yn_mut(2) = y;
    }
    pub fn x4_mut(&mut self, x: f64) {
        *self.poly.xn_mut(3) = x;
    }
    pub fn y4_mut(&mut self, y: f64) {
        *self.poly.yn_mut(3) = y;
    }
    pub fn line_width(&self) -> f64 {
        self.stroke.width()
    }
    pub fn set_line_width(&mut self, w: f64) {
        self.stroke.set_width(w);
    }
    pub fn point_radius(&self) -> f64 {
        self.poly.point_radius()
    }
    pub fn set_point_radius(&mut self, r: f64) {
        self.poly.set_point_radius(r);
    }

	pub fn set_line_color(&mut self, c: &C) {
        self.line_color = *c;
    }

    pub fn set_curve(
        &mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
    ) {
        *self.poly.xn_mut(0) = x1;
        *self.poly.yn_mut(0) = y1;
        *self.poly.xn_mut(1) = x2;
        *self.poly.yn_mut(1) = y2;
        *self.poly.xn_mut(2) = x3;
        *self.poly.yn_mut(2) = y3;
        *self.poly.xn_mut(3) = x4;
        *self.poly.yn_mut(3) = y4;
        self.curve();
    }

    pub fn curve(&mut self) -> &mut Curve4 {
        self.stroke.source_mut().init(
            self.poly.xn(0),
            self.poly.yn(0),
            self.poly.xn(1),
            self.poly.yn(1),
            self.poly.xn(2),
            self.poly.yn(2),
            self.poly.xn(3),
            self.poly.yn(3),
        );
        self.stroke.source_mut()
    }
}

impl<'a, C: Color> Deref for Bezier<'a, C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<'a, C: Color> DerefMut for Bezier<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

impl<'a, C: Color> Ctrl for Bezier<'a, C> {
    fn num_paths(&self) -> u32 {
        7
    }

	fn set_transform(&mut self, mtx: &agg::TransAffine) {
		self.ctrl.set_transform(&mtx);
	}

    fn in_rect(&self, _x: f64, _y: f64) -> bool {
        false
    }
    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
		let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        self.poly.on_mouse_button_down(x, y)
    }

    fn on_mouse_move(&mut self, x: f64, y: f64, button_flag: bool) -> bool {
		let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        self.poly.on_mouse_move(x, y, button_flag)
    }

    fn on_mouse_button_up(&mut self, x: f64, y: f64) -> bool {
        self.poly.on_mouse_button_up(x, y)
    }

    fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool {
        self.poly.on_arrow_keys(left, right, down, up)
    }
}

impl<'a, C: Color> VertexSource for Bezier<'a, C> {
    fn rewind(&mut self, idx: u32) {
        self.idx = idx;

		let s = self.scale();
        self.stroke.source_mut().set_approximation_scale(s);
        match idx {
            0 => {
                self.stroke.source_mut().init(
                    self.poly.xn(0),
                    self.poly.yn(0),
                    (self.poly.xn(0) + self.poly.xn(1)) * 0.5,
                    (self.poly.yn(0) + self.poly.yn(1)) * 0.5,
                    (self.poly.xn(0) + self.poly.xn(1)) * 0.5,
                    (self.poly.yn(0) + self.poly.yn(1)) * 0.5,
                    self.poly.xn(1),
                    self.poly.yn(1),
                );
                self.stroke.rewind(0);
            }
            1 => {
                self.stroke.source_mut().init(
                    self.poly.xn(2),
                    self.poly.yn(2),
                    (self.poly.xn(2) + self.poly.xn(3)) * 0.5,
                    (self.poly.yn(2) + self.poly.yn(3)) * 0.5,
                    (self.poly.xn(2) + self.poly.xn(3)) * 0.5,
                    (self.poly.yn(2) + self.poly.yn(3)) * 0.5,
                    self.poly.xn(3),
                    self.poly.yn(3),
                );
                self.stroke.rewind(0);
            }
            2 => {
                self.stroke.source_mut().init(
                    self.poly.xn(0),
                    self.poly.yn(0),
                    self.poly.xn(1),
                    self.poly.yn(1),
                    self.poly.xn(2),
                    self.poly.yn(2),
                    self.poly.xn(3),
                    self.poly.yn(3),
                );
                self.stroke.rewind(0);
            }
            3 => {
                self.ellipse.init(
                    self.poly.xn(0),
                    self.poly.yn(0),
                    self.point_radius(),
                    self.point_radius(),
                    20,
					false,
                );
                self.ellipse.rewind(0);
            }
            4 => {
                self.ellipse.init(
                    self.poly.xn(1),
                    self.poly.yn(1),
                    self.point_radius(),
                    self.point_radius(),
                    20,
					false,
                );
                self.ellipse.rewind(0);
            }
            5 => {
                self.ellipse.init(
                    self.poly.xn(2),
                    self.poly.yn(2),
                    self.point_radius(),
                    self.point_radius(),
                    20,
					false,
                );
                self.ellipse.rewind(0);
            }
            6 => {
                self.ellipse.init(
                    self.poly.xn(3),
                    self.poly.yn(3),
                    self.point_radius(),
                    self.point_radius(),
                    20,
					false,
                );
                self.ellipse.rewind(0);
            }
            _ => {}
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::Stop as u32;
        match self.idx {
            0 | 1 | 2 => {
                cmd = self.stroke.vertex(x, y);
            }
            3 | 4 | 5 | 6 | 7 => {
                cmd = self.ellipse.vertex(x, y);
            }
            _ => {}
        }

        if !is_stop(cmd) {
            self.transform_xy(x, y);
        }
        cmd
    }
}

impl<'a, C: Color> CtrlColor for Bezier<'a, C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            _ => self.line_color,
        }
    }
}

//--------------------------------------------------------Curve3Ctrl

impl<'a, C: Color> Deref for Curve3Ctrl<'a, C> {
    type Target = CtrlBase;
    fn deref(&self) -> &Self::Target {
        &self.ctrl
    }
}

impl<'a, C: Color> DerefMut for Curve3Ctrl<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctrl
    }
}

pub struct Curve3Ctrl<'a, C: Color> {
    poly: Polygon<'a, C>,
    curve: Curve3,
    ellipse: Ellipse,
    stroke: ConvStroke<'a, Curve3>,
    idx: u32,
    line_color: C,
    ctrl: CtrlBase,
}
impl<'a, C: Color> Curve3Ctrl<'a, C> {
    pub fn new() -> Self {
        let mut poly = Polygon::new(3, 5.0);
        poly.set_polygon_check(false);
        *poly.xn_mut(0) = 100.0;
        *poly.yn_mut(0) = 0.0;
        *poly.xn_mut(1) = 100.0;
        *poly.yn_mut(1) = 50.0;
        *poly.xn_mut(2) = 50.0;
        *poly.yn_mut(2) = 100.0;
        Curve3Ctrl {
            poly: poly,
            curve: Curve3::new(),
            ellipse: Ellipse::new(),
            stroke: ConvStroke::new_owned(Curve3::new()),
            idx: 0,
            line_color: C::new_from_rgba(&Rgba::new_params(1.0, 1.0, 0.9, 1.0)),
            ctrl: CtrlBase::new(0., 0., 1., 1., false),
        }
    }
    pub fn x1(&self) -> f64 {
        self.poly.xn(0)
    }
    pub fn y1(&self) -> f64 {
        self.poly.yn(0)
    }
    pub fn x2(&self) -> f64 {
        self.poly.xn(1)
    }
    pub fn y2(&self) -> f64 {
        self.poly.yn(1)
    }
    pub fn x3(&self) -> f64 {
        self.poly.xn(2)
    }
    pub fn y3(&self) -> f64 {
        self.poly.yn(2)
    }
    pub fn x1_mut(&mut self, x: f64) {
        *self.poly.xn_mut(0) = x;
    }
    pub fn y1_mut(&mut self, y: f64) {
        *self.poly.yn_mut(0) = y;
    }
    pub fn x2_mut(&mut self, x: f64) {
        *self.poly.xn_mut(1) = x;
    }
    pub fn y2_mut(&mut self, y: f64) {
        *self.poly.yn_mut(1) = y;
    }
    pub fn x3_mut(&mut self, x: f64) {
        *self.poly.xn_mut(2) = x;
    }
    pub fn y3_mut(&mut self, y: f64) {
        *self.poly.yn_mut(2) = y;
    }

    pub fn line_width(&self) -> f64 {
        self.stroke.width()
    }
    pub fn set_line_width(&mut self, w: f64) {
        self.stroke.set_width(w);
    }
    pub fn point_radius(&self) -> f64 {
        self.poly.point_radius()
    }
    pub fn set_point_radius(&mut self, r: f64) {
        self.poly.set_point_radius(r);
    }

    pub fn curve(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        *self.poly.xn_mut(0) = x1;
        *self.poly.yn_mut(0) = y1;
        *self.poly.xn_mut(1) = x2;
        *self.poly.yn_mut(1) = y2;
        *self.poly.xn_mut(2) = x3;
        *self.poly.yn_mut(2) = y3;
        self.curve_default();
    }

    pub fn curve_default(&mut self) -> &mut Curve3 {
        self.curve.init(
            self.poly.xn(0),
            self.poly.yn(0),
            self.poly.xn(1),
            self.poly.yn(1),
            self.poly.xn(2),
            self.poly.yn(2),
        );
        &mut self.curve
    }
}

impl<'a, C: Color> Ctrl for Curve3Ctrl<'a, C> {
    fn num_paths(&self) -> u32 {
        6
    }

	fn set_transform(&mut self, mtx: &agg::TransAffine) {
		self.ctrl.set_transform(&mtx);
	}

    fn in_rect(&self, _x: f64, _y: f64) -> bool {
        false
    }

    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool {
		let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        self.poly.on_mouse_button_down(x, y)
    }

    fn on_mouse_move(&mut self, x: f64, y: f64, button_flag: bool) -> bool {
		let (mut x, mut y) = (x, y);
        self.inverse_transform_xy(&mut x, &mut y);
        self.poly.on_mouse_move(x, y, button_flag)
    }

    fn on_mouse_button_up(&mut self, x: f64, y: f64) -> bool {
        self.poly.on_mouse_button_up(x, y)
    }

    fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool {
        self.poly.on_arrow_keys(left, right, down, up)
    }
}

impl<'a, C: Color> VertexSource for Curve3Ctrl<'a, C> {
    fn rewind(&mut self, idx: u32) {
        self.idx = idx;

        match idx {
            0 => {
                self.curve.init(
                    self.poly.xn(0),
                    self.poly.yn(0),
                    (self.poly.xn(0) + self.poly.xn(1)) * 0.5,
                    (self.poly.yn(0) + self.poly.yn(1)) * 0.5,
                    self.poly.xn(1),
                    self.poly.yn(1),
                );
                self.stroke.rewind(0);
            }
            1 => {
                self.curve.init(
                    self.poly.xn(1),
                    self.poly.yn(1),
                    (self.poly.xn(1) + self.poly.xn(2)) * 0.5,
                    (self.poly.yn(1) + self.poly.yn(2)) * 0.5,
                    self.poly.xn(2),
                    self.poly.yn(2),
                );
                self.stroke.rewind(0);
            }
            2 => {
                self.curve.init(
                    self.poly.xn(0),
                    self.poly.yn(0),
                    self.poly.xn(1),
                    self.poly.yn(1),
                    self.poly.xn(2),
                    self.poly.yn(2),
                );
                self.stroke.rewind(0);
            }
            3 => {
                self.ellipse.init(
                    self.poly.xn(0),
                    self.poly.yn(0),
                    self.point_radius(),
                    self.point_radius(),
                    20,
					false,
                );
                self.ellipse.rewind(0);
            }
            4 => {
                self.ellipse.init(
                    self.poly.xn(1),
                    self.poly.yn(1),
                    self.point_radius(),
                    self.point_radius(),
                    20,
					false,
                );
                self.ellipse.rewind(0);
            }
            5 => {
                self.ellipse.init(
                    self.poly.xn(2),
                    self.poly.yn(2),
                    self.point_radius(),
                    self.point_radius(),
                    20,
					false,
                );
                self.ellipse.rewind(0);
            }
            _ => {}
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::Stop as u32;
        match self.idx {
            0 | 1 | 2 => {
                cmd = self.stroke.vertex(x, y);
            }
            3 | 4 | 5 | 6 => {
                cmd = self.ellipse.vertex(x, y);
            }
            _ => {}
        }

        if !is_stop(cmd) {
            self.transform_xy(x, y);
        }
        cmd
    }
}

impl<'a, C: Color> CtrlColor for Curve3Ctrl<'a, C> {
    type Col = C;
    fn color(&self, i: u32) -> C {
        match i {
            _ => self.line_color,
        }
    }
}

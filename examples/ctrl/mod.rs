#![allow(dead_code)]

pub mod bezier;
pub mod cbox;
pub mod gamma;
pub mod gamma_spline;
pub mod polygon;
pub mod rbox;
pub mod scale;
pub mod slider;
pub mod spline;

//
// Function render_ctrl
//
//----------------------------------------------------------------------------

use agg::{
    Color, RasterScanLine, Renderer, RendererScanlineColor, Scanline, Transformer, VertexSource,
};
//use agg::agg::TransAffine::*;
pub use slider::*;
//use agg::renderer_scanline::*;
//use std::{rc::Rc, cell::RefCell};

pub trait Ctrl {
    fn num_paths(&self) -> u32;
    fn set_transform(&mut self, mtx: &agg::TransAffine);
    fn in_rect(&self, x: f64, y: f64) -> bool;
    fn on_mouse_button_down(&mut self, x: f64, y: f64) -> bool;
    fn on_mouse_button_up(&mut self, x: f64, y: f64) -> bool;
    fn on_mouse_move(&mut self, x: f64, y: f64, button_flag: bool) -> bool;
    fn on_arrow_keys(&mut self, left: bool, right: bool, down: bool, up: bool) -> bool;
}

pub trait CtrlColor: Ctrl {
    type Col: Color;
    fn color(&self, i: u32) -> Self::Col;
}

pub struct CtrlBase {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    flip_y: bool,
    mtx: Option<*const agg::TransAffine>,
}

impl CtrlBase {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, flip_y: bool) -> Self {
        CtrlBase {
            x1: x1,
            y1: y1,
            x2: x2,
            y2: y2,
            flip_y: flip_y,
            mtx: None,
        }
    }

    pub fn set_transform(&mut self, mtx: &agg::TransAffine) {
        self.mtx = Some(mtx as *const _);
    }

    pub fn no_transform(&mut self) {
        self.mtx = None;
    }

    pub fn transform_xy(&mut self, x: &mut f64, y: &mut f64) {
        if self.flip_y {
            *y = self.y1 + self.y2 - *y;
        }
        if let Some(mtx) = self.mtx {
            unsafe { (*mtx).transform(x, y) };
        }
    }

    pub fn inverse_transform_xy(&self, x: &mut f64, y: &mut f64) {
        if let Some(mtx) = self.mtx {
            unsafe { (*mtx).inverse_transform(x, y) };
        }
        if self.flip_y {
            *y = self.y1 + self.y2 - *y;
        }
    }

    pub fn scale(&self) -> f64 {
        if let Some(mtx) = self.mtx {
            unsafe { (*mtx).scale() }
        } else {
            1.0
        }
    }
}

pub fn render_ctrl<
    Ras: RasterScanLine,
    Sl: Scanline,
    Ren: Renderer,
    Ct: CtrlColor + VertexSource,
>(
    ras: &mut Ras, sl: &mut Sl, r: &mut Ren, c: &mut Ct,
) where Ren::C : From<Ct::Col>{
    for i in 0..c.num_paths() {
        ras.reset();
        ras.add_path(c, i);
        agg::render_scanlines_aa_solid(ras, sl, r, &c.color(i).into());
    }
}

pub fn render_ctrl_rs<
    Ras: RasterScanLine,
    Sl: Scanline,
    Rensl: RendererScanlineColor,
    Ct: CtrlColor<Col = Rensl::C> + VertexSource,
>(
    ras: &mut Ras, sl: &mut Sl, r: &mut Rensl, c: &mut Ct,
) {
    for i in 0..c.num_paths() {
        ras.reset();
        ras.add_path(c, i);
        r.set_color(c.color(i));
        agg::render_scanlines(ras, sl, r);
    }
}

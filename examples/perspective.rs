use crate::platform::*;

use agg::path_storage::*;

use agg::bounding_rect::*;
use agg::rasterizer_scanline_aa::*;
use agg::rendering_buffer::RenderBuf;
use agg::{RasterScanLine, RendererScanlineColor};

mod ctrl;
mod platform;

use crate::ctrl::rbox::Rbox;

use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::{interactive_polygon::InteractivePolygon, parse_lion::*, pixel_formats::*};

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;
const G_PATH_IDX_LENGTH: usize = 100;

struct Application {
    pub quad: InteractivePolygon<'static>,
    trans_type: Ptr<Rbox<'static, agg::Rgba8>>,

    rasterizer: RasterizerScanlineAa,
    scanline: agg::ScanlineP8,
    path: agg::PathStorage,
    colors: [agg::Rgba8; G_PATH_IDX_LENGTH],
    path_idx: [u32; G_PATH_IDX_LENGTH],
    npaths: u32,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    base_dx: f64,
    base_dy: f64,
    angle: f64,
    scale: f64,
    skew_x: f64,
    skew_y: f64,
    nclick: i32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn parse_lion(&mut self) -> u32 {
        self.npaths = parse_lion(&mut self.path, &mut self.colors, &mut self.path_idx);
        bounding_rect(
            &mut self.path,
            self.path_idx,
            0,
            self.npaths,
            &mut self.x1,
            &mut self.y1,
            &mut self.x2,
            &mut self.y2,
        );
        self.base_dx = (self.x2 - self.x1) / 2.0;
        self.base_dy = (self.y2 - self.y1) / 2.0;
        self.path.flip_x(self.x1, self.x2);
        self.path.flip_y(self.y1, self.y2);
        self.npaths
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let trans_type = ctrl_ptr(Rbox::new(420., 5.0, 420. + 130.0, 55.0, !flip_y));
        trans_type.borrow_mut().add_item("Bilinear");
        trans_type.borrow_mut().add_item("Perspective");
        trans_type.borrow_mut().set_cur_item(0);
        let mut app = Self {
            rasterizer: RasterizerScanlineAa::new(),
            trans_type: trans_type.clone(),
            scanline: agg::ScanlineP8::new(),
            path: PathStorage::new(),
            colors: [agg::Rgba8::default(); 100],
            path_idx: [0; 100],
            npaths: 0,
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            base_dx: 0.0,
            base_dy: 0.0,
            angle: 0.0,
            scale: 1.0,
            skew_x: 0.0,
            skew_y: 0.0,
            nclick: 0,
            quad: InteractivePolygon::new(4, 5.0),
            ctrls: CtrlContainer {
                ctrl: vec![trans_type],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        };

        app.parse_lion();
        *app.quad.xn_mut(0) = app.x1;
        *app.quad.yn_mut(0) = app.y1;
        *app.quad.xn_mut(1) = app.x2;
        *app.quad.yn_mut(1) = app.y1;
        *app.quad.xn_mut(2) = app.x2;
        *app.quad.yn_mut(2) = app.y2;
        *app.quad.xn_mut(3) = app.x1;
        *app.quad.yn_mut(3) = app.y2;
        app
    }

    fn on_init(&mut self) {
        let dx = self.util.borrow().width() / 2.0 - (self.quad.xn(1) - self.quad.xn(0)) / 2.0;
        let dy = self.util.borrow().height() / 2.0 - (self.quad.yn(2) - self.quad.yn(0)) / 2.0;
        *self.quad.xn_mut(0) += dx;
        *self.quad.yn_mut(0) += dy;
        *self.quad.xn_mut(1) += dx;
        *self.quad.yn_mut(1) += dy;
        *self.quad.xn_mut(2) += dx;
        *self.quad.yn_mut(2) += dy;
        *self.quad.xn_mut(3) += dx;
        *self.quad.yn_mut(3) += dy;
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.quad.on_mouse_button_down(x as f64, y as f64) {
                return Draw::Yes;
            }
        }
        return Draw::No;
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.quad.on_mouse_move(x as f64, y as f64) {
                return Draw::Yes;
            }
        }
        if flags & InputFlag::MouseLeft as u32 == 0 {
            return self.on_mouse_button_up(rb, x, y, flags);
        }
        return Draw::No;
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, _flags: u32,
    ) -> Draw {
        if self.quad.on_mouse_button_up(x as f64, y as f64) {
            return Draw::Yes;
        }
        return Draw::No;
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        self.rasterizer.clip_box(
            0.,
            0.,
            self.util.borrow().width(),
            self.util.borrow().height(),
        );

        if self.trans_type.borrow().cur_item() == 0 {
            let tr = agg::TransBilinear::new_rect_to_quad(
                self.x1,
                self.y1,
                self.x2,
                self.y2,
                self.quad.polygon(),
            );

            if tr.is_valid() {
                // Render transformed lion
                //

                let mut trans = agg::ConvTransform::new_borrowed(&mut self.path, tr);

                agg::render_all_paths(
                    &mut self.rasterizer,
                    &mut self.scanline,
                    &mut r,
                    &mut trans,
                    &self.colors,
                    &self.path_idx,
                    self.npaths,
                );

                // Render transformed ellipse
                //
                let mut ell = agg::Ellipse::new_ellipse(
                    (self.x1 + self.x2) * 0.5,
                    (self.y1 + self.y2) * 0.5,
                    (self.x2 - self.x1) * 0.5,
                    (self.y2 - self.y1) * 0.5,
                    200,
                    false,
                );

                let mut trans_ell = agg::ConvTransform::new_borrowed(&mut ell, tr.clone());

                self.rasterizer.add_path(&mut trans_ell, 0);
                r.set_color(ColorType::new_params(125, 75, 0, 75));
                agg::render_scanlines(&mut self.rasterizer, &mut self.scanline, &mut r);

                let mut ell_stroke: agg::ConvStroke<agg::Ellipse> =
                    agg::ConvStroke::new_borrowed(&mut ell);
                ell_stroke.set_width(3.0);
                let mut trans_ell_stroke = agg::ConvTransform::new_owned(ell_stroke, tr);
                self.rasterizer.add_path(&mut trans_ell_stroke, 0);
                r.set_color(ColorType::new_params(0, 75, 50, 255));
                agg::render_scanlines(&mut self.rasterizer, &mut self.scanline, &mut r);
            }
        } else {
            let tr = agg::TransPerspective::new_rect_to_quad(
                self.x1,
                self.y1,
                self.x2,
                self.y2,
                self.quad.polygon(),
            );

            if tr.is_valid(agg::trans_affine::AFFINE_EPSILON) {
                // Render transformed lion
                //
                let mut trans = agg::ConvTransform::new_borrowed(&mut self.path, tr);

                agg::render_all_paths(
                    &mut self.rasterizer,
                    &mut self.scanline,
                    &mut r,
                    &mut trans,
                    &self.colors,
                    &self.path_idx,
                    self.npaths,
                );
                //self.path = trans.set_source(PathStorage::new());
                // Render transformed ellipse
                //
                let ell = agg::Ellipse::new_ellipse(
                    (self.x1 + self.x2) * 0.5,
                    (self.y1 + self.y2) * 0.5,
                    (self.x2 - self.x1) * 0.5,
                    (self.y2 - self.y1) * 0.5,
                    200,
                    false,
                );
                let mut trans_ell = agg::ConvTransform::new_owned(ell, tr);

                self.rasterizer.add_path(&mut trans_ell, 0);
                r.set_color(ColorType::new_params(125, 75, 0, 75));
                agg::render_scanlines(&mut self.rasterizer, &mut self.scanline, &mut r);

                let ell = agg::Ellipse::new_ellipse(
                    (self.x1 + self.x2) * 0.5,
                    (self.y1 + self.y2) * 0.5,
                    (self.x2 - self.x1) * 0.5,
                    (self.y2 - self.y1) * 0.5,
                    200,
                    false,
                );

                let mut ell_stroke: agg::ConvStroke<_> = agg::ConvStroke::new_owned(ell);
                ell_stroke.set_width(3.0);
                let mut trans_ell_stroke = agg::ConvTransform::new_owned(ell_stroke, tr);
                self.rasterizer.add_path(&mut trans_ell_stroke, 0);
                r.set_color(ColorType::new_params(0, 75, 50, 255));
                agg::render_scanlines(&mut self.rasterizer, &mut self.scanline, &mut r);
            }
        }

        // Render the "quad" tool and controls
        self.rasterizer.add_path(&mut self.quad, 0);
        r.set_color(ColorType::new_params(0, 75, 125, 50));
        agg::render_scanlines(&mut self.rasterizer, &mut self.scanline, &mut r);
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.trans_type.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Perspective Transformations");

    if plat.init(600, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

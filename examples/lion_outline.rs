use crate::ctrl::cbox::*;
use crate::ctrl::slider::*;
use crate::platform::*;
//use agg::basics::{RectD, uround};
use agg::color_rgba::*;
use agg::conv_stroke::*;
use agg::conv_transform::ConvTransform;
use agg::path_storage::*;
use agg::pixfmt_rgb::*;
use agg::rasterizer_outline_aa::RasterizerOutlineAa;
use agg::rasterizer_scanline_aa::*;
use agg::renderer_base::RendererBase;
use agg::renderer_outline_aa::{LineProfileAA, RendererOutlineAa};
use agg::renderer_scanline::render_all_paths;
use agg::rendering_buffer::{RenderBuf, RowAccessBuf};
use agg::scanline_p::*;
//use agg::line_aa_basics::*;
use agg::bounding_rect::*;
use agg::math_stroke::LineJoin;
use agg::trans_affine::TransAffine;
//use agg::trans_perspective::TransPerspective;
//use ctrl::render_ctrl;

//use agg::{Color, RasterScanLine};
mod ctrl;
mod platform;

use core::f64::consts::PI;
use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use crate::misc::parse_lion::*;
type Ptr<T> = Rc<RefCell<T>>;
fn crt_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

type RasRen<'a> = RendererBase<
    'a,
    AlphaBlendRgb<'a, Rgba8, OrderBgr, BlenderRgb<Rgba8, OrderBgr>, RowAccessBuf<u8>>,
>;

const FLIP_Y: bool = true;

struct Application {
    width_slider: Ptr<Slider<'static, agg::Rgba8>>,
    cbscanline: Ptr<Cbox<'static, agg::Rgba8>>,
    rasterizer: RasterizerScanlineAa,
    scanline: ScanlineP8,
    path: PathStorage,
    colors: [agg::Rgba8; 100],
    path_idx: [u32; 100],
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
        self.npaths
    }

    pub fn transform(&mut self, width: f64, height: f64, x: f64, y: f64) {
        let (mut x, mut y) = (x, y);

        x -= width / 2.;
        y -= height / 2.;
        self.angle = y.atan2(x);
        self.scale = (y * y + x * x).sqrt() / 100.0;
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let width = self.util.borrow().width();
        let height = self.util.borrow().width();

        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut mtx = TransAffine::new_default();
        mtx.multiply(&TransAffine::trans_affine_translation(
            -self.base_dx,
            -self.base_dy,
        ));
        mtx.multiply(&TransAffine::trans_affine_scaling(self.scale, self.scale));
        mtx.multiply(&TransAffine::trans_affine_rotation(self.angle + PI));
        mtx.multiply(&TransAffine::trans_affine_skewing(
            self.skew_x / 1000.0,
            self.skew_y / 1000.0,
        ));
        mtx.multiply(&TransAffine::trans_affine_translation(
            width / 2.,
            height / 2.,
        ));

        if self.cbscanline.borrow().status() {
            let mut stroke: ConvStroke<'_, _> = ConvStroke::new_borrowed(&mut self.path);
            stroke.set_width(self.width_slider.borrow().value());
            stroke.set_line_join(LineJoin::Round);
            let mut trans = ConvTransform::new_owned(stroke, mtx);
            let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
            render_all_paths(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut r,
                &mut trans,
                &self.colors,
                &self.path_idx,
                self.npaths,
            );
        } else {
            let w = self.width_slider.borrow().value() * mtx.scale();
            let profile = LineProfileAA::new_gamma(w, agg::GammaNone);
            let mut ren = RendererOutlineAa::new(&mut rb, profile);
            let mut ras = RasterizerOutlineAa::<RendererOutlineAa<RasRen>>::new(&mut ren);
            let mut trans = ConvTransform::new_borrowed(&mut self.path, mtx);
            ras.render_all_paths(&mut trans, &self.colors, &self.path_idx, self.npaths);
        }

        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.width_slider.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.cbscanline.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        let mut r = Draw::No;
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let width = self.util.borrow().width();
            let height = self.util.borrow().width();
            self.transform(width as f64, height as f64, x as f64, y as f64);
            r = Draw::Yes;
        }

        if flags & InputFlag::MouseRight as u32 != 0 {
            self.skew_x = x as f64;
            self.skew_y = y as f64;
            r = Draw::Yes;
        }
        r
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No //self.on_mouse_button_down(x, y, flags)
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Application {
        let width_slider = crt_ptr(Slider::new(5., 5., 150., 12., !flip_y));
        let cbscanline = crt_ptr(Cbox::new(160., 5., "Use Scanline Rasterizer", !flip_y));

        width_slider.borrow_mut().no_transform();
        width_slider.borrow_mut().set_range(0.0, 4.0);
        width_slider.borrow_mut().set_value(1.0);
        width_slider.borrow_mut().set_label("Width %3.2f");

        cbscanline.borrow_mut().no_transform();

        let mut app = Application {
            width_slider: width_slider.clone(),
            cbscanline: cbscanline.clone(),
            rasterizer: RasterizerScanlineAa::new(),
            scanline: ScanlineP8::new(),
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
            ctrls: CtrlContainer {
                ctrl: vec![width_slider, cbscanline],
                cur_ctrl: -1,
                num_ctrl: 2,
            },
            util: util,
        };
        app.parse_lion();
        app
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption(r#"AGG Example. Lion"#);

    if plat.init(512, 512, WindowFlag::Resize as u32) {
        plat.run();
    }
}

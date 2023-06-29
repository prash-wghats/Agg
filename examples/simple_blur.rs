use crate::platform::*;

use agg::color_rgba::OrderBgr;
use agg::{Order, RasterScanLine, RenderBuf, RenderBuffer, RendererScanlineColor, SpanGenerator};

mod ctrl;
mod platform;

use std::cell::RefCell;

use std::f64::consts::PI;
use std::marker::PhantomData;
use std::rc::Rc;

mod misc;
use misc::parse_lion::*;

const FLIP_Y: bool = true;

struct SpanSimpleBlurRGB24<Ord: Order> {
    source_image: agg::RenderBuf,
    dum: PhantomData<Ord>,
}
impl<Ord: Order> SpanGenerator for SpanSimpleBlurRGB24<Ord> {
    type C = agg::Rgba8;
    fn prepare(&mut self) {}

    fn generate(&mut self, span: &mut [Self::C], x: i32, y: i32, len: u32) {
        let mut x = x;
        if y < 1 || y >= self.source_image.height() as i32 - 1 {
            for i in 0..len as usize {
                span[i] = agg::Rgba8 {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                };
            }
            return;
        }

        for i in 0..len {
            let mut color: [i32; 4] = [0, 0, 0, 0];

            if x > 0 && x < self.source_image.width() as i32 - 1 {
                for i in (1..4).rev() {
                    let ptr = self.source_image.row(y - i + 2);
                    let off = (x as usize - 1) * 3;

                    color[0] += ptr[off + 0] as i32;
                    color[1] += ptr[off + 1] as i32;
                    color[2] += ptr[off + 2] as i32;
                    color[3] += 255;

                    color[0] += ptr[off + 3] as i32;
                    color[1] += ptr[off + 4] as i32;
                    color[2] += ptr[off + 5] as i32;
                    color[3] += 255;

                    color[0] += ptr[off + 6] as i32;
                    color[1] += ptr[off + 7] as i32;
                    color[2] += ptr[off + 8] as i32;
                    color[3] += 255;
                }

                color[0] /= 9;
                color[1] /= 9;
                color[2] /= 9;
                color[3] /= 9;
            }

            span[i as usize] = agg::Rgba8 {
                r: color[Ord::R] as u8,
                g: color[Ord::G] as u8,
                b: color[Ord::B] as u8,
                a: color[Ord::A] as u8,
            };
            x += 1;
        }
    }
}

struct Application {
    cx: f64,
    cy: f64,
    path: agg::PathStorage,
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
    _nclick: i32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn parse_lion(&mut self) -> u32 {
        self.npaths = parse_lion(&mut self.path, &mut self.colors, &mut self.path_idx);
        //let path_idx_adaptor = agg::pod_array_adaptor::<u32>(path_idx, 100);
        agg::bounding_rect(
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
}
impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, _flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut app = Self {
            cx: 100.0,
            cy: 102.0,
            path: agg::PathStorage::new(),
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
            _nclick: 0,
            ctrls: CtrlContainer {
                ctrl: vec![],
                cur_ctrl: -1,
                num_ctrl: 0,
            },
            util: util,
        };

        app.parse_lion();
        app
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.cx = x as f64;
            self.cy = y as f64;
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        return self.on_mouse_button_down(rb, x, y, flags);
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgr24::new_owned(rbuf.clone());
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut sl = agg::ScanlineP8::new();
        let mut mtx = agg::TransAffine::new_default();

        mtx *= agg::TransAffine::trans_affine_translation(-self.base_dx, -self.base_dy);
        mtx *= agg::TransAffine::trans_affine_scaling(self.scale, self.scale);
        mtx *= agg::TransAffine::trans_affine_rotation(self.angle + PI);
        mtx *= agg::TransAffine::trans_affine_skewing(self.skew_x / 1000.0, self.skew_y / 1000.0);
        mtx *= agg::TransAffine::trans_affine_translation(
            self.util.borrow().initial_width() / 4.,
            self.util.borrow().initial_height() / 2.,
        );
        mtx *= *self.util.borrow_mut().trans_affine_resizing();

        let mut trans = agg::ConvTransform::new_borrowed(&mut self.path, mtx);
        let mut ras2: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl2 = agg::ScanlineU8::new();
        {
            let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
            agg::render_all_paths(
                &mut ras2,
                &mut sl,
                &mut rs,
                &mut trans,
                &self.colors,
                &self.path_idx,
                self.npaths,
            );
        }
        let mut t = *self.util.borrow_mut().trans_affine_resizing();
        t.invert();
        *trans.trans_mut() *= t;
        *trans.trans_mut() *=
            agg::TransAffine::trans_affine_translation(self.util.borrow().initial_width() / 2., 0.);
        *trans.trans_mut() *= *self.util.borrow_mut().trans_affine_resizing();

        let mut profile = agg::LineProfileAA::new();
        profile.set_width(1.0);
        let mut rp = agg::RendererOutlineAa::new(&mut rb, profile);
        let mut ras: agg::RasterizerOutlineAa<'_, _> = agg::RasterizerOutlineAa::new(&mut rp);
        ras.set_round_cap(true);

        ras.render_all_paths(&mut trans, &self.colors, &self.path_idx, self.npaths);

        let ell = agg::Ellipse::new_ellipse(self.cx, self.cy, 100.0, 100.0, 100, false);
        let mut ell_stroke1: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(ell);
        ell_stroke1.set_width(6.0);
        let mut ell_stroke2: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(ell_stroke1);

        ell_stroke2.set_width(2.0);
        {
            let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
            rs.set_color(agg::Rgba8::new_params(0, 50, 0, 255));
            ras2.add_path(&mut ell_stroke2, 0);
            agg::render_scanlines(&mut ras2, &mut sl, &mut rs);
        }

        let mut sa = agg::VecSpan::new();
        let mut sg: SpanSimpleBlurRGB24<OrderBgr> = SpanSimpleBlurRGB24 {
            source_image: self.util.borrow_mut().rbuf_img(0).clone(),
            dum: PhantomData,
        };

        ras2.add_path(ell_stroke2.source_mut().source_mut(), 0);

        self.util.borrow_mut().copy_window_to_img(rbuf, 0);
        agg::render_scanlines_aa(&mut ras2, &mut sl2, &mut rb, &mut sa, &mut sg);

        // More blur if desired :-)
        // copy_window_to_img(0);
        // agg::render_scanlines(&ras2, &mut sl2, &mut rblur);
        // copy_window_to_img(0);
        // agg::render_scanlines(&ras2, &mut sl2, &mut rblur);
        // copy_window_to_img(0);
        // agg::render_scanlines(&ras2, &mut sl2, &mut rblur);
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Lion with Alpha-Masking");

    if plat.init(512, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

use crate::platform::*;
//use agg::basics::{RectD, uround};
use agg::alpha_mask_u8::*;

use agg::conv_transform::ConvTransform;
use agg::path_storage::*;
use agg::pixfmt_gray::*;

use agg::bounding_rect::*;
use agg::rasterizer_scanline_aa::*;
use agg::renderer_scanline::render_all_paths;
use agg::rendering_buffer::RenderBuf;
use agg::scanline_u::*;
use agg::{RasterScanLine, RenderBuffer, RendererScanlineColor};

mod ctrl;
mod platform;

use core::f64::consts::PI;
use libc::*;
use std::cell::RefCell;
use std::ptr::null_mut;
use std::rc::Rc;

mod misc;
use crate::misc::parse_lion::*;

type Ptr<T> = Rc<RefCell<T>>;

fn frand() -> i32 {
    unsafe { rand() }
}

const FLIP_Y: bool = true;
const G_PATH_IDX_LENGTH: usize = 100;

struct Application {
    alpha_buf: *mut u8,
    _alpha_rbuf: agg::RenderBuf,
    alpha_mask: AlphaMaskGray8,
    ras: RasterizerScanlineAa,
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
    _nclick: i32,
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

    fn generate_alpha_mask(&mut self, cx: i32, cy: i32) {
        self.alpha_buf = unsafe {
            std::alloc::alloc(
                std::alloc::Layout::from_size_align(cx as usize * cy as usize, 1).unwrap(),
            )
        };
        self.alpha_mask
            .rbuf_mut()
            .attach(self.alpha_buf, cx as u32, cy as u32, cx);

        let mut pixf = PixGray8::new_borrowed(self.alpha_mask.rbuf_mut());
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&agg::Gray8::new_params(0, 255));
        let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
        let mut sl = agg::ScanlineP8::new();

        let mut ell = agg::Ellipse::new();
        for _ in 0..10 {
            ell.init(
                (frand() % cx) as f64,
                (frand() % cy) as f64,
                ((frand() % 100) + 20) as f64,
                ((frand() % 100) + 20) as f64,
                100,
    false,
            );
            self.ras.add_path(&mut ell, 0);
            r.set_color(agg::Gray8::new_params(
                frand() as u32 & 0xFF,
                frand() as u32 & 0xFF,
            ));
            agg::render_scanlines(&mut self.ras, &mut sl, &mut r);
        }
    }

    fn transform(&mut self, width: f64, height: f64, x: f64, y: f64) {
        let x = x - (width / 2.0);
        let y = y - (height / 2.0);
        self.angle = y.atan2(x);
        self.scale = (y * y + x * x).sqrt() / 100.0;
    }
}
impl Interface for Application {
    fn new(_format: PixFormat, _flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let mut app = Self {
            alpha_buf: null_mut(),
            _alpha_rbuf: agg::RenderBuf::new_default(),
            alpha_mask: AlphaMaskGray8::new(RenderBuf::new_default()),
            ras: RasterizerScanlineAa::new(),
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
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_resize(&mut self, cx: i32, cy: i32) {
        self.generate_alpha_mask(cx, cy);
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let width = self.util.borrow().width();
        let height = self.util.borrow().width();

        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pix);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
        let mut sl = ScanlineU8AM::new(&mut self.alpha_mask);

        let mut mtx = agg::TransAffine::new_default();
        mtx *= agg::TransAffine::trans_affine_translation(-self.base_dx, -self.base_dy);
        mtx *= agg::TransAffine::trans_affine_scaling(self.scale, self.scale);
        mtx *= agg::TransAffine::trans_affine_rotation(self.angle + PI);
        mtx *= agg::TransAffine::trans_affine_skewing(self.skew_x / 1000.0, self.skew_y / 1000.0);
        mtx *= agg::TransAffine::trans_affine_translation(width / 2., height / 2.);

        let mut trans = ConvTransform::new_borrowed(&mut self.path, mtx);
        render_all_paths(
            &mut self.ras,
            &mut sl,
            &mut r,
            &mut trans,
            &self.colors,
            &self.path_idx,
            self.npaths,
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

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        self.on_mouse_button_down(rb, x, y, flags)
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption(r#"AGG Example. Lion with Alpha Masking"#);

    if plat.init(512, 512, WindowFlag::Resize as u32) {
        plat.run();
    }
}

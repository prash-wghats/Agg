use crate::platform::*;

use agg::{RasterScanLine, RenderBuf, RenderBuffer, SpanConverter};

mod ctrl;
mod platform;

use crate::ctrl::spline::Spline;

use libc::*;
use std::cell::RefCell;

use std::f64::consts::PI;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;
use wrapping_arithmetic::wrappit;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

fn frand() -> u32 {
    unsafe { rand() as u32 }
}

const FLIP_Y: bool = true;

pub struct SpanConvBrightnessAlphaRgb8<'a> {
    alpha_array: &'a [u8],
}

impl<'a> SpanConvBrightnessAlphaRgb8<'a> {
    pub const ARRAY_SIZE: usize = 256 * 3;

    pub fn new(alpha_array: &'a [u8]) -> Self {
        SpanConvBrightnessAlphaRgb8 { alpha_array }
    }
}
impl<'a> SpanConverter for SpanConvBrightnessAlphaRgb8<'a> {
    type C = agg::Rgba8;

    fn prepare(&mut self) {}

    #[wrappit]
    fn generate(&mut self, span: &mut [agg::Rgba8], _x: i32, _y: i32, len: u32) {
        for i in 0..len as usize {
            let j = (span[i].r + span[i].g + span[i].b) as usize;
            span[i].a = self.alpha_array[j];
        }
    }
}

pub struct Application {
    alpha: Ptr<Spline<'static, agg::Rgba8>>,
    x: [f64; 50],
    y: [f64; 50],
    rx: [f64; 50],
    ry: [f64; 50],
    colors: [agg::Rgba8; 50],
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let alpha = ctrl_ptr(Spline::new(2., 2., 200., 30., 6, !flip_y));
        alpha.borrow_mut().set_value(0, 1.0);
        alpha.borrow_mut().set_value(1, 1.0);
        alpha.borrow_mut().set_value(2, 1.0);
        alpha.borrow_mut().set_value(3, 0.5);
        alpha.borrow_mut().set_value(4, 0.5);
        alpha.borrow_mut().set_value(5, 1.0);
        alpha.borrow_mut().update_spline();

        Application {
            alpha: alpha.clone(),
            x: [0.0; 50],
            y: [0.0; 50],
            rx: [0.0; 50],
            ry: [0.0; 50],
            colors: [agg::Rgba8::new_params(0, 0, 0, 0); 50],
            ctrls: CtrlContainer {
                ctrl: vec![alpha],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        for i in 0..50 {
            self.x[i] = (frand() % self.util.borrow().width() as u32) as f64;
            self.y[i] = (frand() % self.util.borrow().height() as u32) as f64;
            self.rx[i] = (frand() % 60 + 10) as f64;
            self.ry[i] = (frand() % 60 + 10) as f64;
            let a = frand() & 0xFF;
            let b = frand() & 0xFF;
            let g = frand() & 0xFF;
            let r = frand() & 0xFF;
            self.colors[i] = agg::Rgba8::new_params(r, g, b, a);
        }
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        if key == ' ' as u32 {
            if let Ok(mut fd) = File::create("alpha") {
                for i in 0..SpanConvBrightnessAlphaRgb8::ARRAY_SIZE {
                    let alpha = (self
                        .alpha
                        .borrow_mut()
                        .value(i as f64 / SpanConvBrightnessAlphaRgb8::ARRAY_SIZE as f64)
                        * 255.0) as u8;
                    if i % 32 == 0 {
                        writeln!(fd, "").unwrap();
                    }
                    write!(fd, "{:3}, ", alpha).unwrap();
                }
            }
        }
        Draw::No
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut src_mtx = agg::TransAffine::new_default();
        src_mtx *= agg::TransAffine::trans_affine_translation(
            -(self.util.borrow().initial_width() as f64) / 2.0,
            -(self.util.borrow().initial_height() as f64) / 2.0,
        );
        src_mtx *= agg::TransAffine::trans_affine_rotation(10.0 * PI / 180.0);
        src_mtx *= agg::TransAffine::trans_affine_translation(
            (self.util.borrow().initial_width() as f64) / 2.0,
            (self.util.borrow().initial_height() as f64) / 2.0,
        );
        src_mtx *= *self.util.borrow_mut().trans_affine_resizing();

        let mut img_mtx = src_mtx;
        img_mtx.invert();

        let mut brightness_alpha_array = [0; SpanConvBrightnessAlphaRgb8::ARRAY_SIZE];

        for i in 0..SpanConvBrightnessAlphaRgb8::ARRAY_SIZE {
            brightness_alpha_array[i] = (self
                .alpha
                .borrow_mut()
                .value(i as f64 / SpanConvBrightnessAlphaRgb8::ARRAY_SIZE as f64)
                * 255.0) as u8;
        }
        let mut color_alpha = SpanConvBrightnessAlphaRgb8::new(&brightness_alpha_array);
        let mut sa = agg::VecSpan::new();
        let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(img_mtx);
        let img_pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        let mut img_src =
            agg::ImageAccessorClip::new(img_pixf, &agg::Rgba8::new_params(0, 0, 0, 255));
        let mut sg = agg::SpanImageFilterRgbBilinear::new(&mut img_src, &mut interpolator);
        let mut sc = agg::SpanProcess::new(&mut sg, &mut color_alpha);
        let mut ell = agg::Ellipse::new();

        for i in 0..50 {
            ell.init(self.x[i], self.y[i], self.rx[i], self.ry[i], 50, false);
            ras.add_path(&mut ell, 0);
            agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &self.colors[i]);
        }

        ell.init(
            (self.util.borrow().initial_width() as f64) / 2.0,
            (self.util.borrow().initial_height() as f64) / 2.0,
            (self.util.borrow().initial_width() as f64) / 1.9,
            (self.util.borrow().initial_height() as f64) / 1.9,
            200,
            false,
        );

        let mut tr = agg::ConvTransform::new_owned(ell, src_mtx);

        ras.add_path(&mut tr, 0);
        agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb, &mut sa, &mut sc);

        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.alpha.borrow_mut());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "spheres";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);

    let buf;
	let ext = (plat.app_mut().util.borrow().img_ext()).to_string();
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("spheres") {
            buf = format!(
                "File not found: {}{}. Download http://www.antigrain.com/{}{}
				or copy it from another directory if available.",
                img_name, ext, img_name, ext
            );
        } else {
            buf = format!("File not found: {}{}", img_name, ext);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
        return;
    }

    plat.set_caption("Image Affine Transformations with Alpha-function");
    let w = plat.app_mut().util.borrow_mut().rbuf_img(0).width();
    let h = plat.app_mut().util.borrow_mut().rbuf_img(0).height();
    if plat.init(w, h, WindowFlag::Resize as u32) {
        plat.run();
    }
}

use agg::bounding_rect::*;
use agg::color_rgba::*;
use agg::renderer_mclip::RendererMclip;
use agg::rendering_buffer::*;
use agg::trans_affine::*;
use agg::{RasterScanLine, RenderBuffer, RenderPrim, RendererOutline};

use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use libc::*;
use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::{pixel_formats::*,parse_lion::*};

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

fn frand() -> i32 {
    unsafe { rand() }
}
/*
type ColorType = agg::Rgba8;
struct GradientLinearColor<T> {

    ColorType c1;
    ColorType c2;
}
impl
    fn new(c1: ColorType, c2: ColorType) -> Self {
        Self { c1, c2 }
    }

    fn size() -> u32 {
        256
    }

    fn operator[](v: u32) -> ColorType {
        let c = ColorType {
            r: (((self.c2.r - self.c1.r) * v) + (self.c1.r << 8)) >> 8,
            g: (((self.c2.g - self.c1.g) * v) + (self.c1.g << 8)) >> 8,
            b: (((self.c2.b - self.c1.b) * v) + (self.c1.b << 8)) >> 8,
            a: (((self.c2.a - self.c1.a) * v) + (self.c1.a << 8)) >> 8
        };
        c
    }

    fn colors(&mut self, c1: ColorType, c2: ColorType) {
        self.c1 = c1;
        self.c2 = c2;
    }
}

impl GradientLinearColor<Rgba8> {
    fn new(c1: Rgba8, c2: Rgba8) -> Self {
        Self { c1, c2 }
    }

    fn size() -> u32 {
        256
    }

    fn operator[](v: u32) -> Rgba8 {
        let c = Rgba8 {
            r: (((self.c2.r - self.c1.r) * v) + (self.c1.r << 8)) >> 8,
            g: (((self.c2.g - self.c1.g) * v) + (self.c1.g << 8)) >> 8,
            b: (((self.c2.b - self.c1.b) * v) + (self.c1.b << 8)) >> 8,
            a: (((self.c2.a - self.c1.a) * v) + (self.c1.a << 8)) >> 8
        };
        c
    }

    fn colors(&mut self, c1: Rgba8, c2: Rgba8) {
        self.c1 = c1;
        self.c2 = c2;
    }
}

impl GradientLinearColor<Gray8> {
    fn new(c1: Gray8, c2: Gray8) -> Self {
        Self { c1, c2 }
    }

    fn size() -> u32 {
        256
    }

    fn operator[](v: u32) -> Gray8 {
        let c = Gray8 {
            v: (((self.c2.v - self.c1.v) * v) + (self.c1.v << 8)) >> 8,
            a: (((self.c2.a - self.c1.a) * v) + (self.c1.a << 8)) >> 8
        };
        c
    }

    fn colors(&mut self, c1: Gray8, c2: Gray8) {
        self.c1 = c1;
        self.c2 = c2;
    }
}

*/

struct Application {
    num_cb: Ptr<Slider<'static, agg::Rgba8>>,
    rasterizer: agg::RasterizerScanlineAa,
    scanline: agg::ScanlineP8,
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

    fn transform(&mut self, width: f64, height: f64, x: f64, y: f64) {
        let (mut x, mut y) = (x, y);
        x -= width / 2.0;
        y -= height / 2.0;
        self.angle = y.atan2(x);
        self.scale = (y * y + x * x).sqrt() / 100.0;
    }
}
impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let num_cb = ctrl_ptr(Slider::new(5., 5., 150., 12., !flip_y));
        num_cb.borrow_mut().set_range(2., 10.);
        //m_num_cb.num_steps(8);
        num_cb.borrow_mut().set_label("N=%0.2f");
        num_cb.borrow_mut().no_transform();

        let mut app = Application {
            num_cb: num_cb.clone(),
            rasterizer: agg::RasterizerScanlineAa::new(),
            scanline: agg::ScanlineP8::new(),
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
            nclick: 0,
            ctrls: CtrlContainer {
                ctrl: vec![num_cb],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        };
        app.parse_lion();
        app
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_draw(&mut self, rb: &mut agg::RenderBuf) {
        let width = rb.width() as i32;
        let height = rb.height() as i32;
        unsafe {
            srand(1000);
        }
        let mut pf = Pixfmt::new_borrowed(rb);

        let mut r = RendererMclip::new(&mut pf);
        r.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut r);

        let mut mtx = agg::TransAffine::new_default();
        mtx *= agg::TransAffine::trans_affine_translation(
            -(self.base_dx as f64),
            -(self.base_dy as f64),
        );
        mtx *= agg::TransAffine::trans_affine_scaling(self.scale, self.scale);
        mtx *= agg::TransAffine::trans_affine_rotation(self.angle + std::f64::consts::PI);
        mtx *= agg::TransAffine::trans_affine_skewing(
            self.skew_x as f64 / 1000.0,
            self.skew_y as f64 / 1000.0,
        );
        mtx *= agg::TransAffine::trans_affine_translation(width as f64 / 2.0, height as f64 / 2.0);

        rs.ren_mut().reset_clipping(false);
        let n = self.num_cb.borrow().value() as i32;
        for x in 0..n {
            for y in 0..n {
                let x1 = (width as f64 * x as f64 / n as f64) as i32;
                let y1 = (height as f64 * y as f64 / n as f64) as i32;
                let x2 = (width as f64 * (x + 1) as f64 / n as f64) as i32;
                let y2 = (height as f64 * (y + 1) as f64 / n as f64) as i32;
                rs.ren_mut().add_clip_box(x1 + 5, y1 + 5, x2 - 5, y2 - 5);
            }
        }

        // Render the lion
        let mut trans = agg::ConvTransform::new_borrowed(&mut self.path, mtx);
        agg::render_all_paths(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rs,
            &mut trans,
            &self.colors,
            &self.path_idx,
            self.npaths,
        );

        // Render random Bresenham lines and markers
        {
            let mut m = agg::RendererMarkers::new(&mut r);
            for _i in 0..50 {
                m.set_line_color(agg::Rgba8::new_params(
                    (frand() & 0x7F) as u32,
                    (frand() & 0x7F) as u32,
                    (frand() & 0x7F) as u32,
                    ((frand() & 0x7F) + 0x7F) as u32,
                ));
                m.set_fill_color(agg::Rgba8::new_params(
                    frand() as u32 & 0x7F,
                    frand() as u32 & 0x7F,
                    frand() as u32 & 0x7F,
                    (frand() as u32 & 0x7F) + 0x7F,
                ));
                m.line(
                    m.coord(frand() as f64 % width as f64),
                    m.coord(frand() as f64 % height as f64),
                    m.coord(frand() as f64 % width as f64),
                    m.coord(frand() as f64 % height as f64),
                    false,
                );
                m.marker(
                    frand() % width as i32,
                    frand() % height as i32,
                    frand() % 10 + 5,
                    unsafe { std::mem::transmute(frand() % (agg::MarkerType::Pixel as i32 + 1)) },
                );
            }
        }

        // Render random anti-aliased lines
        let w = 5.0;
        let mut profile = agg::LineProfileAA::new();
        profile.set_width(w);
        let mut ren = agg::RendererOutlineAa::new(&mut r, profile);
        let mut ras = agg::RasterizerOutlineAa::<_, agg::LineCoord>::new(&mut ren);
        ras.set_round_cap(true);

        for _i in 0..50 {
            ras.ren_mut().set_color(agg::Rgba8::new_params(
                frand() as u32 & 0x7F,
                frand() as u32 & 0x7F,
                frand() as u32 & 0x7F,
                (frand() as u32 & 0x7F) + 0x7F,
            ));
            ras.move_to_d((frand() % width) as f64, (frand() % height) as f64);
            ras.line_to_d((frand() % width) as f64, (frand() % height) as f64);
            ras.render(false);
        }

        // Render random circles with gradient
        let grm = agg::TransAffine::new_default();
        let mut grf = agg::GradientCircle {};
        let mut grc = agg::GradientLinearColor::new(
            agg::Rgba8::new_params(0, 0, 0, 255),
            agg::Rgba8::new_params(0, 0, 0, 255),
            256,
        );
        let mut ell = agg::Ellipse::new();
        let mut sa = agg::VecSpan::new();
        let mut inter = agg::SpanIpLinear::new(grm);
        let mut sg = agg::SpanGradient::<
            agg::Rgba8,
            agg::SpanIpLinear<TransAffine>,
            agg::GradientCircle,
            agg::GradientLinearColor<Rgba8>,
        >::new(&mut inter, &mut grf, &mut grc, 0., 10.);
        for _i in 0..50 {
            let x = frand() % width;
            let y = frand() % height;
            let radius = (frand() % 10 + 5) as f64;

            sg.interpolator_mut().transformer_mut().reset();
            *sg.interpolator_mut().transformer_mut() *=
                agg::TransAffine::trans_affine_scaling_eq(radius / 10.0);
            *sg.interpolator_mut().transformer_mut() *=
                agg::TransAffine::trans_affine_translation(x as f64, y as f64);
            sg.interpolator_mut().transformer_mut().invert();
            sg.color_function_mut().colors(
                agg::Rgba8::new_params(255, 255, 255, 0),
                agg::Rgba8::new_params(
                    frand() as u32 & 0x7F,
                    frand() as u32 & 0x7F,
                    frand() as u32 & 0x7F,
                    255,
                ),
                256,
            );
            //sg.set_color_function(grc);
            ell.init(x as f64, y as f64, radius, radius, 32, false);
            self.rasterizer.add_path(&mut ell, 0);
            agg::render_scanlines_aa(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut r,
                &mut sa,
                &mut sg,
            );
        }

        r.reset_clipping(true); // "true" means "all rendering buffer is visible".
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut r,
            &mut *self.num_cb.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let width = rb.width();
            let height = rb.height();
            self.transform(width as f64, height as f64, x as f64, y as f64);
            return Draw::Yes;
        }

        if flags & InputFlag::MouseRight as u32 != 0 {
            self.skew_x = x as f64;
            self.skew_y = y as f64;
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        self.on_mouse_button_down(rb, x, y, flags);
        Draw::No
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Clipping to multiple rectangle regions");

    if plat.init(512, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

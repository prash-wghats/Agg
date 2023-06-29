use crate::ctrl::slider::Slider;
use crate::platform::*;
use misc::{parse_lion::*, pixel_formats::*};

use agg::alpha_mask_u8::*;
use agg::bounding_rect::*;
use agg::conv_transform::ConvTransform;
use agg::path_storage::*;
use agg::pixfmt_amask_adaptor::PixAmaskAdaptor;
use agg::pixfmt_gray::*;
use agg::rasterizer_scanline_aa::*;
use agg::renderer_scanline::render_all_paths;
use agg::rendering_buffer::RenderBuf;
use agg::{
    Interpolator, RasterScanLine, RenderBuffer, RenderPrim, RendererOutline, RendererScanlineColor,
};

mod ctrl;
mod misc;
mod platform;

use core::f64::consts::PI;
use libc::*;
use std::cell::RefCell;
use std::ptr::null_mut;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;

fn frand() -> i32 {
    unsafe { rand() }
}

const FLIP_Y: bool = true;
const G_PATH_IDX_LENGTH: usize = 100;

type AlphaMaskType = AlphaMaskGray8;

struct Application {
    alpha_buf: *mut u8,
    _alpha_rbuf: agg::RenderBuf,
    alpha_mask: AlphaMaskType,
    ras: RasterizerScanlineAa,
    sl: agg::ScanlineU8,
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
    slider_value: f64,
    num_cb: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
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
                (frand() as u32 & 127) + 128,
                (frand() as u32 & 127) + 128,
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
    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let s0 = Rc::new(RefCell::new(Slider::new(5., 5., 150., 12., !flip_y)));
        s0.borrow_mut().set_range(5.0, 100.);
        s0.borrow_mut().set_value(10.);
        s0.borrow_mut().set_label("N=%1.2f");
        s0.borrow_mut().no_transform();

        let mut app = Self {
            alpha_buf: null_mut(),
            _alpha_rbuf: agg::RenderBuf::new_default(),
            alpha_mask: AlphaMaskType::new(RenderBuf::new_default()),
            ras: RasterizerScanlineAa::new(),
            sl: agg::ScanlineU8::new(),
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
            slider_value: 0.,
            num_cb: s0.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![s0],
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

    fn on_resize(&mut self, cx: i32, cy: i32) {
        self.generate_alpha_mask(cx, cy);
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let width = self.util.borrow().width() as i32;
        let height = self.util.borrow().height() as i32;

        if self.num_cb.borrow().value() != self.slider_value {
            self.generate_alpha_mask(width, height);
            self.slider_value = self.num_cb.borrow().value();
        }
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rbase = agg::RendererBase::new_borrowed(&mut pixf);
        rbase.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut pfa = PixAmaskAdaptor::new(&mut pixf, &mut self.alpha_mask);
        let mut r = agg::RendererBase::new_borrowed(&mut pfa);
        r.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut r);

        let mut mtx = agg::TransAffine::new_default();
        mtx *= agg::TransAffine::trans_affine_translation(-self.base_dx, -self.base_dy);
        mtx *= agg::TransAffine::trans_affine_scaling(self.scale, self.scale);
        mtx *= agg::TransAffine::trans_affine_rotation(self.angle + PI);
        mtx *= agg::TransAffine::trans_affine_skewing(self.skew_x / 1000.0, self.skew_y / 1000.0);
        mtx *= agg::TransAffine::trans_affine_translation(width as f64 / 2., height as f64 / 2.);

        // Render the lion
        let mut trans = ConvTransform::new_borrowed(&mut self.path, mtx);
        render_all_paths(
            &mut self.ras,
            &mut self.sl,
            &mut rs,
            &mut trans,
            &self.colors,
            &self.path_idx,
            self.npaths,
        );

        // Render random Bresenham lines and markers

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
            agg::SpanIpLinear<agg::TransAffine>,
            agg::GradientCircle,
            agg::GradientLinearColor<agg::Rgba8>,
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
            self.ras.add_path(&mut ell, 0);
            agg::render_scanlines_aa(&mut self.ras, &mut self.sl, &mut r, &mut sa, &mut sg);
        }

        let mut rbase = agg::RendererBase::new_borrowed(&mut pixf);
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rbase,
            &mut *self.num_cb.borrow_mut(),
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
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption(r#"AGG Example. Lion with Alpha Masking"#);

    if plat.init(512, 512, WindowFlag::Resize as u32) {
        plat.run();
    }
}

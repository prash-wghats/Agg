use agg::basics::*;
use agg::color_rgba::*;
use agg::conv_stroke::*;
use agg::ellipse::*;
use agg::gsv_text::*;
use agg::math::*;
use agg::pixfmt_rgba::*;
use agg::rasterizer_scanline_aa::*;
use agg::renderer_base::*;
use agg::renderer_scanline::*;
use agg::rendering_buffer::*;
use agg::rounded_rect::RoundedRect;
use agg::scanline_u::ScanlineU8;
use agg::span_allocator::VecSpan;
use agg::span_gradient::{GradientX, SpanGradient};
use agg::span_interpolator_linear::SpanIpLinear;
use agg::trans_affine::TransAffine;

use agg::{AggPrimitive, Color, ColorFn, RasterScanLine, Renderer, RendererScanlineColor, RgbArgs};

use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = true;

macro_rules! from_u32 {
    ($v:expr) => {
        C::ValueType::from_u32($v)
    };
}

type BlenderCustom = CompOpRgbaAdaptor<Rgba8, OrderBgra>;
type PixCustom<'a> = CustomBlendRgba<'a, Rgba8, OrderBgra, BlenderCustom, RenderBuf>;

type PriBlender = BlenderRgba<Rgba8, OrderBgra>;
type PriPixfmt<'a> = AlphaBlendRgba<'a, Rgba8, OrderBgra, PriBlender, RenderBuf>;

type PriBlenderPre = BlenderRgbaPre<Rgba8, OrderBgra>;
type PriPixfmtPre<'a> = AlphaBlendRgba<'a, Rgba8, OrderBgra, PriBlenderPre, RenderBuf>;

pub struct GradLinearColor<C: Color + RgbArgs> {
    pub c1: C,
    pub c2: C,
    c: C,
}

impl<C: Color + RgbArgs> GradLinearColor<C> {
    pub fn new(c1: C, c2: C) -> Self {
        let c = C::new();
        GradLinearColor { c1, c2, c }
    }

    pub fn colors(&mut self, c1: C, c2: C) {
        self.c1 = c1;
        self.c2 = c2;
    }
}

impl<C: Color + RgbArgs> ColorFn<C> for GradLinearColor<C> {
    fn size(&self) -> u32 {
        256
    }

    fn get(&mut self, v: u32) -> C {
        let mut c = C::new();
        let v = (v as u32) << (C::BASE_SHIFT - 8);
        let r = from_u32!(
            (self.c2.r().into_u32().wrapping_sub(self.c1.r().into_u32()))
                .wrapping_mul(v)
                .wrapping_add(self.c1.r().into_u32() << C::BASE_SHIFT)
                >> C::BASE_SHIFT
        );
        let g = from_u32!(
            (self.c2.g().into_u32().wrapping_sub(self.c1.g().into_u32()))
                .wrapping_mul(v)
                .wrapping_add(self.c1.g().into_u32() << C::BASE_SHIFT)
                >> C::BASE_SHIFT
        );
        let b = from_u32!(
            (self.c2.b().into_u32().wrapping_sub(self.c1.b().into_u32()))
                .wrapping_mul(v)
                .wrapping_add(self.c1.b().into_u32() << C::BASE_SHIFT)
                >> C::BASE_SHIFT
        );
        let a = from_u32!(
            (self.c2.a().into_u32().wrapping_sub(self.c1.a().into_u32()))
                .wrapping_mul(v)
                .wrapping_add(self.c1.a().into_u32() << C::BASE_SHIFT)
                >> C::BASE_SHIFT
        );
        *c.r_mut() = r;
        *c.g_mut() = g;
        *c.b_mut() = b;
        *c.a_mut() = a;
        self.c = c;
        self.c
    }
}

pub fn gradient_affine(x1: f64, y1: f64, x2: f64, y2: f64, gradient_d2: f64) -> TransAffine {
    let mut mtx = TransAffine::new_default();
    let dx = x2 - x1;
    let dy = y2 - y1;
    mtx.reset();

    mtx.multiply(&TransAffine::trans_affine_scaling_eq(
        f64::sqrt(dx * dx + dy * dy) / gradient_d2,
    ));
    mtx.multiply(&TransAffine::trans_affine_rotation(f64::atan2(dy, dx)));
    mtx.multiply(&TransAffine::trans_affine_translation(x1, y1));
    mtx.invert();
    mtx
}

fn circle<Rb: Renderer<C = Rgba8>>(
    rbase: &mut Rb, c1: Rgba8, c2: Rgba8, x1: f64, y1: f64, x2: f64, y2: f64, shadow_alpha: f64,
) {
    let mut gradient_func = GradientX {}; // The gradient function
    let gradient_mtx = gradient_affine(x1, y1, x2, y2, 100.);
    let mut span_interpolator = SpanIpLinear::new(gradient_mtx); // Span interpolator
    let mut span_allocator = VecSpan::<Rgba8>::new(); // Span Allocator
    let mut color_func = GradLinearColor::<Rgba8>::new(c1, c2);

    let mut span_gradient =
        SpanGradient::<Rgba8, SpanIpLinear<TransAffine>, GradientX, GradLinearColor<Rgba8>>::new(
            &mut span_interpolator,
            &mut gradient_func,
            &mut color_func,
            0.,
            100.,
        );

    let mut sl = ScanlineU8::new();
    let mut ras: RasterizerScanlineAa = RasterizerScanlineAa::new();

    let r = calc_distance(x1, y1, x2, y2) / 2.;
    let mut ell = Ellipse::new_ellipse((x1 + x2) / 2. + 5., (y1 + y2) / 2. - 3., r, r, 100, false);

    ras.add_path(&mut ell, 0);

    agg::render_scanlines_aa_solid(
        &mut ras,
        &mut sl,
        rbase,
        &agg::Rgba8::new_from_rgba(&Rgba::new_params(0.6, 0.6, 0.6, 0.7 * shadow_alpha)),
    );

    ell.init((x1 + x2) / 2., (y1 + y2) / 2., r, r, 100, false);
    ras.add_path(&mut ell, 0);
    agg::render_scanlines_aa(
        &mut ras,
        &mut sl,
        rbase,
        &mut span_allocator,
        &mut span_gradient,
    );
}

fn src_shape<Rb: Renderer<C = Rgba8>>(
    rbase: &mut Rb, c1: Rgba8, c2: Rgba8, x1: f64, y1: f64, x2: f64, y2: f64,
) {
    let mut gradient_func = GradientX {}; // The gradient function
    let gradient_mtx = gradient_affine(x1, y1, x2, y2, 100.);
    let mut span_interpolator = SpanIpLinear::new(gradient_mtx); // Span interpolator
    let mut span_allocator = VecSpan::<Rgba8>::new(); // Span Allocator
    let mut color_func = GradLinearColor::<Rgba8>::new(c1, c2);

    let mut span_gradient =
        SpanGradient::<Rgba8, SpanIpLinear<TransAffine>, GradientX, GradLinearColor<Rgba8>>::new(
            &mut span_interpolator,
            &mut gradient_func,
            &mut color_func,
            0.,
            100.,
        );

    let mut sl = ScanlineU8::new();
    let mut ras: RasterizerScanlineAa = RasterizerScanlineAa::new();

    let mut shape = RoundedRect::new(x1, y1, x2, y2, 40.);

    ras.add_path(&mut shape, 0);
    agg::render_scanlines_aa(
        &mut ras,
        &mut sl,
        rbase,
        &mut span_allocator,
        &mut span_gradient,
    );
}

struct Application {
    alpha_src: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    alpha_dst: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    comp_op: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    pub ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn render_scene(&self, rbuf: &mut RenderBuf, pixf: &mut PriPixfmt) {
        let mut ren_pixf = PixCustom::new_borrowed(rbuf);
        ren_pixf.set_comp_op(self.comp_op.borrow().cur_item() as u32);

        let mut renderer = agg::RendererBase::<PixCustom>::new_borrowed(&mut ren_pixf);
        let mut rb_base = agg::RendererBase::<PriPixfmt>::new_borrowed(pixf);
        let mut px = PriPixfmt::new_owned(self.util.borrow_mut().rbuf_img_mut(1).clone());

        rb_base.blend_from(
            &mut px,
            None,
            250,
            180,
            (self.alpha_dst.borrow().value() * 255.0) as u32,
        );

        circle(
            &mut rb_base,
            Rgba8::new_params(
                0xFD,
                0xF0,
                0x6F,
                (self.alpha_dst.borrow().value() * 255.0) as u32,
            ),
            Rgba8::new_params(
                0xFE,
                0x9F,
                0x34,
                (self.alpha_dst.borrow().value() * 255.0) as u32,
            ),
            70. * 3.,
            100. + 24. * 3.,
            37. * 3.,
            100. + 79. * 3.,
            self.alpha_dst.borrow().value(),
        );

        if self.comp_op.borrow().cur_item() == 25 {
            // Contrast
            let v = self.alpha_src.borrow().value();

            src_shape(
                &mut renderer,
                Rgba8::new_from_rgba(&Rgba::new_params(v, v, v, 1.0)),
                Rgba8::new_from_rgba(&Rgba::new_params(v, v, v, 1.0)),
                300. + 50.,
                100. + 24. * 3.,
                107. + 50.,
                100. + 79. * 3.,
            );
        } else {
            src_shape(
                &mut renderer,
                Rgba8::new_params(
                    0x7F,
                    0xC1,
                    0xFF,
                    (self.alpha_src.borrow().value() * 255.0) as u32,
                ),
                Rgba8::new_params(
                    0x05,
                    0x00,
                    0x5F,
                    (self.alpha_src.borrow().value() * 255.0) as u32,
                ),
                300. + 50.,
                100. + 24. * 3.,
                107. + 50.,
                100. + 79. * 3.,
            );
        }
    }
}

impl Interface for Application {
    fn new(_format: PixFormat, _flip_y_: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut alpha_src = Slider::new(5., 5., 400., 11., !FLIP_Y);
        let mut alpha_dst = Slider::new(5., 5. + 15., 400., 11. + 15., !FLIP_Y);
        let mut comp_op = Rbox::new(420., 5.0, 420.0 + 170.0, 395.0, !FLIP_Y);

        alpha_src.set_label("Src Alpha=%1.2f");
        alpha_src.set_value(0.75);

        alpha_dst.set_label("Dst Alpha=%1.2f");
        alpha_dst.set_value(1.0);

        comp_op.set_text_size(6.8, 0.);
        comp_op.add_item("clear");
        comp_op.add_item("src");
        comp_op.add_item("dst");
        comp_op.add_item("src-over");
        comp_op.add_item("dst-over");
        comp_op.add_item("src-in");
        comp_op.add_item("dst-in");
        comp_op.add_item("src-out");
        comp_op.add_item("dst-out");
        comp_op.add_item("src-atop");
        comp_op.add_item("dst-atop");
        comp_op.add_item("xor");
        comp_op.add_item("plus");
        comp_op.add_item("minus");
        comp_op.add_item("multiply");
        comp_op.add_item("screen");
        comp_op.add_item("overlay");
        comp_op.add_item("darken");
        comp_op.add_item("lighten");
        comp_op.add_item("color-dodge");
        comp_op.add_item("color-burn");
        comp_op.add_item("hard-light");
        comp_op.add_item("soft-light");
        comp_op.add_item("difference");
        comp_op.add_item("exclusion");
        comp_op.add_item("contrast");
        comp_op.add_item("invert");
        comp_op.add_item("invert-rgb");
        comp_op.set_cur_item(3);

        let s0 = Rc::new(RefCell::new(alpha_src));
        let s1 = Rc::new(RefCell::new(alpha_dst));
        let rb = Rc::new(RefCell::new(comp_op));
        let app = Self {
            ctrls: CtrlContainer {
                ctrl: vec![s0.clone(), s1.clone(), rb.clone()],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            alpha_src: s0,
            alpha_dst: s1,
            comp_op: rb,
            util: util,
        };

        app
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }
    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }
    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        Draw::No
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut pixf = PriPixfmt::new_owned(rb.clone());
        let mut ren_base = agg::RendererBase::<PriPixfmt>::new_borrowed(&mut pixf);

        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let w = ren_base.width();
        let h = ren_base.height();
        for y in (0..h).step_by(8) {
            for x in ((((y >> 3) & 1) << 3)..w).step_by(16) {
                ren_base.copy_bar(
                    x as i32,
                    y as i32,
                    (x + 7) as i32,
                    (y + 7) as i32,
                    &Rgba8::new_params(0xdf, 0xdf, 0xdf, 0xff),
                );
            }
        }

        let (wi, hi) = (self.util.borrow().width(), self.util.borrow().height());
        self.util.borrow_mut().create_img(0, wi as u32, hi as u32); // agg_platform_support functionality

        let mut pixf2 = PriPixfmt::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        {
            let mut ren_base2 = RendererBase::<PriPixfmt>::new_borrowed(&mut pixf2);

            ren_base2.clear(&agg::Rgba8::new_params(0, 0, 0, 0));
        }

        let mut pix = PriPixfmtPre::new_borrowed(rb);
        let mut ren_base_pre = RendererBase::<PriPixfmtPre>::new_borrowed(&mut pix);

        self.util.borrow_mut().start_timer();
        let mut aimg = *self.util.borrow_mut().rbuf_img_mut(0);
        self.render_scene(&mut aimg, &mut pixf2);
        let tm = self.util.borrow_mut().elapsed_time();

        ren_base_pre.blend_from(&mut pixf2, None, 0, 0, CoverScale::FULL as u32);

        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let mut rensl =
            agg::RendererScanlineAASolid::<RendererBase<PriPixfmt>>::new_borrowed(&mut ren_base);

        let mut t = GsvText::new();
        t.set_size(10.0, 0.);

        let mut pt: ConvStroke<_> = ConvStroke::new_owned(t);
        pt.set_width(1.5);

        let s = format!("{:.2} ms", tm);
        pt.source_mut().set_start_point(10.0, 35.0);
        pt.source_mut().set_text(&s);

        ras.add_path(&mut pt, 0);
        rensl.set_color(Rgba8::new_params(0, 0, 0, 255));

        render_scanlines(&mut ras, &mut sl, &mut rensl);

        ctrl::render_ctrl_rs(
            &mut ras,
            &mut sl,
            &mut rensl,
            &mut *self.alpha_src.borrow_mut(),
        );
        ctrl::render_ctrl_rs(
            &mut ras,
            &mut sl,
            &mut rensl,
            &mut *self.alpha_dst.borrow_mut(),
        );
        ctrl::render_ctrl_rs(
            &mut ras,
            &mut sl,
            &mut rensl,
            &mut *self.comp_op.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "compositing";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);

    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(1, img_name) {
        if img_name.eq("compositing") {
            let ext = (plat.app_mut().util.borrow().img_ext()).to_string();
            buf = format!(
                "File not found: {}{}. Download http://www.antigrain.com/{}{}
				or copy it from another directory if available.",
                img_name, ext, img_name, ext
            );
        } else {
            buf = format!("File not found: {}", img_name);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
    }

    plat.set_caption("AGG Example. Compositing Modes");

    if plat.init(600, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

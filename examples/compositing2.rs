use agg::color_rgba::*;
use agg::pixfmt_rgba::*;
use agg::rendering_buffer::*;
use agg::span_gradient::SpanGradient;
use agg::span_interpolator_linear::SpanIpLinear;
use agg::trans_affine::TransAffine;

use agg::{Color, RasterScanLine, Renderer};

use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = true;

type BlenderCustom = CompOpRgbaAdaptor<Rgba8, OrderBgra>;
type PixCustom<'a> = CustomBlendRgba<'a, Rgba8, OrderBgra, BlenderCustom, RenderBuf>;
type PriBlender = BlenderRgba<Rgba8, OrderBgra>;
type PriPixfmt<'a> = AlphaBlendRgba<'a, Rgba8, OrderBgra, PriBlender, RenderBuf>;

fn generate_color_ramp(
    c: &mut Vec<Rgba8>, c1: agg::Rgba8, c2: agg::Rgba8, c3: agg::Rgba8, c4: agg::Rgba8,
) {
    c.clear();
    for i in 0..85 {
        c.push(c1.gradient(&c2, i as f64 / 85.0));
    }
    for i in 85..170 {
        c.push(c2.gradient(&c3, (i - 85) as f64 / 85.0));
    }
    for i in 170..256 {
        c.push(c3.gradient(&c4, (i - 170) as f64 / 85.0));
    }
}

struct Application {
    alpha_src: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    alpha_dst: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    comp_op: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    ramp1: Vec<Rgba8>,
    ramp2: Vec<Rgba8>,
    ras: agg::RasterizerScanlineAa,
    sl: agg::ScanlineU8,
    pub ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, _flip_y_: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut alpha_src = Slider::new(5., 5., 400., 11., !FLIP_Y);
        let mut alpha_dst = Slider::new(5., 5. + 15., 400., 11. + 15., !FLIP_Y);
        let mut comp_op = Rbox::new(420., 5.0, 420.0 + 170.0, 395.0, !FLIP_Y);

        alpha_src.set_label("Src Alpha=%1.2f");
        alpha_src.set_value(1.0);

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
            ramp1: vec![],
            ramp2: vec![],
            ras: agg::RasterizerScanlineAa::new(),
            sl: agg::ScanlineU8::new(),
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

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = PriPixfmt::new_owned(rbuf.clone());
        let mut rb = agg::RendererBase::<PriPixfmt>::new_borrowed(&mut pixf);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        generate_color_ramp(
            &mut self.ramp1,
            agg::Rgba8::new_params(0, 0, 0, (self.alpha_dst.borrow().value() * 255.) as u32),
            agg::Rgba8::new_params(0, 0, 255, (self.alpha_dst.borrow().value() * 255.) as u32),
            agg::Rgba8::new_params(0, 255, 0, (self.alpha_dst.borrow().value() * 255.) as u32),
            agg::Rgba8::new_params(255, 0, 0, 0),
        );

        generate_color_ramp(
            &mut self.ramp2,
            agg::Rgba8::new_params(0, 0, 0, (self.alpha_src.borrow().value() * 255.) as u32),
            agg::Rgba8::new_params(0, 0, 255, (self.alpha_src.borrow().value() * 255.) as u32),
            agg::Rgba8::new_params(0, 255, 0, (self.alpha_src.borrow().value() * 255.) as u32),
            agg::Rgba8::new_params(255, 0, 0, 0),
        );

        self.render_scene(rbuf);
        let mut rensl = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        ctrl::render_ctrl_rs(
            &mut self.ras,
            &mut self.sl,
            &mut rensl,
            &mut *self.alpha_src.borrow_mut(),
        );
        ctrl::render_ctrl_rs(
            &mut self.ras,
            &mut self.sl,
            &mut rensl,
            &mut *self.alpha_dst.borrow_mut(),
        );
        ctrl::render_ctrl_rs(
            &mut self.ras,
            &mut self.sl,
            &mut rensl,
            &mut *self.comp_op.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }
}

impl Application {
    fn radial_shape<RenBase: Renderer<C = Rgba8>>(
        &mut self, rbase: &mut RenBase, i: u32, x1: f64, y1: f64, x2: f64, y2: f64,
    ) {
        let mut gradient_func = agg::GradientRadial {}; // The gradient function
        let mut gradient_mtx = agg::TransAffine::new_default();

        let cx = (x1 + x2) / 2.0;
        let cy = (y1 + y2) / 2.0;
        let r = 0.5
            * (if (x2 - x1) < (y2 - y1) {
                x2 - x1
            } else {
                y2 - y1
            });

        gradient_mtx *= agg::TransAffine::trans_affine_scaling_eq(r / 100.0);
        gradient_mtx *= agg::TransAffine::trans_affine_translation(cx, cy);
        gradient_mtx *= self.util.borrow().trans_affine_resizing().clone();
        gradient_mtx.invert();

        let mut span_interpolator = agg::SpanIpLinear::new(gradient_mtx); // Span interpolator
        let mut span_allocator = agg::VecSpan::<Rgba8>::new(); // Span Allocator

        let mut span_gradient = SpanGradient::<
            '_,
            Rgba8,
            SpanIpLinear<TransAffine>,
            agg::GradientRadial,
            Vec<Rgba8>,
        >::new(
            &mut span_interpolator,
            &mut gradient_func,
            if i == 0 {
                &mut self.ramp1
            } else {
                &mut self.ramp2
            },
            0.,
            100.,
        );

        let ell = agg::Ellipse::new_ellipse(cx, cy, r, r, 100, false);
        let mut trans =
            agg::ConvTransform::new_owned(ell, self.util.borrow().trans_affine_resizing().clone());
        self.ras.add_path(&mut trans, 0);

        agg::render_scanlines_aa(
            &mut self.ras,
            &mut self.sl,
            rbase,
            &mut span_allocator,
            &mut span_gradient,
        );
    }

    fn render_scene(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = PixCustom::new_borrowed(rbuf);

        let mut ren = agg::RendererBase::<PixCustom>::new_borrowed(&mut pixf);

        ren.ren_mut().set_comp_op(CompOp::CompOpDifference as u32);
        self.radial_shape(&mut ren, 0, 50., 50., 50. + 320., 50. + 320.);

        ren.ren_mut()
            .set_comp_op(self.comp_op.borrow().cur_item() as u32);
        let cx = 50.;
        let cy = 50.;
        self.radial_shape(
            &mut ren,
            1,
            cx + 120. - 70.,
            cy + 120. - 70.,
            cx + 120. + 70.,
            cy + 120. + 70.,
        );
        self.radial_shape(
            &mut ren,
            1,
            cx + 200. - 70.,
            cy + 120. - 70.,
            cx + 200. + 70.,
            cy + 120. + 70.,
        );
        self.radial_shape(
            &mut ren,
            1,
            cx + 120. - 70.,
            cy + 200. - 70.,
            cx + 120. + 70.,
            cy + 200. + 70.,
        );
        self.radial_shape(
            &mut ren,
            1,
            cx + 200. - 70.,
            cy + 200. - 70.,
            cx + 200. + 70.,
            cy + 200. + 70.,
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);
    plat.set_caption("AGG Example. Compositing Modes");

    if plat.init(
        600,
        400,
        WindowFlag::Resize as u32 | WindowFlag::KeepAspectRatio as u32,
    ) {
        plat.run();
    }
}

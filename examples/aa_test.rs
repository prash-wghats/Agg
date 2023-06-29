use agg::basics::{PathCmd, PathFlag};
use agg::color_rgba::{Rgba, Rgba8};
use agg::conv_dash::ConvDash;
use agg::conv_stroke::ConvStroke;
use agg::gamma_lut::GammaLut;
use agg::math_stroke::LineCap;
use agg::rendering_buffer::*;
use agg::span_allocator::VecSpan;
use agg::span_gradient::{GradientX, SpanGradient};
use agg::span_interpolator_linear::SpanIpLinear;
use agg::trans_affine::TransAffine;

use libc::*;
use std::f64::consts::*;
use std::ops::{Index, IndexMut};

use crate::ctrl::slider::Slider;
use crate::platform::*;
use agg::{Color, ColorFn, RasterScanLine, RendererScanline, RendererScanlineColor, VertexSource};
mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = false;

struct SimpleVertexSource {
    num_vertices: u32,
    count: u32,
    x: [f64; 8],
    y: [f64; 8],
    cmd: [u32; 8],
}

impl SimpleVertexSource {
    fn new() -> SimpleVertexSource {
        SimpleVertexSource {
            num_vertices: 0,
            count: 0,
            x: [0.0; 8],
            y: [0.0; 8],
            cmd: [PathCmd::Stop as u32; 8],
        }
    }
    fn init(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.num_vertices = 2;
        self.count = 0;
        self.x[0] = x1;
        self.y[0] = y1;
        self.x[1] = x2;
        self.y[1] = y2;
        self.cmd[0] = PathCmd::MoveTo as u32;
        self.cmd[1] = PathCmd::LineTo as u32;
        self.cmd[2] = PathCmd::Stop as u32;
    }
    fn init2(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
        self.num_vertices = 3;
        self.count = 0;
        self.x[0] = x1;
        self.y[0] = y1;
        self.x[1] = x2;
        self.y[1] = y2;
        self.x[2] = x3;
        self.y[2] = y3;
        self.x[3] = 0.;
        self.y[3] = 0.;
        self.x[4] = 0.;
        self.y[4] = 0.;
        self.cmd[0] = PathCmd::MoveTo as u32;
        self.cmd[1] = PathCmd::LineTo as u32;
        self.cmd[2] = PathCmd::LineTo as u32;
        self.cmd[3] = PathCmd::EndPoly as u32 | PathFlag::Close as u32;
        self.cmd[4] = PathCmd::Stop as u32;
    }
}

impl VertexSource for SimpleVertexSource {
    fn rewind(&mut self, _: u32) {
        self.count = 0;
    }
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        *x = self.x[self.count as usize];
        *y = self.y[self.count as usize];
        self.count += 1;
        self.cmd[(self.count - 1) as usize] as u32
    }
}

struct DashedLine<'a> {
    ras: agg::RasterizerScanlineAa,
    sl: agg::ScanlineU8,
    //src: SimpleVertexSource,
    //dash: ConvDash<SimpleVertexSource>,
    stroke: ConvStroke<'a, SimpleVertexSource>,
    dash_stroke: ConvStroke<'a, ConvDash<'a, SimpleVertexSource>>,
}

impl<'a> DashedLine<'a> {
    fn new() -> Self {
        Self {
            ras: agg::RasterizerScanlineAa::new(),
            sl: agg::ScanlineU8::new(),
            stroke: ConvStroke::new_owned(SimpleVertexSource::new()),
            dash_stroke: ConvStroke::new_owned(ConvDash::new_owned(SimpleVertexSource::new())),
        }
    }

    fn draw<Rensl: RendererScanline>(
        &mut self, rensl: &mut Rensl, x1: f64, y1: f64, x2: f64, y2: f64, line_width: f64,
        dash_length: f64,
    ) {
        self.stroke
            .source_mut()
            .init(x1 + 0.5, y1 + 0.5, x2 + 0.5, y2 + 0.5);
        self.dash_stroke
            .source_mut()
            .source_mut()
            .init(x1 + 0.5, y1 + 0.5, x2 + 0.5, y2 + 0.5);

        self.ras.reset();
        if dash_length > 0.0 {
            self.dash_stroke.source_mut().remove_all_dashes();
            self.dash_stroke
                .source_mut()
                .add_dash(dash_length, dash_length);
            self.dash_stroke.set_width(line_width);
            self.dash_stroke.set_line_cap(LineCap::Round);
            self.ras.add_path(&mut self.dash_stroke, 0);
        } else {
            self.stroke.set_width(line_width);
            self.stroke.set_line_cap(LineCap::Round);
            self.ras.add_path(&mut self.stroke, 0);
        }
        agg::render_scanlines(&mut self.ras, &mut self.sl, rensl);
    }
}

fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

fn frand(x: f64) -> f64 {
    //((((frand() << 15) | frand()) & 0x3FFFFFFF) as f64 / 1000000.0) * x
    unsafe {
        return ((((rand() << 15) | rand()) & 0x3FFFFFFF) % 1000000) as f64 * x / 1000000.0;
    }
}

// Calculate the affine transformation matrix for the linear gradient
// from (x1, y1) to (x2, y2). gradient_d2 is the "base" to scale the
// gradient. Here d1 must be 0.0, and d2 must equal gradient_d2.
//---------------------------------------------------------------
fn calc_linear_gradient_transform(
    x1: f64, y1: f64, x2: f64, y2: f64, mtx: &mut TransAffine, gradient_d2: f64,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    mtx.reset();
    mtx.multiply(&TransAffine::trans_affine_scaling_eq(
        f64::sqrt(dx * dx + dy * dy) / gradient_d2,
    ));
    mtx.multiply(&TransAffine::trans_affine_rotation(f64::atan2(dy, dx)));
    mtx.multiply(&TransAffine::trans_affine_translation(x1 + 0.5, y1 + 0.5));
    mtx.invert();
}

struct ColorFunc<C: Color>([C; 256]);
impl<C: Color> ColorFunc<C> {
    pub fn new() -> Self {
        Self([C::new(); 256])
    }
}
impl<C: Color> ColorFn<C> for ColorFunc<C> {
    fn size(&self) -> u32 {
        self.0.len() as u32
    }
    fn get(&mut self, i: u32) -> C {
        self.0[i as usize]
    }
}
impl<C: Color> Index<usize> for ColorFunc<C> {
    type Output = C;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}
impl<C: Color> IndexMut<usize> for ColorFunc<C> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

fn fill_color_array(array: &mut ColorFunc<Rgba8>, begin: agg::Rgba8, end: agg::Rgba8) {
    for i in 0..256 {
        array[i] = begin.gradient(&end, i as f64 / 255.0);
    }
}

struct Application {
    m_gamma: GammaLut<u8, u8, 8, 8>,
    m_slider_gamma: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    pub m_ctrls: CtrlContainer,
    m_util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y_: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let sl = Rc::new(RefCell::new(Slider::new(3., 3., 480. - 3., 8., !flip_y_)));
        let app = Self {
            m_ctrls: CtrlContainer {
                ctrl: vec![sl.clone()],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            m_gamma: GammaLut::new_with_gamma(1.0),
            m_slider_gamma: sl,
            m_util: util,
        };
        app.m_slider_gamma.borrow_mut().set_range(0.1, 3.0);
        app.m_slider_gamma.borrow_mut().set_value(1.6);
        app.m_slider_gamma.borrow_mut().set_label("Gamma=%4.3f");
        app
    }
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.m_ctrls
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
        let mut pix = agg::PixBgr24Gamma::new_borrowed(rb);
        pix.blender_mut().set_gamma_borrowed(&mut self.m_gamma);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24Gamma>::new_borrowed(&mut pix);
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        ren_base.clear(&agg::Rgba8::new_params(0, 0, 0, 255));

        // gamma correction
        ////ras.gamma(agg::gamma_power());
        ren_base
            .ren_mut()
            .blender_mut()
            .gamma_mut()
            .set_gamma(self.m_slider_gamma.borrow().value());

        {
            let mut ren_sl =
                agg::RendererScanlineAASolid::<agg::RendererBase<agg::PixBgr24Gamma>>::new_borrowed(
                    &mut ren_base,
                );

            // radial line test
            //-------------------------
            let mut dash = DashedLine::new();

            let cx = self.m_util.borrow().width() / 2.0;
            let cy = self.m_util.borrow().height() as f64 / 2.0;

            ren_sl.set_color(Rgba8::new_params(255, 255, 255, 51));
            for i in (1..=180).rev() {
                let n = 2.0 * PI * i as f64 / 180.0;
                dash.draw(
                    &mut ren_sl,
                    cx + min(cx, cy) * n.sin(),
                    cy + min(cx, cy) * n.cos(),
                    cx,
                    cy,
                    1.0,
                    if i < 90 { i as f64 } else { 0.0 },
                );
            }
            for j in 1..=20 {
                let i = j as f64;
                ren_sl.set_color(Rgba8::new_params(255, 255, 255, 255));

                // integral point sizes 1..20
                //----------------
                let mut ell = agg::Ellipse::new();

                ell.init(
                    20. + i * (i + 1.) + 0.5,
                    20.5,
                    i / 2.0,
                    i / 2.0,
                    8 + j,
        false,
                );
                ras.reset();
                ras.add_path(&mut ell, 0);

                agg::render_scanlines(&mut ras, &mut sl, &mut ren_sl);

                // fractional point sizes 0..2
                //----------------
                ell.init(18. + i * 4. + 0.5, 33. + 0.5, i / 20.0, i / 20.0, 8, false);
                ras.reset();
                ras.add_path(&mut ell, 0);

                agg::render_scanlines(&mut ras, &mut sl, &mut ren_sl);

                // fractional point positioning
                //---------------
                ell.init(
                    18. + i * 4. + (i - 1.) / 10.0 + 0.5,
                    27. + (i - 1.) / 10.0 + 0.5,
                    0.5,
                    0.5,
                    8,
        false,
                );
                ras.reset();
                ras.add_path(&mut ell, 0);

                agg::render_scanlines(&mut ras, &mut sl, &mut ren_sl);
                if j <= 10 {
                    // integral line width, horz aligned (mipmap test)
                    //-------------------
                    dash.draw(
                        &mut ren_sl,
                        125.5,
                        119.5 + (i + 2.) * (i / 2.0),
                        135.5,
                        119.5 + (i + 2.) * (i / 2.0),
                        i,
                        0.0,
                    );
                }

                // fractional line width 0..2, 1 px H
                //-----------------
                dash.draw(
                    &mut ren_sl,
                    17.5 + i * 4.0,
                    192.0,
                    18.5 + i * 4.0,
                    192.0,
                    i / 10.0,
                    0.,
                );

                // fractional line positioning, 1 px H
                //-----------------
                dash.draw(
                    &mut ren_sl,
                    17.5 + i * 4.0 + (i - 1.) / 10.0,
                    186.0,
                    18.5 + i * 4.0 + (i - 1.) / 10.0,
                    186.0,
                    1.0,
                    0.,
                );
            }
        }

        let mut gradient_func = GradientX {};
        let mut gradient_mtx = TransAffine::new_default();
        let mut span_interpolator = SpanIpLinear::new(gradient_mtx);
        let span_allocator = VecSpan::<Rgba8>::new();
        let mut gradient_colors = ColorFunc::new(); //agg::array::pod_auto_array::<Rgba8>::new();
        let mut span_gradient =
            SpanGradient::<Rgba8, SpanIpLinear<TransAffine>, GradientX, ColorFunc<Rgba8>>::new(
                &mut span_interpolator,
                &mut gradient_func,
                &mut gradient_colors,
                0.,
                100.,
            );
        let mut ren_gradient =
            agg::RendererScanlineAA::<agg::RendererBase<agg::PixBgr24Gamma>, _, _>::new_borrowed(
                &mut ren_base,
                span_allocator,
                &mut span_gradient,
            );
        let mut dash_gradient = DashedLine::new();

        let mut x1: f64;
        let mut y1: f64;
        let mut x2: f64;
        let mut y2: f64;

        for j in 1..=20 {
            let i = j as f64;

            // integral line widths 1..20
            //----------------
            fill_color_array(
                ren_gradient.span_gen_mut().color_function_mut(),
                Rgba8::new_params(255, 255, 255, 255),
                Rgba8::new_from_rgba(&Rgba::new_params(
                    (j % 2) as f64,
                    (j % 3) as f64 * 0.5,
                    (j % 5) as f64 * 0.25,
                    1.,
                )),
            );

            x1 = 20.0 + i * (i + 1.0);
            y1 = 40.5;
            x2 = 20.0 + i * (i + 1.0) + (i - 1.) * 4.0;
            y2 = 100.5;
            calc_linear_gradient_transform(x1, y1, x2, y2, &mut gradient_mtx, 100.0);
            dash_gradient.draw(&mut ren_gradient, x1, y1, x2, y2, i as f64, 0.0);

            fill_color_array(
                ren_gradient.span_gen_mut().color_function_mut(),
                Rgba8::new_params(255, 0, 0, 255),
                Rgba8::new_params(0, 0, 255, 255),
            );

            // fractional line lengths H (red/blue)
            //----------------
            x1 = 17.5 + i * 4.0;
            y1 = 107.0;
            x2 = 17.5 + i * 4.0 + i / 6.66666667;
            y2 = 107.0;
            calc_linear_gradient_transform(x1, y1, x2, y2, &mut gradient_mtx, 100.0);
            dash_gradient.draw(&mut ren_gradient, x1, y1, x2, y2, 1.0, 0.0);

            // fractional line lengths V (red/blue)
            //---------------
            x1 = 18.0 + i * 4.0;
            y1 = 112.5;
            x2 = 18.0 + i * 4.0;
            y2 = 112.5 + i / 6.66666667;
            calc_linear_gradient_transform(x1, y1, x2, y2, &mut gradient_mtx, 100.0);
            dash_gradient.draw(&mut ren_gradient, x1, y1, x2, y2, 1.0, 0.);

            // fractional line positioning (red)
            //---------------
            fill_color_array(
                ren_gradient.span_gen_mut().color_function_mut(),
                Rgba8::new_params(255, 0, 0, 255),
                Rgba8::new_params(255, 255, 255, 255),
            );
            x1 = 21.5;
            y1 = 120.0 + (i - 1.) * 3.1;
            x2 = 52.5;
            y2 = 120.0 + (i - 1.) * 3.1;
            calc_linear_gradient_transform(x1, y1, x2, y2, &mut gradient_mtx, 100.);
            dash_gradient.draw(&mut ren_gradient, x1, y1, x2, y2, 1.0, 0.);

            // fractional line width 2..0 (green)
            fill_color_array(
                ren_gradient.span_gen_mut().color_function_mut(),
                Rgba8::new_params(0, 255, 0, 255),
                Rgba8::new_params(255, 255, 255, 255),
            );
            x1 = 52.5;
            y1 = 118.0 + i as f64 * 3.0;
            x2 = 83.5;
            y2 = 118.0 + i as f64 * 3.0;
            calc_linear_gradient_transform(x1, y1, x2, y2, &mut gradient_mtx, 100.);
            dash_gradient.draw(&mut ren_gradient, x1, y1, x2, y2, 2.0 - (i - 1.) / 10.0, 0.);

            // stippled fractional width 2..0 (blue)
            fill_color_array(
                ren_gradient.span_gen_mut().color_function_mut(),
                Rgba8::new_params(0, 0, 255, 255),
                Rgba8::new_params(255, 255, 255, 255),
            );
            x1 = 83.5;
            y1 = 119.0 + i as f64 * 3.0;
            x2 = 114.5;
            y2 = 119.0 + i as f64 * 3.0;
            calc_linear_gradient_transform(x1, y1, x2, y2, &mut gradient_mtx, 100.);
            dash_gradient.draw(
                &mut ren_gradient,
                x1,
                y1,
                x2,
                y2,
                2.0 - (i - 1.) / 10.0,
                3.0,
            );
        }

        // Triangles
        //---------------
        for i in 1..14 {
            fill_color_array(
                ren_gradient.span_gen_mut().color_function_mut(),
                Rgba8::new_params(255, 255, 255, 255),
                Rgba8::new_from_rgba(&Rgba::new_params(
                    (i % 2) as f64,
                    (i % 3) as f64 * 0.4,
                    (i % 5) as f64 * 0.25,
                    1.0,
                )),
            );
            calc_linear_gradient_transform(
                self.m_util.borrow().width() - 150.0,
                self.m_util.borrow().height() - 20.0 - i as f64 * (i as f64 + 1.5),
                self.m_util.borrow().width() - 20.0,
                self.m_util.borrow().height() - 20.0 - i as f64 * (i as f64 + 1.0),
                &mut gradient_mtx,
                100.,
            );
            ras.reset();
            ras.move_to_d(
                self.m_util.borrow().width() - 150.0,
                self.m_util.borrow().height() - 20.0 - i as f64 * (i as f64 + 1.5),
            );
            ras.line_to_d(
                self.m_util.borrow().width() - 20.0,
                self.m_util.borrow().height() - 20.0 - i as f64 * (i as f64 + 1.0),
            );
            ras.line_to_d(
                self.m_util.borrow().width() - 20.0,
                self.m_util.borrow().height() - 20.0 - i as f64 * (i as f64 + 2.0),
            );
            agg::render_scanlines(&mut ras, &mut sl, &mut ren_gradient);
        }

        // Reset AA Gamma and render the controls
        ras.set_gamma(&agg::GammaPower::new_with_gamma(1.0));
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.m_slider_gamma.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, rb: &mut RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        //let mut rnd = rand::thread_rng();
        unsafe {
            srand(123);
        }

        let mut pix = agg::PixBgr24Gamma::new_borrowed(rb);
        pix.blender_mut().set_gamma_borrowed(&mut self.m_gamma);

        let mut ren_base = agg::RendererBase::<agg::PixBgr24Gamma>::new_borrowed(&mut pix);
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut ren_sl =
            agg::RendererScanlineAASolid::<agg::RendererBase<agg::PixBgr24Gamma>>::new_borrowed(
                &mut ren_base,
            );
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let mut ell = agg::Ellipse::new();

        let w = self.m_util.borrow().width();
        let h = self.m_util.borrow().height();

        self.m_util.borrow_mut().start_timer();

        for _ in 0..20000 {
            let r = frand(20.0) + 1.0;
            ell.init(frand(w), frand(h), r / 2.0, r / 2.0, (r as u32) + 10, false);
            ras.reset();
            ras.add_path(&mut ell, 0);

            agg::render_scanlines(&mut ras, &mut sl, &mut ren_sl);
            ren_sl.set_color(Rgba8::new_from_rgba(&Rgba::new_params(
                frand(1.0),
                frand(1.0),
                frand(1.0),
                0.5 + frand(0.5),
            )));
        }
        let t1 = self.m_util.borrow_mut().elapsed_time();

        let mut gradient_func = GradientX {};
        let mut gradient_mtx = TransAffine::new_default();
        let mut span_interpolator = SpanIpLinear::new(gradient_mtx);
        let span_allocator = VecSpan::<Rgba8>::new();
        let span_allocator0 = VecSpan::<Rgba8>::new();
        let mut gradient_colors = ColorFunc::new(); //agg::array::pod_auto_array::<Rgba8>::new();
        let mut span_gradient =
            SpanGradient::<Rgba8, SpanIpLinear<TransAffine>, GradientX, ColorFunc<Rgba8>>::new(
                &mut span_interpolator,
                &mut gradient_func,
                &mut gradient_colors,
                0.,
                100.,
            );
        let mut ren_gradient =
            agg::RendererScanlineAA::<agg::RendererBase<agg::PixBgr24Gamma>, _, _>::new_borrowed(
                &mut ren_base,
                span_allocator,
                &mut span_gradient,
            );
        let mut dash_gradient = DashedLine::new();

        self.m_util.borrow_mut().start_timer();
        for _ in 0..2000 {
            let x1 = frand(w);
            let y1 = frand(h);
            let x2 = x1 + frand(w * 0.5) - w * 0.25;
            let y2 = y1 + frand(h * 0.5) - h * 0.25;

            fill_color_array(
                ren_gradient.span_gen_mut().color_function_mut(),
                Rgba8::new_from_rgba(&Rgba::new_params(
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                    0.5 + frand(0.5),
                )),
                Rgba8::new_from_rgba(&Rgba::new_params(
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                )),
            );
            calc_linear_gradient_transform(x1, y1, x2, y2, &mut gradient_mtx, 100.0);
            dash_gradient.draw(&mut ren_gradient, x1, y1, x2, y2, 10.0, 0.);
        }
        let t2 = self.m_util.borrow_mut().elapsed_time();

        let mut span_gouraud = agg::SpanGouraudRgba::new_default();
        let mut ren_gouraud =
            agg::RendererScanlineAA::<agg::RendererBase<agg::PixBgr24Gamma>, _, _>::new_borrowed(
                &mut ren_base,
                span_allocator0,
                &mut span_gouraud,
            );

        self.m_util.borrow_mut().start_timer();
        for _ in 0..2000 {
            let x1 = frand(w);
            let y1 = frand(h);
            let x2 = x1 + frand(w * 0.4) - w * 0.2;
            let y2 = y1 + frand(h * 0.4) - h * 0.2;
            let x3 = x1 + frand(w * 0.4) - w * 0.2;
            let y3 = y1 + frand(h * 0.4) - h * 0.2;

            ren_gouraud.span_gen_mut().set_colors(
                Rgba8::new_from_rgba(&Rgba::new_params(
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                    0.5 + frand(0.5),
                )),
                Rgba8::new_from_rgba(&Rgba::new_params(
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                )),
                Rgba8::new_from_rgba(&Rgba::new_params(
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                    frand(1.0),
                )),
            );
            ren_gouraud
                .span_gen_mut()
                .set_triangle(x1, y1, x2, y2, x3, y3, 0.0);
            ras.add_path(ren_gouraud.span_gen_mut(), 0);
            agg::render_scanlines(&mut ras, &mut sl, &mut ren_gouraud);
        }
        let t3 = self.m_util.borrow_mut().elapsed_time();

        let msg = format!(
            "Points={:.2}K/sec, Lines={:.2}K/sec, Triangles={:.2}K/sec",
            20000.0 / t1,
            2000.0 / t2,
            2000.0 / t3
        );
        self.m_util.borrow_mut().message(&msg);
        Draw::Update
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Anti-Aliasing Test");

    if plat.init(480, 350, WindowFlag::Resize as u32) {
        plat.run();
    }
}

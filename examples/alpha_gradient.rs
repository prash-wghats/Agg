use crate::platform::*;
use agg::AggPrimitive;

use agg::basics::{PathCmd, PathFlag};
use agg::rendering_buffer::RenderBuf;
use agg::{AlphaFn, Args, Color, ColorFn, Generator, RasterScanLine};

mod ctrl;
mod platform;
use crate::ctrl::spline::Spline;

use core::f64::consts::PI;
use libc::*;
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

fn frand() -> f64 {
    unsafe { rand() as f64 }
}

const FLIP_Y: bool = true;
const RAND_MAX: f64 = 0x7fff as f64;

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

struct AlphaFunc<V: AggPrimitive>([V; 256]);
impl<V: AggPrimitive> AlphaFunc<V> {
    pub fn new() -> Self {
        Self([V::from_u32(0); 256])
    }
}

impl<V: AggPrimitive> AlphaFn<V> for AlphaFunc<V> {
    fn size(&self) -> u32 {
        self.0.len() as u32
    }
    fn get(&mut self, i: u32) -> V {
        self.0[i as usize]
    }
}
impl<V: AggPrimitive> Index<usize> for AlphaFunc<V> {
    type Output = V;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}
impl<V: AggPrimitive> IndexMut<usize> for AlphaFunc<V> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

struct Application {
    x: [f64; 3],
    y: [f64; 3],
    dx: f64,
    dy: f64,
    idx: i32,
    alpha: Ptr<Spline<'static, ColorType>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn fill_color_array(
        array: &mut ColorFunc<ColorType>, begin: ColorType, middle: ColorType, end: ColorType,
    ) {
        for i in 0..128 {
            array[i] = begin.gradient(&middle, i as f64 / 128.0);
        }
        for i in 128..256 {
            array[i] = middle.gradient(&end, (i - 128) as f64 / 128.0);
        }
    }
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let mut alpha = Spline::new(2., 2., 200., 30., 6, !flip_y);
        alpha.set_point(0, 0.0, 0.0);
        alpha.set_point(1, 1.0 / 5.0, 1.0 - 4.0 / 5.0);
        alpha.set_point(2, 2.0 / 5.0, 1.0 - 3.0 / 5.0);
        alpha.set_point(3, 3.0 / 5.0, 1.0 - 2.0 / 5.0);
        alpha.set_point(4, 4.0 / 5.0, 1.0 - 1.0 / 5.0);
        alpha.set_point(5, 1.0, 1.0);
        alpha.update_spline();
        let alpha = ctrl_ptr(alpha);
        Application {
            x: [257.0, 369.0, 143.0],
            y: [60.0, 170.0, 310.0],
            dx: 0.0,
            dy: 0.0,
            idx: -1,
            alpha: alpha.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![alpha],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        let mut i = 0;
        let (x, y): (f64, f64) = (x as f64, y as f64);
        if flags & InputFlag::MouseLeft as u32 != 0 {
            loop {
                if i == 3 {
                    break;
                }
                if (x - self.x[i]).powi(2) + (y - self.y[i]).powi(2).sqrt() < 10.0 {
                    self.dx = x - self.x[i];
                    self.dy = y - self.y[i];
                    self.idx = i as i32;
                    break;
                }
                i += 1;
            }
            if i == 3 {
                if agg::math::point_in_triangle(
                    self.x[0], self.y[0], self.x[1], self.y[1], self.x[2], self.y[2], x, y,
                ) {
                    self.dx = x - self.x[0];
                    self.dy = y - self.y[0];
                    self.idx = 3;
                }
            }
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        let (x, y): (f64, f64) = (x as f64, y as f64);

        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.idx == 3 {
                let dx = x - self.dx;
                let dy = y - self.dy;
                self.x[1] -= self.x[0] - dx;
                self.y[1] -= self.y[0] - dy;
                self.x[2] -= self.x[0] - dx;
                self.y[2] -= self.y[0] - dy;
                self.x[0] = dx;
                self.y[0] = dy;
                //self.return true
                return Draw::Yes;
            }

            if self.idx >= 0 {
                self.x[self.idx as usize] = x - self.dx;
                self.y[self.idx as usize] = y - self.dy;
                //self.return true
                return Draw::Yes;
            }
        } else {
            self.on_mouse_button_up(rb, x as i32, y as i32, flags);
        }
        Draw::No
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.idx = -1;

        Draw::No
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        let mut dx = 0.0;
        let mut dy = 0.0;
        match key {
            x if x == KeyCode::Left as u32 => {
                dx = -0.1;
            }
            x if x == KeyCode::Right as u32 => {
                dx = 0.1;
            }
            x if x == KeyCode::Up as u32 => {
                dy = 0.1;
            }
            x if x == KeyCode::Down as u32 => {
                dy = -0.1;
            }
            _ => {}
        }
        self.x[0] += dx;
        self.y[0] += dy;
        self.x[1] += dx;
        self.y[1] += dy;

        Draw::Yes
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pf = Pixfmt::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pf);
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        // Draw some background
        let mut ell = agg::Ellipse::new();
        unsafe { srand(1234) };
        let w = self.util.borrow().width();
        let h = self.util.borrow().height();
        for _i in 0..100 {
            ell.init(
                frand() % w,
                frand() % h,
                frand() % 60. + 5.,
                frand() % 60. + 5.,
                50,
    false,
            );
            ras.add_path(&mut ell, 0);
            agg::render_scanlines_aa_solid(
                &mut ras,
                &mut sl,
                &mut ren_base,
                &ColorType::new_from_rgba(&agg::Rgba::new_params(
                    frand() / RAND_MAX,
                    frand() / RAND_MAX,
                    frand() / RAND_MAX,
                    frand() / RAND_MAX / 2.0,
                )),
            );
        }

        let parallelogram = [
            self.x[0], self.y[0], self.x[1], self.y[1], self.x[2], self.y[2],
        ];

        // The gradient objects declarations
        //----------------
        let mut gradient_func = agg::GradientCircle {};
        let mut alpha_func = agg::GradientXY {};
        let mut gradient_mtx = agg::TransAffine::new_default();
        let alpha_mtx = agg::TransAffine::parl_to_rect(&parallelogram, -100., -100., 100., 100.);

        // Finally we can draw a circle.
        //----------------
        gradient_mtx *= agg::TransAffine::trans_affine_scaling(0.75, 1.2);
        gradient_mtx *= agg::TransAffine::trans_affine_rotation(-PI / 3.0);
        gradient_mtx *= agg::TransAffine::trans_affine_translation(w / 2., h / 2.);
        gradient_mtx.invert();

        let mut span_interpolator = agg::SpanIpLinear::new(gradient_mtx);
        let mut span_interpolator_alpha = agg::SpanIpLinear::new(alpha_mtx);
        let mut span_allocator = agg::VecSpan::<ColorType>::new();
        let mut color_array = ColorFunc::new();

        // Declare the gradient span itself.
        // The last two arguments are so called "d1" and "d2"
        // defining two distances in pixels, where the gradient starts
        // and where it ends. The actual meaning of "d1" and "d2" depands
        // on the gradient function.
        //----------------
        let mut span_gradient = agg::SpanGradient::<
            ColorType,
            agg::SpanIpLinear<agg::TransAffine>,
            agg::GradientCircle,
            ColorFunc<ColorType>,
        >::new(
            &mut span_interpolator,
            &mut gradient_func,
            &mut color_array,
            0.,
            150.,
        );

        // Declare the gradient span itself.
        // The last two arguments are so called "d1" and "d2"
        // defining two distances in pixels, where the gradient starts
        // and where it ends. The actual meaning of "d1" and "d2" depands
        // on the gradient function.
        //----------------
        let mut alpha_array = AlphaFunc::<<ColorType as Args>::ValueType>::new();
        // Fill Alpha array
        //----------------
        for i in 0..256 {
            alpha_array[i] = <ColorType as Args>::ValueType::from_f64(
                self.alpha.borrow_mut().value(i as f64 / 255.0) * (ColorType::BASE_MASK as f64),
            );
        }

        let mut span_gradient_alpha = agg::SpanGradientAlpha::<
            ColorType,
            agg::SpanIpLinear<agg::TransAffine>,
            agg::GradientXY,
            AlphaFunc<<ColorType as Args>::ValueType>,
        >::new(
            &mut span_interpolator_alpha,
            &mut alpha_func,
            &mut alpha_array,
            0.,
            100.,
        );

        Self::fill_color_array(
            span_gradient.color_function_mut(),
            ColorType::new_from_rgba(&agg::Rgba::new_params(0.0, 0.19, 0.19, 0.0)),
            ColorType::new_from_rgba(&agg::Rgba::new_params(0.7, 0.7, 0.19, 0.0)),
            ColorType::new_from_rgba(&agg::Rgba::new_params(0.31, 0.0, 0.0, 0.0)),
        );

        // Span converter declaration
        let mut span_conv = agg::SpanProcess::new(&mut span_gradient, &mut span_gradient_alpha);

        ell.init(w / 2., h / 2., 150., 150., 100, false);
        ras.add_path(&mut ell, 0);

        // Render the circle with gradient plus alpha-gradient
        agg::render_scanlines_aa(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut span_allocator,
            &mut span_conv,
        );

        // Draw the control points and the parallelogram
        //-----------------
        let color_pnt = ColorType::new_from_rgba(&agg::Rgba::new_params(0.0, 0.4, 0.4, 0.31));
        ell.init(self.x[0], self.y[0], 5., 5., 20, false);
        ras.add_path(&mut ell, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut ren_base, &color_pnt);
        ell.init(self.x[1], self.y[1], 5., 5., 20, false);
        ras.add_path(&mut ell, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut ren_base, &color_pnt);
        ell.init(self.x[2], self.y[2], 5., 5., 20, false);
        ras.add_path(&mut ell, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut ren_base, &color_pnt);

        let mut stroke = agg::VcgenStroke::new();
        stroke.add_vertex(self.x[0], self.y[0], PathCmd::MoveTo as u32);
        stroke.add_vertex(self.x[1], self.y[1], PathCmd::LineTo as u32);
        stroke.add_vertex(self.x[2], self.y[2], PathCmd::LineTo as u32);
        stroke.add_vertex(
            self.x[0] + self.x[2] - self.x[1],
            self.y[0] + self.y[2] - self.y[1],
            PathCmd::LineTo as u32,
        );
        stroke.add_vertex(0., 0., PathCmd::EndPoly as u32 | PathFlag::Close as u32);
        ras.add_path(&mut stroke, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &ColorType::new_from_rgba(&agg::Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
        );

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.alpha.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption(r#"AGG Example. Alpha channel gradient"#);

    if plat.init(400, 320, WindowFlag::Resize as u32) {
        plat.run();
    }
}

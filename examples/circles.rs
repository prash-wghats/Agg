use crate::platform::*;
use agg::rendering_buffer::RenderBuf;
use agg::{Color, RasterScanLine};

mod ctrl;
mod platform;
use crate::ctrl::scale::Scale;
use crate::ctrl::slider::Slider;

use core::f64::consts::PI;
use libc::*;
use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

fn frand() -> i32 {
    unsafe { rand() }
}

const FLIP_Y: bool = true;

const DEFAULT_NUM_POINTS: u32 = 10000;

const START_WIDTH: u32 = 400;
const START_HEIGHT: u32 = 400;

static SPLINE_R_X: [f64; 6] = [0.0, 0.2, 0.4, 0.910484, 0.957258, 1.0];
static SPLINE_R_Y: [f64; 6] = [1.0, 0.8, 0.6, 0.066667, 0.169697, 0.6];

static SPLINE_G_X: [f64; 6] = [0.0, 0.292244, 0.485655, 0.564859, 0.795607, 1.0];
static SPLINE_G_Y: [f64; 6] = [0.0, 0.607260, 0.964065, 0.892558, 0.435571, 0.0];

static SPLINE_B_X: [f64; 6] = [0.0, 0.055045, 0.143034, 0.433082, 0.764859, 1.0];
static SPLINE_B_Y: [f64; 6] = [0.385480, 0.128493, 0.021416, 0.271507, 0.713974, 1.0];

struct ScatterPoint {
    x: f64,
    y: f64,
    z: f64,
    color: agg::Rgba,
}

fn random_dbl(start: f64, end: f64) -> f64 {
    let r = frand() & 0x7FFF;
    return (r as f64) * (end - start) / 32768.0 + start;
}

struct Application {
    num_points: usize,
    points: Vec<ScatterPoint>,
    scale_ctrl_z: Ptr<Scale<agg::Rgba8>>,
    slider_ctrl_sel: Ptr<Slider<'static, agg::Rgba8>>,
    slider_ctrl_size: Ptr<Slider<'static, agg::Rgba8>>,
    spline_r: agg::Bspline,
    spline_g: agg::Bspline,
    spline_b: agg::Bspline,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    pub fn generate(&mut self) {
        let rx = self.util.borrow().initial_width() / 3.5;
        let ry = self.util.borrow().initial_height() / 3.5;

        for _ in 0..self.num_points {
            let z = random_dbl(0.0, 1.0);
            let x = (z * 2.0 * PI).cos() * rx;
            let y = (z * 2.0 * PI).sin() * ry;

            let dist = random_dbl(0.0, rx / 2.0);
            let angle = random_dbl(0.0, PI * 2.0);

            let x = self.util.borrow().initial_width() / 2.0 + x + (angle).cos() * dist;
            let y = self.util.borrow().initial_height() / 2.0 + y + (angle).sin() * dist;
            let color = agg::Rgba::new_params(
                self.spline_r.get(z) * 0.8,
                self.spline_g.get(z) * 0.8,
                self.spline_b.get(z) * 0.8,
                1.0,
            );

            self.points.push(ScatterPoint { x, y, z, color });
        }
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let start_width = START_WIDTH as f64;
        let _start_height = START_HEIGHT as f64;
        let num_points = DEFAULT_NUM_POINTS as usize; // < 20000
        let mut points = Vec::new();
        points.reserve(num_points as usize);
        let scale_ctrl_z = ctrl_ptr(Scale::new(5., 5., start_width - 5., 12., !flip_y));
        let slider_ctrl_sel = ctrl_ptr(Slider::new(5., 20., start_width - 5., 27., !flip_y));
        let slider_ctrl_size = ctrl_ptr(Slider::new(5., 35., start_width - 5., 42., !flip_y));
        slider_ctrl_sel.borrow_mut().set_label("Size=%4.3f");
        slider_ctrl_size.borrow_mut().set_label("Size=%4.3f");

        let mut app = Application {
            num_points,
            points,
            scale_ctrl_z: scale_ctrl_z.clone(),
            slider_ctrl_sel: slider_ctrl_sel.clone(),
            slider_ctrl_size: slider_ctrl_size.clone(),
            spline_r: agg::Bspline::new(),
            spline_g: agg::Bspline::new(),
            spline_b: agg::Bspline::new(),
            ctrls: CtrlContainer {
                ctrl: vec![scale_ctrl_z, slider_ctrl_sel, slider_ctrl_size],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        };
        app.spline_r.init_with_points(6, &SPLINE_R_X, &SPLINE_R_Y);
        app.spline_g.init_with_points(6, &SPLINE_G_X, &SPLINE_G_Y);
        app.spline_b.init_with_points(6, &SPLINE_B_X, &SPLINE_B_Y);

        app
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut sl = agg::ScanlineP8::new();
        let mut pf: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&ColorType::new_params(255, 255, 255, 255));

        let e1 = agg::Ellipse::new();
        let mut t1 =
            agg::ConvTransform::new_owned(e1, *self.util.borrow_mut().trans_affine_resizing());

        let mut n_drawn = 0;
        for i in 0..self.num_points {
            let z = self.points[i].z;
            let mut alpha = 1.0;
            if z < self.scale_ctrl_z.borrow().value1() {
                alpha = 1.0
                    - (self.scale_ctrl_z.borrow().value1() - z)
                        * self.slider_ctrl_sel.borrow().value()
                        * 100.0;
            }
            if z > self.scale_ctrl_z.borrow().value2() {
                alpha = 1.0
                    - (z - self.scale_ctrl_z.borrow().value2())
                        * self.slider_ctrl_sel.borrow().value()
                        * 100.0;
            }

            if alpha > 1.0 {
                alpha = 1.0;
            }
            if alpha < 0.0 {
                alpha = 0.0;
            }

            if alpha > 0.0 {
                t1.source_mut().init(
                    self.points[i].x,
                    self.points[i].y,
                    self.slider_ctrl_size.borrow().value() * 5.0,
                    self.slider_ctrl_size.borrow().value() * 5.0,
                    8,
                    false,
                );
                pf.add_path(&mut t1, 0);

                agg::render_scanlines_aa_solid(
                    &mut pf,
                    &mut sl,
                    &mut rb,
                    &ColorType::new_from_rgba(&agg::Rgba::new_params(
                        self.points[i].color.r,
                        self.points[i].color.g,
                        self.points[i].color.b,
                        alpha,
                    )),
                );
                n_drawn += 1;
            }
        }

        ctrl::render_ctrl(
            &mut pf,
            &mut sl,
            &mut rb,
            &mut *self.scale_ctrl_z.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut pf,
            &mut sl,
            &mut rb,
            &mut *self.slider_ctrl_sel.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut pf,
            &mut sl,
            &mut rb,
            &mut *self.slider_ctrl_size.borrow_mut(),
        );

        let mut txt = agg::GsvText::new();
        txt.set_size(15.0, 0.);
        txt.set_text(&format!("{:08}", n_drawn));
        txt.set_start_point(10.0, self.util.borrow().initial_height() - 20.0);
        let mut txt_o = agg::GsvTextOutline::new(txt, *self.util.borrow().trans_affine_resizing());
        pf.add_path(&mut txt_o, 0);
        agg::render_scanlines_aa_solid(
            &mut pf,
            &mut sl,
            &mut rb,
            &ColorType::new_from_rgba(&agg::Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
        );
    }

    fn on_init(&mut self) {
        self.generate();
    }

    fn on_idle(&mut self) -> Draw {
        for point in &mut self.points {
            point.x += random_dbl(0.0, self.slider_ctrl_sel.borrow().value())
                - self.slider_ctrl_sel.borrow().value() * 0.5;
            point.y += random_dbl(0.0, self.slider_ctrl_sel.borrow().value())
                - self.slider_ctrl_sel.borrow().value() * 0.5;
            point.z += random_dbl(0.0, self.slider_ctrl_sel.borrow().value() * 0.01)
                - self.slider_ctrl_sel.borrow().value() * 0.005;

            if point.z < 0.0 {
                point.z = 0.0;
            }
            if point.z > 1.0 {
                point.z = 1.0;
            }
        }
        return Draw::Yes;
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, _x: i32, _y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.generate();
            return Draw::Yes;
        }

        if flags & InputFlag::MouseRight as u32 != 0 {
            let wmode = self.util.borrow_mut().wait_mode();
            self.util.borrow_mut().set_wait_mode(!wmode);
            return Draw::Yes;
        }
        Draw::No
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Drawing random circles - A scatter plot prototype");

    if plat.init(START_WIDTH, START_HEIGHT, WindowFlag::Resize as u32) {
        plat.run();
    }
}


use agg::rendering_buffer::RenderBuf;
use agg::{RasterScanLine};

mod ctrl;
mod platform;
use crate::platform::*;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}


const FLIP_Y: bool = true;

struct Application {
    x: [f64; 3],
    y: [f64; 3],
    dx: f64,
    dy: f64,
    idx: i32,
    dilation: Ptr<Slider<'static, agg::Rgba8>>,
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    alpha: Ptr<Slider<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut dilation = Slider::new(5., 5., 400. - 5., 11., !flip_y);
        let mut gamma = Slider::new(5., 5. + 15., 400. - 5., 11. + 15., !flip_y);
        let mut alpha = Slider::new(5., 5. + 30., 400. - 5., 11. + 30., !flip_y);
        dilation.set_label("Dilation=%3.2f");
        gamma.set_label("Linear gamma=%3.2f");
        alpha.set_label("Opacity=%3.2f");

        dilation.set_value(0.175);
        gamma.set_value(0.809);
        alpha.set_value(1.0);
        let dilation = ctrl_ptr(dilation);
        let gamma = ctrl_ptr(gamma);
        let alpha = ctrl_ptr(alpha);

        Application {
            dilation: dilation.clone(),
            gamma: gamma.clone(),
            alpha: alpha.clone(),
            x: [57.0, 369.0, 143.0],
            y: [60.0, 170.0, 310.0],
            dx: 0.0,
            dy: 0.0,
            idx: -1,
            ctrls: CtrlContainer {
                ctrl: vec![dilation, gamma, alpha],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut sl = agg::ScanlineU8::new();
        let mut ras = agg::RasterizerScanlineAa::new();
        let mut pf = Pixfmt::new_owned(rb.clone());
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pf);
        ren_base.clear(&ColorType::new_params(255, 255, 255, 255));
        self.render_gouraud(rb, &mut sl, &mut ras);

        let g = agg::GammaNone::new();
        ras.set_gamma(&g);
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.dilation.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.gamma.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.alpha.borrow_mut(),
        );
    }

    fn on_mouse_button_down(
        &mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32,
    ) -> Draw {
        let mut j: _;
        if flags & InputFlag::MouseRight as u32 != 0 {
            let mut sl = agg::ScanlineU8::new();
            let mut ras = agg::RasterizerScanlineAa::new();
            self.util.borrow_mut().start_timer();
            for _i in 0..100 {
                self.render_gouraud(rb, &mut sl, &mut ras);
            }
            let buf = format!("Time={} ms", self.util.borrow_mut().elapsed_time());
            self.util.borrow_mut().message(&buf);
        }
        let x = x as f64;
        let y = y as f64;
        if flags & InputFlag::MouseLeft as u32 != 0 {
            j = 0;
            for i in 0..3 {
                if (x - self.x[i]) * (x - self.x[i]) + (y - self.y[i]) * (y - self.y[i]) < 10.0 {
                    self.dx = x - self.x[i];
                    self.dy = y - self.y[i];
                    self.idx = i as i32;
                    j = i;
                    break;
                }
                j = i;
            }
            if j == 3 {
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

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        let x = x as f64;
        let y = y as f64;
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
                return Draw::Yes;
            }

            if self.idx >= 0 {
                self.x[self.idx as usize] = x - self.dx;
                self.y[self.idx as usize] = y - self.dy;
                return Draw::Yes;
            }
        } else {
            return self.on_mouse_button_up(_rb, x as i32, y as i32, flags);
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
        let mut dx = 0.;
        let mut dy = 0.;
        match key {
            x if x == KeyCode::Left as u32 => dx = -0.1,
            x if x == KeyCode::Right as u32 => dx = 0.1,
            x if x == KeyCode::Up as u32 => dy = 0.1,
            x if x == KeyCode::Down as u32 => dy = -0.1,
            _ => (),
        }
        self.x[0] += dx;
        self.y[0] += dy;
        self.x[1] += dx;
        self.y[1] += dy;
        Draw::Yes
    }
}

impl Application {
    fn render_gouraud(
        &mut self, rb: &mut RenderBuf, sl: &mut agg::ScanlineU8,
        ras: &mut agg::RasterizerScanlineAa,
    ) {
        let alpha = (self.alpha.borrow().value() * 255.) as u32;
        let mut brc = (1.0 * 255.) as u32;

        let mut pf = Pixfmt::new_borrowed(rb);
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pf);
        let mut span_alloc = agg::VecSpan::new();

		#[cfg(feature = "agg_gray8")]
		let mut span_gen = agg::SpanGouraudGray::new_default();
		#[cfg(not(feature = "agg_gray8"))]
		let mut span_gen = agg::SpanGouraudRgba::new_default();

        let g = agg::GammaLinear::new_with_start_end(0.0, self.gamma.borrow().value());
        ras.set_gamma(&g);

        let d = self.dilation.borrow().value();

        // Single triangle
        //span_gen.colors(ColorType(1,   0,   0,  alpha),
        //                ColorType(0,   1,   0,  alpha),
        //                ColorType(0,   0,   1,  alpha));
        //span_gen.triangle(m_x[0], m_y[0], m_x[1], m_y[1], m_x[2], m_y[2], d);
        //ras.add_path(span_gen);
        //agg::render_scanlines_aa(ras, sl, ren_base, span_alloc, span_gen);

        // Six triangles
        let xc = (self.x[0] + self.x[1] + self.x[2]) / 3.0;
        let yc = (self.y[0] + self.y[1] + self.y[2]) / 3.0;

        let x1 = (self.x[1] + self.x[0]) / 2. - (xc - (self.x[1] + self.x[0]) / 2.);
        let y1 = (self.y[1] + self.y[0]) / 2. - (yc - (self.y[1] + self.y[0]) / 2.);

        let x2 = (self.x[2] + self.x[1]) / 2. - (xc - (self.x[2] + self.x[1]) / 2.);
        let y2 = (self.y[2] + self.y[1]) / 2. - (yc - (self.y[2] + self.y[1]) / 2.);

        let x3 = (self.x[0] + self.x[2]) / 2. - (xc - (self.x[0] + self.x[2]) / 2.);
        let y3 = (self.y[0] + self.y[2]) / 2. - (yc - (self.y[0] + self.y[2]) / 2.);

        span_gen.set_colors(
            ColorType::new_params(255, 0, 0, alpha),
            ColorType::new_params(0, 255, 0, alpha),
            ColorType::new_params(brc, brc, brc, alpha),
        );
        span_gen.set_triangle(self.x[0], self.y[0], self.x[1], self.y[1], xc, yc, d);
        ras.add_path(&mut span_gen, 0);
        agg::render_scanlines_aa(ras, sl, &mut ren_base, &mut span_alloc, &mut span_gen);

        span_gen.set_colors(
            ColorType::new_params(0, 255, 0, alpha),
            ColorType::new_params(0, 0, 255, alpha),
            ColorType::new_params(brc, brc, brc, alpha),
        );
        span_gen.set_triangle(self.x[1], self.y[1], self.x[2], self.y[2], xc, yc, d);
        ras.add_path(&mut span_gen, 0);
        agg::render_scanlines_aa(ras, sl, &mut ren_base, &mut span_alloc, &mut span_gen);

        span_gen.set_colors(
            ColorType::new_params(0, 0, 255, alpha),
            ColorType::new_params(255, 0, 0, alpha),
            ColorType::new_params(brc, brc, brc, alpha),
        );
        span_gen.set_triangle(self.x[2], self.y[2], self.x[0], self.y[0], xc, yc, d);
        ras.add_path(&mut span_gen, 0);
        agg::render_scanlines_aa(ras, sl, &mut ren_base, &mut span_alloc, &mut span_gen);

        brc = 255 - brc;
        span_gen.set_colors(
            ColorType::new_params(255, 0, 0, alpha),
            ColorType::new_params(0, 255, 0, alpha),
            ColorType::new_params(brc, brc, brc, alpha),
        );
        span_gen.set_triangle(self.x[0], self.y[0], self.x[1], self.y[1], x1, y1, d);
        ras.add_path(&mut span_gen, 0);
        agg::render_scanlines_aa(ras, sl, &mut ren_base, &mut span_alloc, &mut span_gen);

        span_gen.set_colors(
            ColorType::new_params(0, 255, 0, alpha),
            ColorType::new_params(0, 0, 255, alpha),
            ColorType::new_params(brc, brc, brc, alpha),
        );
        span_gen.set_triangle(self.x[1], self.y[1], self.x[2], self.y[2], x2, y2, d);
        ras.add_path(&mut span_gen, 0);
        agg::render_scanlines_aa(ras, sl, &mut ren_base, &mut span_alloc, &mut span_gen);

        span_gen.set_colors(
            ColorType::new_params(0, 0, 255, alpha),
            ColorType::new_params(255, 0, 0, alpha),
            ColorType::new_params(brc, brc, brc, alpha),
        );
        span_gen.set_triangle(self.x[2], self.y[2], self.x[0], self.y[0], x3, y3, d);
        ras.add_path(&mut span_gen, 0);
        agg::render_scanlines_aa(ras, sl, &mut ren_base, &mut span_alloc, &mut span_gen);
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Gouraud Shading");

    if plat.init(400, 320, WindowFlag::Resize as u32) {
        plat.run();
    }
}

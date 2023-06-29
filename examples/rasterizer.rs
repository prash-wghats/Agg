use crate::ctrl::cbox::Cbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;
use agg::{RasterScanLine, RenderBuf, RendererScanlineColor};

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

struct Application {
    x: [f64; 3],
    y: [f64; 3],
    dx: f64,
    dy: f64,
    idx: i32,
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    alpha: Ptr<Slider<'static, agg::Rgba8>>,
    test: Ptr<Cbox<'static, agg::Rgba8>>,
    ras: agg::RasterizerScanlineAa,
    sl_p8: agg::ScanlineP8,
    sl_bin: agg::ScanlineBin,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn draw_anti_aliased(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut ren_aa = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        let mut path = agg::PathStorage::new();

        path.move_to(self.x[0], self.y[0]);
        path.line_to(self.x[1], self.y[1]);
        path.line_to(self.x[2], self.y[2]);
        path.close_polygon(0);

        ren_aa.set_color(agg::Rgba8::new_params(
            175,
            125,
            25,
            (self.alpha.borrow().value() * 255.) as u32,
        ));

        self.ras.set_gamma(&agg::GammaPower::new_with_gamma(
            self.gamma.borrow().value() * 2.0,
        ));
        self.ras.add_path(&mut path, 0);
        agg::render_scanlines(&mut self.ras, &mut self.sl_p8, &mut ren_aa);
    }

    fn draw_aliased(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut ren_bin = agg::RendererScanlineBinSolid::new_borrowed(&mut rb);

        let mut path = agg::PathStorage::new();

        path.move_to(self.x[0] - 200.0, self.y[0]);
        path.line_to(self.x[1] - 200.0, self.y[1]);
        path.line_to(self.x[2] - 200.0, self.y[2]);
        path.close_polygon(0);

        ren_bin.set_color(agg::Rgba8::new_params(
            25,
            125,
            175,
            (self.alpha.borrow().value() * 255.) as u32,
        ));

        self.ras.set_gamma(&agg::GammaThreshold::new_with_threshold(
            self.gamma.borrow().value(),
        ));
        self.ras.add_path(&mut path, 0);
        agg::render_scanlines(&mut self.ras, &mut self.sl_bin, &mut ren_bin);
    }
}
impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let gamma = ctrl_ptr(Slider::new(
            130.0 + 10.0,
            10.0 + 4.0,
            130.0 + 150.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let alpha = ctrl_ptr(Slider::new(
            130.0 + 150.0 + 10.0,
            10.0 + 4.0,
            500.0 - 10.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let test = ctrl_ptr(Cbox::new(
            130.0 + 10.0,
            10.0 + 4.0 + 16.0,
            "Test Performance",
            !flip_y,
        ));
        let mut the_app = Application {
            x: [0.0; 3],
            y: [0.0; 3],
            dx: 0.0,
            dy: 0.0,
            idx: -1,
            gamma: gamma.clone(),
            alpha: alpha.clone(),
            test: test.clone(),
            ras: agg::RasterizerScanlineAa::new(),
            sl_p8: agg::ScanlineP8::new(),
            sl_bin: agg::ScanlineBin::new(),
            ctrls: CtrlContainer {
                ctrl: vec![gamma, alpha, test],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        };

        the_app.x[0] = 100.0 + 120.0;
        the_app.y[0] = 60.0;
        the_app.x[1] = 369.0 + 120.0;
        the_app.y[1] = 170.0;
        the_app.x[2] = 143.0 + 120.0;
        the_app.y[2] = 310.0;

        the_app.gamma.borrow_mut().set_range(0.0, 1.0);
        the_app.gamma.borrow_mut().set_value(0.5);
        the_app.gamma.borrow_mut().set_label("Gamma=%1.2f");
        the_app.gamma.borrow_mut().no_transform();

        the_app.alpha.borrow_mut().set_range(0.0, 1.0);
        the_app.alpha.borrow_mut().set_value(1.0);
        the_app.alpha.borrow_mut().set_label("Alpha=%1.2f");
        the_app.alpha.borrow_mut().no_transform();

        the_app.test.borrow_mut().no_transform();

        the_app
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.idx == 3 {
                let dx = x as f64 - self.dx;
                let dy = y as f64 - self.dy;
                self.x[1] -= self.x[0] - dx;
                self.y[1] -= self.y[0] - dy;
                self.x[2] -= self.x[0] - dx;
                self.y[2] -= self.y[0] - dy;
                self.x[0] = dx;
                self.y[0] = dy;
                return Draw::Yes;
            } else if self.idx >= 0 {
                self.x[self.idx as usize] = x as f64 - self.dx;
                self.y[self.idx as usize] = y as f64 - self.dy;
                return Draw::Yes;
            }
        } else {
            return self.on_mouse_button_up(rb, x, y, flags);
        }
        Draw::No
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.idx = -1;
        Draw::No
    }

    fn on_ctrl_change(&mut self, rb: &mut agg::RenderBuf) {
        if self.test.borrow().status() {
            self.on_draw(rb);
            //self.update_window();
            self.test.borrow_mut().set_status(false);

            self.util.borrow_mut().start_timer();

            for _ in 0..1000 {
                self.draw_aliased(rb);
            }
            let t1 = self.util.borrow_mut().elapsed_time();

            self.util.borrow_mut().start_timer();
            for _ in 0..1000 {
                self.draw_anti_aliased(rb);
            }
            let t2 = self.util.borrow_mut().elapsed_time();

            //self.update_window();
            let buf = format!("Time Aliased= {:.2}ms Time Anti-Aliased= {:.2}ms", t1, t2);
            self.util.borrow_mut().message(&buf);
        }
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = Pixfmt::new_owned(rbuf.clone());
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        self.draw_anti_aliased(rbuf);
        self.draw_aliased(rbuf);

        let _ren_aa = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        let mut ras_aa: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        ctrl::render_ctrl(
            &mut ras_aa,
            &mut self.sl_p8,
            &mut rb,
            &mut *self.gamma.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras_aa,
            &mut self.sl_p8,
            &mut rb,
            &mut *self.alpha.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras_aa,
            &mut self.sl_p8,
            &mut rb,
            &mut *self.test.borrow_mut(),
        );
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32,
    ) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let mut i = 0;
            while i < 3 {
                let left_distance =
                    (x as f64 - self.x[i]).powf(2.) + (y as f64 - self.y[i]).powf(2.);
                let right_distance =
                    (x as f64 - self.x[i] + 200.).powf(2.) + (y as f64 - self.y[i]).powf(2.);

                if left_distance.sqrt() < 20.0 || right_distance.sqrt() < 20.0 {
                    self.dx = x as f64 - self.x[i];
                    self.dy = y as f64 - self.y[i];
                    self.idx = i as i32;
                    break;
                }
                i += 1;
            }

            if i == 3 {
                let left_triangle = agg::point_in_triangle(
                    self.x[0], self.y[0], self.x[1], self.y[1], self.x[2], self.y[2], x as f64,
                    y as f64,
                );
                let right_triangle = agg::point_in_triangle(
                    self.x[0] - 200.,
                    self.y[0],
                    self.x[1] - 200.,
                    self.y[1],
                    self.x[2] - 200.,
                    self.y[2],
                    x as f64,
                    y as f64,
                );

                if left_triangle || right_triangle {
                    self.dx = x as f64 - self.x[0];
                    self.dy = y as f64 - self.y[0];
                    self.idx = 3;
                }
            }
        }
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
        return Draw::Yes;
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Line Join");

    if plat.init(500, 330, WindowFlag::Resize as u32) {
        plat.run();
    }
}

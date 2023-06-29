use crate::platform::*;
use agg::{RasterScanLine, VertexSourceWithMarker};

mod ctrl;
mod platform;
use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::rc::Rc;

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
    cap: Ptr<Rbox<'static, agg::Rgba8>>,
    width: Ptr<Slider<'static, agg::Rgba8>>,
    smooth: Ptr<Slider<'static, agg::Rgba8>>,
    close: Ptr<Cbox<'static, agg::Rgba8>>,
    even_odd: Ptr<Cbox<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    _util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let cap = ctrl_ptr(Rbox::new(10.0, 10.0, 130.0, 80.0, !flip_y));
        let width = ctrl_ptr(Slider::new(
            130.0 + 10.0,
            10.0 + 4.0,
            130.0 + 150.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let smooth = ctrl_ptr(Slider::new(
            130.0 + 150.0 + 10.0,
            10.0 + 4.0,
            500.0 - 10.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let close = ctrl_ptr(Cbox::new(
            130.0 + 10.0,
            10.0 + 4.0 + 16.0,
            "Close Polygons",
            !flip_y,
        ));
        let even_odd = ctrl_ptr(Cbox::new(
            130.0 + 150.0 + 10.0,
            10.0 + 4.0 + 16.0,
            "Even-Odd Fill",
            !flip_y,
        ));
        cap.borrow_mut().add_item("Butt Cap");
        cap.borrow_mut().add_item("Square Cap");
        cap.borrow_mut().add_item("Round Cap");
        cap.borrow_mut().set_cur_item(0);
        cap.borrow_mut().no_transform();

        width.borrow_mut().set_range(0.0, 10.0);
        width.borrow_mut().set_value(3.0);
        width.borrow_mut().set_label("Width=%1.2f");
        width.borrow_mut().no_transform();

        smooth.borrow_mut().set_range(0.0, 2.0);
        smooth.borrow_mut().set_value(1.0);
        smooth.borrow_mut().set_label("Smooth=%1.2f");
        smooth.borrow_mut().no_transform();

        close.borrow_mut().no_transform();

        even_odd.borrow_mut().no_transform();

        Application {
            x: [57.0 + 100.0, 369.0 + 100.0, 143.0 + 100.0],
            y: [60.0, 170.0, 310.0],
            dx: 0.0,
            dy: 0.0,
            idx: -1,

            ctrls: CtrlContainer {
                ctrl: vec![
                    cap.clone(),
                    width.clone(),
                    smooth.clone(),
                    close.clone(),
                    even_odd.clone(),
                ],
                cur_ctrl: -1,
                num_ctrl: 5,
            },
            cap,
            width,
            smooth,
            close,
            even_odd,
            _util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32,
    ) -> Draw {
        let (x, y) = (x as f64, y as f64);
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let mut i = 0;

            for _ in 0..3 {
                if ((x - self.x[i]) * (x - self.x[i]) + (y - self.y[i]) * (y - self.y[i])).sqrt()
                    < 20.0
                {
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
            }

            if self.idx >= 0 {
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

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut renb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        let mut sl = agg::ScanlineP8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        renb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let cap = match self.cap.borrow().cur_item() {
            0 => agg::LineCap::Butt,
            1 => agg::LineCap::Square,
            2 => agg::LineCap::Round,
            _ => unreachable!(),
        };

        let mut path = agg::PathStorage::new();

        path.move_to(self.x[0], self.y[0]);
        path.line_to(self.x[1], self.y[1]);
        path.line_to(
            (self.x[0] + self.x[1] + self.x[2]) / 3.0,
            (self.y[0] + self.y[1] + self.y[2]) / 3.0,
        );
        path.line_to(self.x[2], self.y[2]);
        if self.close.borrow().status() {
            path.close_polygon(0);
        }

        path.move_to((self.x[0] + self.x[1]) / 2.0, (self.y[0] + self.y[1]) / 2.0);
        path.line_to((self.x[1] + self.x[2]) / 2.0, (self.y[1] + self.y[2]) / 2.0);
        path.line_to((self.x[2] + self.x[0]) / 2.0, (self.y[2] + self.y[0]) / 2.0);
        if self.close.borrow().status() {
            path.close_polygon(0);
        }

        if self.even_odd.borrow().status() {
            ras.set_filling_rule(agg::FillingRule::FillEvenOdd);
        }

        // (1)
        ras.add_path(&mut path, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(180, 128, 25, 128),
        );
        // (1)

        // Start of (2, 3, 4)
        let mut smooth = agg::ConvSmoothPoly1::new_borrowed(&mut path);
        smooth.set_smooth_value(self.smooth.borrow().value());

        // (2)
        ras.add_path(&mut smooth, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(25, 128, 180, 25),
        );
        // (2)

        // (3)
        let mut smooth_outline: agg::ConvStroke<'_, _> = agg::ConvStroke::new_borrowed(&mut smooth);
        ras.add_path(&mut smooth_outline, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(0, 153, 0, 204),
        );
        // (3)

        // (4)
        let curve: agg::ConvCurve<'_, _> = agg::ConvCurve::new_borrowed(&mut smooth);
        let mut dash: agg::ConvDash<'_, _, agg::VcgenMarkersTerm> = agg::ConvDash::new_owned(curve);

        let k = self.width.borrow().value().powf(0.7);

        let mut ah = agg::Arrowhead::new();
        ah.head(4.0 * k, 4.0 * k, 3.0 * k, 2.0 * k);
        if !self.close.borrow().status() {
            ah.tail(1.0 * k, 1.5 * k, 3.0 * k, 5.0 * k);
        }

        dash.add_dash(20.0, 5.0);
        dash.add_dash(5.0, 5.0);
        dash.add_dash(5.0, 5.0);
        dash.dash_start(10.0);

        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(dash);
        stroke.set_line_cap(cap);
        stroke.set_width(self.width.borrow().value());

        ras.add_path(&mut stroke, 0);
        let mut arrow = agg::ConvMarker::new_borrowed(stroke.source_mut().markers_mut(), &mut ah);
        ras.add_path(&mut arrow, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(0, 0, 0, 255),
        );
        // (4)

        ras.set_filling_rule(agg::FillingRule::FillNonZero);
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.cap.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.width.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.smooth.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.close.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut renb,
            &mut *self.even_odd.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Line Join");

    if plat.init(500, 330, WindowFlag::Resize as u32) {
        plat.run();
    }
}

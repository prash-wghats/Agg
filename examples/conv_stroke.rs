use crate::platform::*;

use agg::rendering_buffer::RenderBuf;
use agg::RasterScanLine;

mod ctrl;
mod platform;

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
    join: Ptr<Rbox<'static, agg::Rgba8>>,
    cap: Ptr<Rbox<'static, agg::Rgba8>>,
    width: Ptr<Slider<'static, agg::Rgba8>>,
    miter_limit: Ptr<Slider<'static, agg::Rgba8>>,

    pub ctrls: CtrlContainer,
    _util: Rc<RefCell<PlatUtil>>,
}
impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let join = ctrl_ptr(Rbox::new(10.0, 10.0, 133.0, 80.0, !flip_y));
        let cap = ctrl_ptr(Rbox::new(10.0, 80. + 10.0, 133.0, 80. + 80.0, !flip_y));
        let width = ctrl_ptr(Slider::new(
            130.0 + 10.0,
            10.0 + 4.0,
            500.0 - 10.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let miter_limit = ctrl_ptr(Slider::new(
            130. + 10.0,
            20.0 + 10.0 + 4.0,
            500.0 - 10.0,
            20.0 + 10.0 + 8.0 + 4.0,
            !flip_y,
        ));

        join.borrow_mut().set_text_size(7.5, 0.);
        join.borrow_mut().set_text_thickness(1.0);
        join.borrow_mut().add_item("Miter Join");
        join.borrow_mut().add_item("Miter Join Revert");
        join.borrow_mut().add_item("Round Join");
        join.borrow_mut().add_item("Bevel Join");
        join.borrow_mut().set_cur_item(2);

        cap.borrow_mut().add_item("Butt Cap");
        cap.borrow_mut().add_item("Square Cap");
        cap.borrow_mut().add_item("Round Cap");
        cap.borrow_mut().set_cur_item(2);

        width.borrow_mut().set_range(3.0, 40.0);
        width.borrow_mut().set_value(20.0);
        width.borrow_mut().set_label("Width=%1.2f");

        miter_limit.borrow_mut().set_range(1.0, 10.0);
        miter_limit.borrow_mut().set_value(4.0);
        miter_limit.borrow_mut().set_label("Miter Limit=%1.2f");

        Application {
            x: [57.0 + 100.0, 369.0 + 100.0, 143.0 + 100.0],
            y: [60.0, 170.0, 310.0],
            dx: 0.0,
            dy: 0.0,
            idx: -1,

            ctrls: CtrlContainer {
                ctrl: vec![
                    join.clone(),
                    cap.clone(),
                    width.clone(),
                    miter_limit.clone(),
                ],
                cur_ctrl: -1,
                num_ctrl: 4,
            },
            cap,
            width,
            join,
            miter_limit,

            _util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        let (x, y) = (x as f64, y as f64);
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let mut i = 0;
            for _ in 0..3 {
                if ((x - self.x[i]).powi(2) + (y - self.y[i]).powi(2)).sqrt() < 20.0 {
                    self.dx = x as f64 - self.x[i];
                    self.dy = y as f64 - self.y[i];
                    self.idx = i as i32;
                    break;
                }
                i += 1;
            }
            if i == 3 {
                if agg::point_in_triangle(
                    self.x[0], self.y[0], self.x[1], self.y[1], self.x[2], self.y[2], x as f64,
                    y as f64,
                ) {
                    self.dx = x as f64 - self.x[0];
                    self.dy = y as f64 - self.y[0];
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

        let mut path = agg::PathStorage::new();

        path.move_to(self.x[0], self.y[0]);
        path.line_to((self.x[0] + self.x[1]) / 2., (self.y[0] + self.y[1]) / 2.);
        path.line_to(self.x[1], self.y[1]);
        path.line_to(self.x[2], self.y[2]);
        path.line_to(self.x[2], self.y[2]);

        path.move_to((self.x[0] + self.x[1]) / 2., (self.y[0] + self.y[1]) / 2.);
        path.line_to((self.x[1] + self.x[2]) / 2., (self.y[1] + self.y[2]) / 2.);
        path.line_to((self.x[2] + self.x[0]) / 2., (self.y[2] + self.y[0]) / 2.);

        path.close_polygon(0);
        let cap = match self.cap.borrow().cur_item() {
            0 => agg::LineCap::Butt,
            1 => agg::LineCap::Square,
            2 => agg::LineCap::Round,
            _ => unreachable!(),
        };
        let join = match self.join.borrow().cur_item() {
            0 => agg::LineJoin::Miter,
            1 => agg::LineJoin::MiterRevert,
            2 => agg::LineJoin::Round,
            3 => agg::LineJoin::Bevel,
            _ => unreachable!(),
        };

        {
            // (2)
            let mut poly1: agg::ConvStroke<'_, _> = agg::ConvStroke::new_borrowed(&mut path);
            poly1.set_width(1.5);
            ras.add_path(&mut poly1, 0);
            agg::render_scanlines_aa_solid(
                &mut ras,
                &mut sl,
                &mut renb,
                &agg::Rgba8::new_params(0, 0, 0, 255),
            );
            // (2)
        }
        // (1)
        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_borrowed(&mut path);
        stroke.set_line_join(join);
        stroke.set_line_cap(cap);
        stroke.set_miter_limit(self.miter_limit.borrow().value());
        stroke.set_width(self.width.borrow().value());
        ras.add_path(&mut stroke, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(200, 175, 150, 255),
        );

        // (1)

        // (3)
        let mut poly2_dash: agg::ConvDash<'_, _> = agg::ConvDash::new_borrowed(&mut stroke);
        poly2_dash.add_dash(20.0, self.width.borrow().value() / 2.5);
        let mut poly2: agg::ConvStroke<'_, _> = agg::ConvStroke::new_borrowed(&mut poly2_dash);
        poly2.set_miter_limit(4.0);
        poly2.set_width(self.width.borrow().value() / 5.0);
        poly2.set_line_cap(cap);
        poly2.set_line_join(join);

        ras.add_path(&mut poly2, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(0, 0, 75, 255),
        );
        // (3)

        // (4)
        ras.add_path(&mut path, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(0, 0, 0, 50),
        );
        // (4)

        ras.set_filling_rule(agg::FillingRule::FillNonZero);
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.join.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.cap.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.width.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut renb,
            &mut *self.miter_limit.borrow_mut(),
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

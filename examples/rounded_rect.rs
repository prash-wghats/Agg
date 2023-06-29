use crate::platform::*;

use agg::{RasterScanLine, RenderBuf, RendererScanlineColor};

mod ctrl;
mod platform;

use crate::ctrl::cbox::Cbox;
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
    x: [f64; 2],
    y: [f64; 2],
    dx: f64,
    dy: f64,
    idx: i32,
    radius: Ptr<Slider<'static, agg::Rgba8>>,
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    offset: Ptr<Slider<'static, agg::Rgba8>>,
    white_on_black: Ptr<Cbox<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    _util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let gamma = ctrl_ptr(Slider::new(10.0, 10.0, 600.0 - 10.0, 19.0, !flip_y));
        let radius = ctrl_ptr(Slider::new(
            10.0,
            10.0 + 20.0,
            600.0 - 10.0,
            19.0 + 20.0,
            !flip_y,
        ));
        let offset = ctrl_ptr(Slider::new(
            10.0,
            10.0 + 40.0,
            600.0 - 10.0,
            19.0 + 40.0,
            !flip_y,
        ));
        let white_on_black = ctrl_ptr(Cbox::new(10.0, 10.0 + 60.0, "White on black", !flip_y));

        gamma.borrow_mut().set_label("gamma=%4.3f");
        gamma.borrow_mut().set_range(0.0, 3.0);
        gamma.borrow_mut().set_value(1.8);

        radius.borrow_mut().set_label("radius=%4.3f");
        radius.borrow_mut().set_range(0.0, 50.0);
        radius.borrow_mut().set_value(25.0);

        offset.borrow_mut().set_label("subpixel offset=%4.3f");
        offset.borrow_mut().set_range(-2.0, 3.0);

        white_on_black
            .borrow_mut()
            .set_text_color(agg::Rgba8::new_params(127, 127, 127, 255));
        white_on_black
            .borrow_mut()
            .set_inactive_color(agg::Rgba8::new_params(127, 127, 127, 255));

        Application {
            x: [100.0, 500.0],
            y: [100.0, 350.0],
            dx: 0.0,
            dy: 0.0,
            idx: -1,
            gamma: gamma.clone(),
            radius: radius.clone(),
            offset: offset.clone(),
            white_on_black: white_on_black.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![gamma, radius, offset, white_on_black],
                cur_ctrl: -1,
                num_ctrl: 4,
            },
            _util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let gamma = agg::GammaLut::new_with_gamma(self.gamma.borrow().value());
        let mut pixf = agg::PixBgr24Gamma::new_borrowed(rbuf);
        pixf.blender_mut().set_gamma_owned(gamma);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);

        if self.white_on_black.borrow().status() {
            rb.clear(&agg::Rgba8::new_params(0, 0, 0, 255));
        } else {
            rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        }

        let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineP8::new();

        let mut e = agg::Ellipse::new();

        // Render two "control" circles
        rs.set_color(agg::Rgba8::new_params(127, 127, 127, 255));
        e.init(self.x[0], self.y[0], 3.0, 3.0, 16, false);
        ras.add_path(&mut e, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);
        e.init(self.x[1], self.y[1], 3.0, 3.0, 16, false);
        ras.add_path(&mut e, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);

        let d = self.offset.borrow().value();

        // Creating a rounded rectangle
        let mut r = agg::RoundedRect::new(
            self.x[0] + d,
            self.y[0] + d,
            self.x[1] + d,
            self.y[1] + d,
            self.radius.borrow().value(),
        );
        r.normalize_radius();

        // Drawing as an outline
        let mut p: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(r);
        p.set_width(1.0);
        ras.add_path(&mut p, 0);
        if self.white_on_black.borrow().status() {
            rs.set_color(agg::Rgba8::new_params(255, 255, 255, 255));
        } else {
            rs.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        }
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);

        ras.set_gamma(&agg::GammaNone::new());

        // Render the controls
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.radius.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.gamma.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.offset.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.white_on_black.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            for i in 0..2 {
                if (x as f64 - self.x[i]).powi(2) + (y as f64 - self.y[i]).powi(2) < 5.0 {
                    self.dx = x as f64 - self.x[i];
                    self.dy = y as f64 - self.y[i];
                    self.idx = i as i32;
                    break;
                }
            }
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
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
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Rounded rectangle with gamma-correction & stuff");

    if plat.init(600, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

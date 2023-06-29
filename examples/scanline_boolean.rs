use crate::platform::*;

use agg::{RasterScanLine, RendererScanlineColor};

mod ctrl;
mod platform;

use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::{interactive_polygon::InteractivePolygon, pixel_formats::*};

type Ptr<T> = Rc<RefCell<T>>;

const FLIP_Y: bool = true;

fn generate_circles(ps: &mut agg::PathStorage, quad: &[f64], num_circles: u32, radius: f64) {
    ps.remove_all();
    for i in 0..4 {
        let n1 = i * 2;
        let n2 = if i < 3 { i * 2 + 2 } else { 0 };
        for j in 0..num_circles {
            let mut ell = agg::Ellipse::new_ellipse(
                quad[n1] + (quad[n2] - quad[n1]) * (j as f64) / (num_circles as f64),
                quad[n1 + 1] + (quad[n2 + 1] - quad[n1 + 1]) * (j as f64) / (num_circles as f64),
                radius,
                radius,
                100,
                false,
            );
            ps.concat_path(&mut ell, 0);
        }
    }
}

struct Application {
    quad1: InteractivePolygon<'static>,
    quad2: InteractivePolygon<'static>,
    trans_type: Ptr<Rbox<'static, agg::Rgba8>>,
    reset: Ptr<Cbox<'static, agg::Rgba8>>,
    mul1: Ptr<Slider<'static, agg::Rgba8>>,
    mul2: Ptr<Slider<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let trans_type = Rc::new(RefCell::new(Rbox::new(
            420.,
            5.0,
            420. + 130.0,
            145.0,
            !flip_y,
        )));
        let reset = Rc::new(RefCell::new(Cbox::new(350., 5.0, "Reset", !flip_y)));
        let mul1 = Rc::new(RefCell::new(Slider::new(5.0, 5.0, 340.0, 12.0, !flip_y)));
        let mul2 = Rc::new(RefCell::new(Slider::new(5.0, 20.0, 340.0, 27.0, !flip_y)));

        trans_type.borrow_mut().add_item("Union");
        trans_type.borrow_mut().add_item("Intersection");
        trans_type.borrow_mut().add_item("Linear XOR");
        trans_type.borrow_mut().add_item("Saddle XOR");
        trans_type.borrow_mut().add_item("Abs Diff XOR");
        trans_type.borrow_mut().add_item("A-B");
        trans_type.borrow_mut().add_item("B-A");
        trans_type.borrow_mut().set_cur_item(0);

        mul1.borrow_mut().set_value(1.0);
        mul2.borrow_mut().set_value(1.0);
        mul1.borrow_mut().set_label("Opacity1=%.3f");
        mul2.borrow_mut().set_label("Opacity2=%.3f");

        Application {
            quad1: InteractivePolygon::new(4, 5.0),
            quad2: InteractivePolygon::new(4, 5.0),
            trans_type: trans_type.clone(),
            reset: reset.clone(),
            mul1: mul1.clone(),
            mul2: mul2.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![trans_type, reset, mul1, mul2],
                cur_ctrl: -1,
                num_ctrl: 4,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut sl = agg::ScanlineP8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut ras1: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut ras2: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        //let op = agg::SBoolOp::from_u8(self.trans_type.borrow().cur_item()).unwrap();
        let op: agg::SBoolOp =
            unsafe { std::mem::transmute(self.trans_type.borrow().cur_item() as i8) };
        ras1.set_gamma(&agg::GammaMultiply::new_with_value(
            self.mul1.borrow().value(),
        ));
        ras2.set_gamma(&agg::GammaMultiply::new_with_value(
            self.mul2.borrow().value(),
        ));

        ras.clip_box(
            0.,
            0.,
            self.util.borrow().width(),
            self.util.borrow().height(),
        );
        let mut ps1 = agg::PathStorage::new();
        generate_circles(&mut ps1, self.quad1.polygon(), 5, 20.);

        let mut ps2 = agg::PathStorage::new();
        generate_circles(&mut ps2, self.quad2.polygon(), 5, 20.);

        ras1.set_filling_rule(agg::FillingRule::FillEvenOdd);

        {
            let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

            r.set_color(agg::Rgba8::new_params(240, 255, 200, 100));
            ras1.add_path(&mut ps1, 0);
            agg::render_scanlines(&mut ras1, &mut sl, &mut r);

            r.set_color(agg::Rgba8::new_params(255, 240, 240, 100));
            ras2.add_path(&mut ps2, 0);
            agg::render_scanlines(&mut ras2, &mut sl, &mut r);
        }
        //--------------------------
        let mut sl_result = agg::ScanlineP8::new();
        let mut sl1 = agg::ScanlineP8::new();
        let mut sl2 = agg::ScanlineP8::new();
        let mut sren = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        sren.set_color(agg::Rgba8::new_params(0, 0, 0, 255));

        agg::sbool_combine_shapes_aa(
            op,
            &mut ras1,
            &mut ras2,
            &mut sl1,
            &mut sl2,
            &mut sl_result,
            &mut sren,
        );

        //--------------------------
        // Render the "quad" tools and controls
        let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
        r.set_color(agg::Rgba8::new_params(0, 75, 125, 150));
        ras.add_path(&mut self.quad1, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut r);
        ras.add_path(&mut self.quad2, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut r);
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.trans_type.borrow_mut(),
        );
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.reset.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.mul1.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.mul2.borrow_mut());
    }

    fn on_init(&mut self) {
        let width = self.util.borrow().width();
        let height = self.util.borrow().height();
        *self.quad1.xn_mut(0) = 50.;
        *self.quad1.yn_mut(0) = 200. - 20.;
        *self.quad1.xn_mut(1) = width / 2. - 25.;
        *self.quad1.yn_mut(1) = 200.;
        *self.quad1.xn_mut(2) = width / 2. - 25.;
        *self.quad1.yn_mut(2) = height - 50. - 20.;
        *self.quad1.xn_mut(3) = 50.;
        *self.quad1.yn_mut(3) = height - 50.;

        *self.quad2.xn_mut(0) = width / 2. + 25.;
        *self.quad2.yn_mut(0) = 200. - 20.;
        *self.quad2.xn_mut(1) = width - 50.;
        *self.quad2.yn_mut(1) = 200.;
        *self.quad2.xn_mut(2) = width - 50.;
        *self.quad2.yn_mut(2) = height - 50. - 20.;
        *self.quad2.xn_mut(3) = width / 2. + 25.;
        *self.quad2.yn_mut(3) = height - 50.;
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32,
    ) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.quad1.on_mouse_button_down(x as f64, y as f64)
                || self.quad2.on_mouse_button_down(x as f64, y as f64)
            {
                return Draw::Yes;
            }
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.quad1.on_mouse_move(x as f64, y as f64)
                || self.quad2.on_mouse_move(x as f64, y as f64)
            {
                return Draw::Yes;
            }
        }
        if flags & InputFlag::MouseLeft as u32 == 0 {
            return self.on_mouse_button_up(rb, x, y, flags);
        }
        Draw::No
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, _flags: u32,
    ) -> Draw {
        if self.quad1.on_mouse_button_up(x as f64, y as f64)
            || self.quad2.on_mouse_button_up(x as f64, y as f64)
        {
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {
        if self.reset.borrow().status() {
            self.on_init();
            self.reset.borrow_mut().set_status(false);
            //return Draw::Yes;
        }
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Scanline Boolean");

    if plat.init(800, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

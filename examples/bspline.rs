use agg::{Color, RasterScanLine};

use crate::ctrl::cbox::Cbox;
use crate::ctrl::polygon::SimplePolygonVertexSource;
use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::{interactive_polygon::InteractivePolygon, pixel_formats::*};

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

struct Application {
    poly: InteractivePolygon<'static>,
    num_points: Ptr<Slider<'static, agg::Rgba8>>,
    close: Ptr<Cbox<'static, agg::Rgba8>>,
    flip: i32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let num_points = ctrl_ptr(Slider::new(5.0, 5.0, 340.0, 12.0, !flip_y));
        let close = ctrl_ptr(Cbox::new(350., 5.0, "Close", !flip_y));
        num_points.borrow_mut().set_range(1.0, 40.0);
        num_points.borrow_mut().set_value(1.);
        num_points
            .borrow_mut()
            .set_label("Number of intermediate Points = %0.3f");
        Application {
            poly: InteractivePolygon::new(6, 5.0),
            num_points: num_points.clone(),
            close: close.clone(),
            flip: 0,
            ctrls: CtrlContainer {
                ctrl: vec![num_points, close],
                cur_ctrl: -1,
                num_ctrl: 2,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        if self.flip == 1 {
            *self.poly.xn_mut(0) = 100.;
            *self.poly.yn_mut(0) = self.util.borrow_mut().height() - 100.;
            *self.poly.xn_mut(1) = self.util.borrow_mut().width() - 100.;
            *self.poly.yn_mut(1) = self.util.borrow_mut().height() - 100.;
            *self.poly.xn_mut(2) = self.util.borrow_mut().width() - 100.;
            *self.poly.yn_mut(2) = 100.;
            *self.poly.xn_mut(3) = 100.;
            *self.poly.yn_mut(3) = 100.;
        } else {
            *self.poly.xn_mut(0) = 100.;
            *self.poly.yn_mut(0) = 100.;
            *self.poly.xn_mut(1) = self.util.borrow_mut().width() - 100.;
            *self.poly.yn_mut(1) = 100.;
            *self.poly.xn_mut(2) = self.util.borrow_mut().width() - 100.;
            *self.poly.yn_mut(2) = self.util.borrow_mut().height() - 100.;
            *self.poly.xn_mut(3) = 100.;
            *self.poly.yn_mut(3) = self.util.borrow_mut().height() - 100.;
        }
        *self.poly.xn_mut(4) = self.util.borrow_mut().width() / 2.;
        *self.poly.yn_mut(4) = self.util.borrow_mut().height() / 2.;
        *self.poly.xn_mut(5) = self.util.borrow_mut().width() / 2.;
        *self.poly.yn_mut(5) = self.util.borrow_mut().height() / 3.;
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&ColorType::new_params(255, 255, 255, 255));

        let mut sl = agg::ScanlineP8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let path = SimplePolygonVertexSource::new(
            self.poly.polygon().as_ptr(),
            self.poly.num_points() as u32,
            false,
            self.close.borrow().status(),
        );

        let mut bspline = agg::ConvBspline::new_owned(path);
        bspline.set_interpolation_step(1.0 / self.num_points.borrow().value());

        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(bspline);
        stroke.set_width(2.0);

        ras.add_path(&mut stroke, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut rb,
            &ColorType::new_params(0, 0, 0, 255),
        );

        //--------------------------
        // Render the "poly" tool and controls
        ras.add_path(&mut self.poly, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut rb,
            &ColorType::new_from_rgba(&agg::Rgba::new_params(0., 0.3, 0.5, 0.6)),
        );

        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.close.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.num_points.borrow_mut(),
        );
        //--------------------------
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32,
    ) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            return Draw::from(self.poly.on_mouse_button_down(x as f64, y as f64));
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            return Draw::from(self.poly.on_mouse_move(x as f64, y as f64));
        }
        if flags & InputFlag::MouseLeft as u32 == 0 {
            return self.on_mouse_button_up(rb, x, y, flags);
        }
        Draw::No
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, _flags: u32,
    ) -> Draw {
        return Draw::from(self.poly.on_mouse_button_up(x as f64, y as f64));
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        if key == ' ' as u32 {
            self.flip ^= 1;
            self.on_init();
            return Draw::Yes;
        }
        Draw::No
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    //plat.app_mut().init();
    plat.set_caption("AGG Example. BSpline Interpolator");

    if plat.init(600, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

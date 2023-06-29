use crate::platform::*;
use agg::rendering_buffer::RenderBuf;
use agg::{Color, GammaFn, RasterScanLine};

mod ctrl;
mod platform;

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
    thickness: Ptr<Slider<'static, agg::Rgba8>>,
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    contrast: Ptr<Slider<'static, agg::Rgba8>>,
    rx: f64,
    ry: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let thickness = ctrl_ptr(Slider::new(5., 5., 400. - 5., 11., !flip_y));
        let gamma = ctrl_ptr(Slider::new(5., 5. + 15., 400. - 5., 11. + 15., !flip_y));
        let contrast = ctrl_ptr(Slider::new(5., 5. + 30., 400. - 5., 11. + 30., !flip_y));

        thickness.borrow_mut().set_label("Thickness=%3.2f");
        gamma.borrow_mut().set_label("Gamma=%3.2f");
        contrast.borrow_mut().set_label("Contrast=%3.2f");

        thickness.borrow_mut().set_range(0.0, 3.0);
        gamma.borrow_mut().set_range(0.5, 3.0);
        contrast.borrow_mut().set_range(0.0, 1.0);

        thickness.borrow_mut().set_value(1.0);
        gamma.borrow_mut().set_value(1.0);
        contrast.borrow_mut().set_value(1.0);

        Application {
            thickness: thickness.clone(),
            gamma: gamma.clone(),
            contrast: contrast.clone(),
            rx: 0.0,
            ry: 0.0,
            ctrls: CtrlContainer {
                ctrl: vec![thickness, gamma, contrast],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        self.rx = self.util.borrow().width() / 3.0;
        self.ry = self.util.borrow().height() / 3.0;
    }

    fn on_draw(&mut self, rb: &mut agg::RenderBuf) {
        let g = self.gamma.borrow().value();
        let gamma = agg::GammaLut::new_with_gamma(g);
        let mut pixf = PixfmtGamma::new_borrowed(rb);
		pixf.blender_mut().set_gamma_owned(gamma);
        let mut renb = agg::RendererBase::new_borrowed(&mut pixf);
        renb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let dark = 1.0 - self.contrast.borrow().value();
        let light = self.contrast.borrow().value();
        let width = self.util.borrow().width();
        let height = self.util.borrow().height();
        renb.copy_bar(
            0,
            0,
            (width as i32 / 2) as i32,
            height as i32,
            &ColorType::new_from_rgba(&agg::Rgba::new_params(dark, dark, dark, 1.0)),
        );
        renb.copy_bar(
            (width as i32 / 2 + 1) as i32,
            0,
            width as i32,
            height as i32,
            &ColorType::new_from_rgba(&agg::Rgba::new_params(light, light, light, 1.0)),
        );
        renb.copy_bar(
            0,
            (height as i32 / 2 + 1) as i32,
            width as i32,
            height as i32,
            &ColorType::new_from_rgba(&agg::Rgba::new_params(1.0, dark, dark, 1.0)),
        );

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineU8::new();
        let mut path = agg::PathStorage::new();

        let x = (width - 256.0) / 2.0;
        let y = 50.0;
        path.remove_all();
        let gp = agg::GammaPower::new_with_gamma(g);
        for i in 0..256 {
            let v = i as f64 / 255.0;
            let gval = gp.call(v);
            let dy = gval * 255.0;
            if i == 0 {
                path.move_to(x + i as f64, y + dy);
            } else {
                path.line_to(x + i as f64, y + dy);
            }
        }
        let mut gpoly: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(path);
        gpoly.set_width(2.0);
        ras.reset();
        ras.add_path(&mut gpoly, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(80, 127, 80, 255),
        );

        let ell = agg::Ellipse::new_ellipse(width / 2., height / 2., self.rx, self.ry, 150, false);
        let mut poly: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(ell);
        poly.set_width(self.thickness.borrow().value());
        ras.reset();
        ras.add_path(&mut poly, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(255, 0, 0, 255),
        );

        poly.source_mut().init(
            width / 2.,
            height / 2.,
            self.rx - 5.0,
            self.ry - 5.0,
            150,
false,
        );
        ras.reset();
        ras.add_path(&mut poly, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(0, 255, 0, 255),
        );

        poly.source_mut().init(
            width / 2.,
            height / 2.,
            self.rx - 10.0,
            self.ry - 10.0,
            150,
false,
        );
        ras.reset();
        ras.add_path(&mut poly, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(0, 0, 255, 255),
        );

        poly.source_mut().init(
            width / 2.,
            height / 2.,
            self.rx - 15.0,
            self.ry - 15.0,
            150,
false,
        );
        ras.reset();
        ras.add_path(&mut poly, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(0, 0, 0, 255),
        );

        poly.source_mut().init(
            width / 2.,
            height / 2.,
            self.rx - 20.0,
            self.ry - 20.0,
            150,
false,
        );
        ras.reset();
        ras.add_path(&mut poly, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(255, 255, 255, 255),
        );

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut renb,
            &mut *self.thickness.borrow_mut(),
        );
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.gamma.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut renb,
            &mut *self.contrast.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw{
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.rx = f64::abs(self.util.borrow().width() / 2.0 - x as f64);
            self.ry = f64::abs(self.util.borrow().height() / 2.0 - y as f64);
            return Draw::Yes;
        }
Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        self.on_mouse_button_down(rb, x, y, flags)
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Thin red ellipse");

    if plat.init(400, 320, WindowFlag::Resize as u32) {
        plat.run();
    }
}

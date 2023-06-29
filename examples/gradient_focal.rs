use crate::platform::*;

use agg::color_rgba::*;
use agg::RenderBuf;
use agg::{Gamma, RasterScanLine};

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
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    scanline: agg::ScanlineU8,
    rasterizer: agg::RasterizerScanlineAa,
    alloc: agg::VecSpan<agg::Rgba8>,
    gradient_lut: agg::GradientLut<agg::ColorIp<agg::Rgba8>, 1024>,
    gamma_lut: agg::GammaLut,
    mouse_x: f64,
    mouse_y: f64,
    old_gamma: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}
impl Application {
    fn build_gradient_lut(&mut self) {
        self.gradient_lut.remove_all();

        self.gradient_lut.add_color(
            0.0,
            rgba8_gamma_dir(agg::Rgba8::new_params(0, 255, 0, 255), &self.gamma_lut),
        );
        self.gradient_lut.add_color(
            0.2,
            rgba8_gamma_dir(agg::Rgba8::new_params(120, 0, 0, 255), &self.gamma_lut),
        );
        self.gradient_lut.add_color(
            0.7,
            rgba8_gamma_dir(agg::Rgba8::new_params(120, 120, 0, 255), &self.gamma_lut),
        );
        self.gradient_lut.add_color(
            1.0,
            rgba8_gamma_dir(agg::Rgba8::new_params(0, 0, 255, 255), &self.gamma_lut),
        );

        self.gradient_lut.build_lut();
    }
}
impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let gamma = ctrl_ptr(Slider::new(5.0, 5.0, 340.0, 12.0, !flip_y));
        gamma.borrow_mut().set_range(0.5, 2.5);
        gamma.borrow_mut().set_value(1.8);
        gamma.borrow_mut().set_label("Gamma = %.3f");

        gamma.borrow_mut().no_transform();

        Application {
            gamma: gamma.clone(),
            scanline: agg::ScanlineU8::new(),
            rasterizer: agg::RasterizerScanlineAa::new(),
            alloc: agg::VecSpan::new(),
            gradient_lut: agg::GradientLut::new(),
            gamma_lut: agg::GammaLut::new(),
            mouse_x: 200.0,
            mouse_y: 200.0,
            old_gamma: 0.0,
            ctrls: CtrlContainer {
                ctrl: vec![gamma],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        self.gamma_lut.set_gamma(self.gamma.borrow().value());
        self.old_gamma = self.gamma.borrow().value();

        self.build_gradient_lut();
        self.mouse_y = self.util.borrow().initial_height() / 2.;
        self.mouse_x = self.util.borrow().initial_width() / 2.;
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pixf);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        //let rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        // When Gamma changes rebuild the gamma and gradient LUTs
        //------------------
        if self.old_gamma != self.gamma.borrow().value() {
            self.gamma_lut.set_gamma(self.gamma.borrow().value());
            self.build_gradient_lut();
            self.old_gamma = self.gamma.borrow().value();
        }

        // Gradient center. All gradient functions assume the
        // center being in the origin (0,0) and you can't
        // change it. But you can apply arbitrary transformations
        // to the gradient (see below).
        //------------------
        let cx = self.util.borrow_mut().initial_width() / 2.;
        let cy = self.util.borrow_mut().initial_height() / 2.;
        let r = 100.0;

        // Focal center. Defined in the gradient coordinates,
        // that is, with respect to the origin (0,0)
        //------------------
        let fx = self.mouse_x - cx;
        let fy = self.mouse_y - cy;

        let gradient_func = agg::GradientRadialFocus::new_with_params(r, fx, fy);
        let mut gradient_adaptor = agg::GradientReflectAdaptor::new(gradient_func);
        let mut gradient_mtx = agg::TransAffine::new_default();

        // Making the affine matrix. Move to (cx,cy),
        // apply the resizing transformations and invert
        // the matrix. Gradients and images always assume the
        // inverse transformations.
        //------------------
        gradient_mtx.translate(&cx, &cy);
        gradient_mtx *= *self.util.borrow_mut().trans_affine_resizing();
        gradient_mtx.invert();

        let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(gradient_mtx);
        let mut span_gradient = agg::SpanGradient::new(
            &mut interpolator,
            &mut gradient_adaptor,
            &mut self.gradient_lut,
            0.,
            r,
        );

        // Form the simple rectangle
        //------------------
        self.rasterizer.reset();
        self.rasterizer.move_to_d(0.0, 0.0);
        self.rasterizer.line_to_d(self.util.borrow().width(), 0.0);
        self.rasterizer
            .line_to_d(self.util.borrow().width(), self.util.borrow().height());
        self.rasterizer.line_to_d(0.0, self.util.borrow().height());

        // Render the gradient to the whole screen and measure the time
        //------------------
        self.util.borrow_mut().start_timer();
        agg::render_scanlines_aa(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut self.alloc,
            &mut span_gradient,
        );
        let tm = self.util.borrow_mut().elapsed_time();

        // Draw the transformed circle that shows the gradient boundary
        //------------------
        let e = agg::Ellipse::new_ellipse(cx, cy, r, r, 0, false);
        let estr: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(e);
        let mut etrans =
            agg::ConvTransform::new_owned(estr, *self.util.borrow_mut().trans_affine_resizing());

        self.rasterizer.add_path(&mut etrans, 0);
        agg::render_scanlines_aa_solid(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &agg::Rgba8::new_params(255, 255, 255, 255),
        );

        // Show the gradient time
        //------------------
        let mut t = agg::GsvText::new();
        t.set_size(10.0, 0.);
        let buf = format!("{:.2} ms", tm);
        t.set_start_point(10.0, 35.0);
        t.set_text(&buf);
        let mut pt: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(t);
        pt.set_width(1.5);

        self.rasterizer.add_path(&mut pt, 0);
        agg::render_scanlines_aa_solid(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &agg::Rgba8::new_params(0, 0, 0, 255),
        );

        // Show the controls
        //------------------
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.gamma.borrow_mut(),
        );

        // Apply the inverse gamma to the whole buffer
        // (transform the colors to the perceptually uniform space)
        //------------------
        pixf.apply_gamma_inv(&self.gamma_lut);
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.mouse_x = x as f64;
            self.mouse_y = y as f64;
            self.util
                .borrow_mut()
                .trans_affine_resizing()
                .inverse_transform(&mut self.mouse_x, &mut self.mouse_y);
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.mouse_x = x as f64;
            self.mouse_y = y as f64;
            self.util
                .borrow_mut()
                .trans_affine_resizing()
                .inverse_transform(&mut self.mouse_x, &mut self.mouse_y);
            return Draw::Yes;
        }
        Draw::No
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. PDF linear and radial gradients");

    if plat.init(600, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

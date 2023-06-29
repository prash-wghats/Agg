use crate::ctrl::cbox::Cbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;
use agg::{Color, Gamma, PixFmt, RasterScanLine, RasterStyle, RenderBuf, Renderer};

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

struct StyleHandler<'a> {
    transparent: agg::Rgba8,
    styles: &'a [agg::Rgba8],
    count: u32,
}

impl<'a> StyleHandler<'a> {
    pub fn new(styles: &'a [agg::Rgba8], count: u32) -> Self {
        Self {
            transparent: agg::Rgba8::new_params(0, 0, 0, 0),
            styles,
            count,
        }
    }
}

impl<'a> RasterStyle<agg::Rgba8> for StyleHandler<'a> {
    fn is_solid(&self, _style: u32) -> bool {
        true
    }

    fn color(&self, style: u32) -> &agg::Rgba8 {
        if style < self.count {
            &self.styles[style as usize]
        } else {
            &self.transparent
        }
    }

    fn generate_span(
        &mut self, _span: &mut [agg::Rgba8], _x: i32, _y: i32, _len: u32, _style: u32,
    ) {
        // TODO
    }
}

struct Application {
    width: Ptr<Slider<'static, agg::Rgba8>>,
    alpha1: Ptr<Slider<'static, agg::Rgba8>>,
    alpha2: Ptr<Slider<'static, agg::Rgba8>>,
    alpha3: Ptr<Slider<'static, agg::Rgba8>>,
    alpha4: Ptr<Slider<'static, agg::Rgba8>>,
    invert_order: Ptr<Cbox<'static, agg::Rgba8>>,
    path: agg::PathStorage,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    pub fn compose_path(&mut self) {
        self.path.remove_all();
        self.path.move_to(28.47, 6.45);
        self.path.curve3_ctrl(21.58, 1.12, 19.82, 0.29);
        self.path.curve3_ctrl(17.19, -0.93, 14.21, -0.93);
        self.path.curve3_ctrl(9.57, -0.93, 6.57, 2.25);
        self.path.curve3_ctrl(3.56, 5.42, 3.56, 10.60);
        self.path.curve3_ctrl(3.56, 13.87, 5.03, 16.26);
        self.path.curve3_ctrl(7.03, 19.58, 11.99, 22.51);
        self.path.curve3_ctrl(16.94, 25.44, 28.47, 29.64);
        self.path.line_to(28.47, 31.40);
        self.path.curve3_ctrl(28.47, 38.09, 26.34, 40.58);
        self.path.curve3_ctrl(24.22, 43.07, 20.17, 43.07);
        self.path.curve3_ctrl(17.09, 43.07, 15.28, 41.41);
        self.path.curve3_ctrl(13.43, 39.75, 13.43, 37.60);
        self.path.line_to(13.53, 34.77);
        self.path.curve3_ctrl(13.53, 32.52, 12.38, 31.30);
        self.path.curve3_ctrl(11.23, 30.08, 9.38, 30.08);
        self.path.curve3_ctrl(7.57, 30.08, 6.42, 31.35);
        self.path.curve3_ctrl(5.27, 32.62, 5.27, 34.81);
        self.path.curve3_ctrl(5.27, 39.01, 9.57, 42.53);
        self.path.curve3_ctrl(13.87, 46.04, 21.63, 46.04);
        self.path.curve3_ctrl(27.59, 46.04, 31.40, 44.04);
        self.path.curve3_ctrl(34.28, 42.53, 35.64, 39.31);
        self.path.curve3_ctrl(36.52, 37.21, 36.52, 30.71);
        self.path.line_to(36.52, 15.53);
        self.path.curve3_ctrl(36.52, 9.13, 36.77, 7.69);
        self.path.curve3_ctrl(37.01, 6.25, 37.57, 5.76);
        self.path.curve3_ctrl(38.13, 5.27, 38.87, 5.27);
        self.path.curve3_ctrl(39.65, 5.27, 40.23, 5.62);
        self.path.curve3_ctrl(41.26, 6.25, 44.19, 9.18);
        self.path.line_to(44.19, 6.45);
        self.path.curve3_ctrl(38.72, -0.88, 33.74, -0.88);
        self.path.curve3_ctrl(31.35, -0.88, 29.93, 0.78);
        self.path.curve3_ctrl(28.52, 2.44, 28.47, 6.45);
        self.path.close_polygon(0);

        self.path.move_to(28.47, 9.62);
        self.path.line_to(28.47, 26.66);
        self.path.curve3_ctrl(21.09, 23.73, 18.95, 22.51);
        self.path.curve3_ctrl(15.09, 20.36, 13.43, 18.02);
        self.path.curve3_ctrl(11.77, 15.67, 11.77, 12.89);
        self.path.curve3_ctrl(11.77, 9.38, 13.87, 7.06);
        self.path.curve3_ctrl(15.97, 4.74, 18.70, 4.74);
        self.path.curve3_ctrl(22.41, 4.74, 28.47, 9.62);
        self.path.close_polygon(0);
    }
}
impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let width = ctrl_ptr(Slider::new(180.0 + 10.0, 5.0, 130.0 + 300.0, 12.0, !flip_y));
        let alpha1 = ctrl_ptr(Slider::new(5.0, 5.0, 180.0, 12.0, !flip_y));
        let alpha2 = ctrl_ptr(Slider::new(5.0, 25.0, 180.0, 32.0, !flip_y));
        let alpha3 = ctrl_ptr(Slider::new(5.0, 45.0, 180.0, 52.0, !flip_y));
        let alpha4 = ctrl_ptr(Slider::new(5.0, 65.0, 180.0, 72.0, !flip_y));
        let invert_order = ctrl_ptr(Cbox::new(190.0, 25.0, "Invert Z-Order", !flip_y));
        width.borrow_mut().set_range(-20.0, 50.0);
        width.borrow_mut().set_value(10.0);
        width.borrow_mut().set_label("Width=%1.2f");

        alpha1.borrow_mut().set_range(0., 1.);
        alpha1.borrow_mut().set_value(1.);
        alpha1.borrow_mut().set_label("Alpha1=%1.3f");

        alpha2.borrow_mut().set_range(0., 1.);
        alpha2.borrow_mut().set_value(1.);
        alpha2.borrow_mut().set_label("Alpha2=%1.3f");

        alpha3.borrow_mut().set_range(0., 1.);
        alpha3.borrow_mut().set_value(1.);
        alpha3.borrow_mut().set_label("Alpha3=%1.3f");

        alpha4.borrow_mut().set_range(0., 1.);
        alpha4.borrow_mut().set_value(1.);
        alpha4.borrow_mut().set_label("Alpha4=%1.3f");

        Self {
            width: width.clone(),
            alpha1: alpha1.clone(),
            alpha2: alpha2.clone(),
            alpha3: alpha3.clone(),
            alpha4: alpha4.clone(),
            invert_order: invert_order.clone(),
            path: agg::PathStorage::new(),
            ctrls: CtrlContainer {
                ctrl: vec![width, alpha1, alpha2, alpha3, alpha4, invert_order],
                cur_ctrl: -1,
                num_ctrl: 6,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let lut: agg::GammaLut = agg::GammaLut::new_with_gamma(2.0);
        let mut pixf = agg::PixBgra32::new_owned(rbuf.clone());
        let mut renb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut pixf_pre = agg::PixBgra32Pre::new_borrowed(rbuf);
        let mut renb_pre = agg::RendererBase::new_borrowed(&mut pixf_pre);

        // Clear the window with a gradient
        let mut gr = vec![];
        for i in 0..renb.ren().width() {
            gr.push(agg::Rgba8::new_params(255, 255, 0, 255).gradient(
                &agg::Rgba8::new_params(0, 255, 255, 255),
                i as f64 / renb.ren().width() as f64,
            ));
        }
        for i in 0..renb.ren().height() {
            renb.copy_color_hspan(0, i as i32, renb.ren().width() as i32, &gr[..]);
        }
        renb.ren_mut().apply_gamma_dir(&lut);

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut rasc = agg::RasterizerCompoundAa::<agg::RasterizerSlClipDbl>::new();
        let mut sl = agg::ScanlineU8::new();
        let mut alloc = agg::VecSpan::new();

        // Draw two triangles
        ras.move_to_d(0.0, 0.0);
        ras.line_to_d(self.util.borrow().width(), 0.0);
        ras.line_to_d(self.util.borrow().width(), self.util.borrow().height());
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(
                lut.dir(0) as u32,
                lut.dir(100) as u32,
                lut.dir(0) as u32,
                255,
            ),
        );

        ras.move_to_d(0.0, 0.0);
        ras.line_to_d(0.0, self.util.borrow().height());
        ras.line_to_d(self.util.borrow().width(), 0.0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut renb,
            &agg::Rgba8::new_params(
                lut.dir(0) as u32,
                lut.dir(100) as u32,
                lut.dir(100) as u32,
                255,
            ),
        );

        let mut mtx = agg::TransAffine::new_default();
        mtx *= agg::TransAffine::trans_affine_scaling_eq(4.0);
        mtx *= agg::TransAffine::trans_affine_translation(150.0, 100.0);
        self.compose_path();

        let trans = agg::ConvTransform::new_borrowed(&mut self.path, mtx);
        let curve: agg::ConvCurve<'_, _> = agg::ConvCurve::new_owned(trans);

        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(curve);

        let mut styles = [agg::Rgba8::new_params(0, 0, 0, 0); 4];

        if self.invert_order.borrow().status() {
            rasc.set_layer_order(agg::LayerOrder::Inverse);
        } else {
            rasc.set_layer_order(agg::LayerOrder::Direct);
        }

        styles[3] = *agg::Rgba8::new_params(
            lut.dir(255) as u32,
            lut.dir(0) as u32,
            lut.dir(108) as u32,
            200,
        )
        .premultiply();

        styles[2] = *agg::Rgba8::new_params(
            lut.dir(51) as u32,
            lut.dir(0) as u32,
            lut.dir(151) as u32,
            180,
        )
        .premultiply();

        styles[1] = *agg::Rgba8::new_params(
            lut.dir(143) as u32,
            lut.dir(90) as u32,
            lut.dir(6) as u32,
            200,
        )
        .premultiply();

        styles[0] = *agg::Rgba8::new_params(
            lut.dir(0) as u32,
            lut.dir(0) as u32,
            lut.dir(255) as u32,
            220 as u32,
        )
        .premultiply();

        let mut sh = StyleHandler::new(&styles[..], 4);

        stroke.set_width(self.width.borrow().value());

        rasc.reset();
        rasc.set_master_alpha(3, self.alpha1.borrow().value());
        rasc.set_master_alpha(2, self.alpha2.borrow().value());
        rasc.set_master_alpha(1, self.alpha3.borrow().value());
        rasc.set_master_alpha(0, self.alpha4.borrow().value());

        let ell = agg::Ellipse::new_ellipse(220.0, 180.0, 120.0, 10.0, 128, false);
        let mut str_ell: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(ell);
        str_ell.set_width(self.width.borrow().value() / 2.);

        rasc.set_styles(3, -1);
        rasc.add_path(&mut str_ell, 0);

        rasc.set_styles(2, -1);
        rasc.add_path(str_ell.source_mut(), 0);

        rasc.set_styles(1, -1);
        rasc.add_path(&mut stroke, 0);

        rasc.set_styles(0, -1);
        rasc.add_path(stroke.source_mut(), 0);

        agg::render_scanlines_compound_layered(
            &mut rasc,
            &mut sl,
            &mut renb_pre,
            &mut alloc,
            &mut sh,
        );
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.width.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.alpha1.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.alpha2.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.alpha3.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.alpha4.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut renb,
            &mut *self.invert_order.borrow_mut(),
        );

        pixf.apply_gamma_inv(&lut);
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);
    plat.set_caption("AGG Example. Compound Rasterizer -- Geometry Flattening");

    if plat.init(440, 330, WindowFlag::Resize as u32) {
        plat.run();
    }
}

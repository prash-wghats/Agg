use crate::platform::*;

use agg::{Color, RasterScanLine, RenderBuf, RenderBuffer};

mod ctrl;
mod platform;

use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;

use core::f64::consts::PI;
use std::cell::RefCell;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

struct Application {
    radius: Ptr<Slider<'static, agg::Rgba8>>,
    step: Ptr<Slider<'static, agg::Rgba8>>,
    filters: Ptr<Rbox<'static, agg::Rgba8>>,
    normalize: Ptr<Cbox<'static, agg::Rgba8>>,
    run: Ptr<Cbox<'static, agg::Rgba8>>,
    single_step: Ptr<Cbox<'static, agg::Rgba8>>,
    refresh: Ptr<Cbox<'static, agg::Rgba8>>,
    cur_angle: f64,
    cur_filter: i32,
    num_steps: i32,
    num_pix: f64,
    time1: f64,
    time2: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn transform_image(&mut self, angle: f64) {
        let width = self.util.borrow_mut().rbuf_img(0).width() as f64;
        let height = self.util.borrow_mut().rbuf_img(0).height() as f64;

        let mut pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        let mut pixf_pre =
            agg::PixBgr24Pre::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut rb_pre = agg::RendererBase::new_borrowed(&mut pixf_pre);
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut sa = agg::VecSpan::new();

        let mut src_mtx = agg::TransAffine::new_default();
        src_mtx *= agg::TransAffine::trans_affine_translation(-width / 2.0, -height / 2.0);
        src_mtx *= agg::TransAffine::trans_affine_rotation(angle * PI / 180.0);
        src_mtx *= agg::TransAffine::trans_affine_translation(width / 2.0, height / 2.0);

        let mut img_mtx = src_mtx;
        img_mtx.invert();

        let mut r = width;
        if height < r {
            r = height;
        }

        r *= 0.5;
        r -= 4.0;
        let ell = agg::Ellipse::new_ellipse(width / 2.0, height / 2.0, r, r, 200, false);
        let mut tr = agg::ConvTransform::new_owned(ell, src_mtx);

        self.num_pix += r * r * PI;

        let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(img_mtx);

        let mut filter = agg::ImageFilterLut::new();
        let norm = self.normalize.borrow().status();

        let mut pixf_img = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(1).clone());

        match self.filters.borrow().cur_item() {
            0 => {
                let mut source = agg::ImageAccessorClip::new(
                    pixf_img,
                    &agg::Rgba8::new_from_rgba(&agg::color_rgba::rgba_pre(0., 0., 0., 0.)),
                );
                let mut sg = agg::SpanImageFilterRgbNn::new(&mut source, &mut interpolator);
                ras.add_path(&mut tr, 0);
                agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb_pre, &mut sa, &mut sg);
            }
            1 => {
                let mut sg = agg::SpanImageFilterRgbBilinearClip::new(
                    &mut pixf_img,
                    &mut interpolator,
                    agg::Rgba8::new_from_rgba(&agg::color_rgba::rgba_pre(0., 0., 0., 0.)),
                );
                ras.add_path(&mut tr, 0);
                agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb_pre, &mut sa, &mut sg);
            }
            5 | 6 | 7 => {
                match self.filters.borrow().cur_item() {
                    5 => filter.calculate(&agg::ImageFilterHanning::new(), norm),
                    6 => filter.calculate(&agg::ImageFilterHamming::new(), norm),
                    7 => filter.calculate(&agg::ImageFilterHermite::new(), norm),
                    _ => {}
                }
                let mut source = agg::ImageAccessorClip::new(
                    pixf_img,
                    &agg::Rgba8::new_from_rgba(&agg::color_rgba::rgba_pre(0., 0., 0., 0.)),
                );
                let mut sg =
                    agg::SpanImageFilterRgb2x2::new(&mut source, &mut interpolator, filter);
                ras.add_path(&mut tr, 0);
                agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb_pre, &mut sa, &mut sg);
            }
            2 | 3 | 4 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 => {
                match self.filters.borrow().cur_item() {
                    2 => filter.calculate(&agg::ImageFilterBicubic::new(), norm),
                    3 => filter.calculate(&agg::ImageFilterSpline16::new(), norm),
                    4 => filter.calculate(&agg::ImageFilterSpline36::new(), norm),
                    8 => filter.calculate(&agg::ImageFilterKaiser::new(), norm),
                    9 => filter.calculate(&agg::ImageFilterQuadric::new(), norm),
                    10 => filter.calculate(&agg::ImageFilterCatrom::new(), norm),
                    11 => filter.calculate(&agg::ImageFilterGaussian::new(), norm),
                    12 => filter.calculate(&agg::ImageFilterBessel::new(), norm),
                    13 => filter.calculate(&agg::ImageFilterMitchell::new(), norm),
                    14 => filter.calculate(
                        &agg::ImageFilterSinc::new_parms(self.radius.borrow().value()),
                        norm,
                    ),
                    15 => filter.calculate(
                        &agg::ImageFilterLanczos::new_parms(self.radius.borrow().value()),
                        norm,
                    ),
                    16 => filter.calculate(
                        &agg::ImageFilterBlackman::new_parms(self.radius.borrow().value()),
                        norm,
                    ),
                    _ => {}
                }
                let mut source = agg::ImageAccessorClip::new(
                    pixf_img,
                    &agg::Rgba8::new_from_rgba(&agg::color_rgba::rgba_pre(0., 0., 0., 0.)),
                );
                let mut sg = agg::SpanImageFilterRgb::new(&mut source, &mut interpolator, filter);
                ras.add_path(&mut tr, 0);
                agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb_pre, &mut sa, &mut sg);
            }
            _ => {}
        }
    }
}
impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let step = ctrl_ptr(Slider::new(115., 5., 400., 11., !flip_y));
        let radius = ctrl_ptr(Slider::new(115., 5. + 15., 400., 11. + 15., !flip_y));
        let filters = ctrl_ptr(Rbox::new(0.0, 0.0, 110.0, 210.0, !flip_y));
        let normalize = ctrl_ptr(Cbox::new(8.0, 215.0, "Normalize Filter", !flip_y));
        let run = ctrl_ptr(Cbox::new(8.0, 245.0, "RUN Test!", !flip_y));
        let single_step = ctrl_ptr(Cbox::new(8.0, 230.0, "Single Step", !flip_y));
        let refresh = ctrl_ptr(Cbox::new(8.0, 265.0, "Refresh", !flip_y));

        run.borrow_mut().set_text_size(7.5, 0.);
        single_step.borrow_mut().set_text_size(7.5, 0.);
        normalize.borrow_mut().set_text_size(7.5, 0.);
        refresh.borrow_mut().set_text_size(7.5, 0.);
        normalize.borrow_mut().set_status(true);

        radius.borrow_mut().set_label("Filter Radius=%0.3f");
        step.borrow_mut().set_label("Step=%3.2f");
        radius.borrow_mut().set_range(2.0, 8.0);
        radius.borrow_mut().set_value(4.0);
        step.borrow_mut().set_range(1.0, 10.0);
        step.borrow_mut().set_value(5.0);

        filters.borrow_mut().add_item("simple (NN)");
        filters.borrow_mut().add_item("bilinear");
        filters.borrow_mut().add_item("bicubic");
        filters.borrow_mut().add_item("spline16");
        filters.borrow_mut().add_item("spline36");
        filters.borrow_mut().add_item("hanning");
        filters.borrow_mut().add_item("hamming");
        filters.borrow_mut().add_item("hermite");
        filters.borrow_mut().add_item("kaiser");
        filters.borrow_mut().add_item("quadric");
        filters.borrow_mut().add_item("catrom");
        filters.borrow_mut().add_item("gaussian");
        filters.borrow_mut().add_item("bessel");
        filters.borrow_mut().add_item("mitchell");
        filters.borrow_mut().add_item("sinc");
        filters.borrow_mut().add_item("lanczos");
        filters.borrow_mut().add_item("blackman");
        filters.borrow_mut().set_cur_item(1);

        filters.borrow_mut().set_border_width(0., 0.);
        filters
            .borrow_mut()
            .set_background_color(agg::Rgba8::new_params(0, 0, 0, 25));
        filters.borrow_mut().set_text_size(6.0, 0.);
        filters.borrow_mut().set_text_thickness(0.85);

        Application {
            radius: radius.clone(),
            step: step.clone(),
            filters: filters.clone(),
            normalize: normalize.clone(),
            run: run.clone(),
            single_step: single_step.clone(),
            refresh: refresh.clone(),

            cur_angle: 0.0,
            cur_filter: 1,
            num_steps: 0,
            num_pix: 0.0,
            time1: 0.0,
            time2: 0.0,
            ctrls: CtrlContainer {
                ctrl: vec![radius, step, filters, normalize, run, single_step, refresh],
                cur_ctrl: -1,
                num_ctrl: 7,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        //let mut sl = agg::ScanlineU8::new();
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        rb.copy_from(self.util.borrow_mut().rbuf_img(0), None, 110, 35);

        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let buf = format!("NSteps={}", self.num_steps);
        let mut t = agg::GsvText::new();
        t.set_start_point(10.0, 295.0);
        t.set_size(10.0, 0.);
        t.set_text(&buf);

        let mut pt: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(t);
        pt.set_width(1.5);

        ras.add_path(&mut pt, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut rb,
            &agg::Rgba8::new_params(0, 0, 0, 255),
        );

        if self.time1 != self.time2 && self.num_pix > 0.0 {
            let buf = format!("{:.2} Kpix/sec", self.num_pix / (self.time2 - self.time1));
            pt.source_mut().set_start_point(10.0, 310.0);
            pt.source_mut().set_text(&buf);
            ras.add_path(&mut pt, 0);
            agg::render_scanlines_aa_solid(
                &mut ras,
                &mut sl,
                &mut rb,
                &agg::Rgba8::new_params(0, 0, 0, 255),
            );
        }

        if self.filters.borrow_mut().cur_item() >= 14 {
            ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.radius.borrow_mut());
        }
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.step.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.filters.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.run.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.normalize.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.single_step.borrow_mut(),
        );
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.refresh.borrow_mut());
    }

    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {
        if self.single_step.borrow().status() {
            self.cur_angle += self.step.borrow().value();
            self.util.borrow_mut().copy_img_to_img(1, 0);
            let t = self.step.borrow().value();
            self.transform_image(t);
            self.num_steps += 1;

            self.single_step.borrow_mut().set_status(false);
            //return Draw::Yes;
        }

        if self.run.borrow().status() {
            self.util.borrow_mut().start_timer();
            self.time1 = self.util.borrow_mut().elapsed_time();
            self.time2 = self.util.borrow_mut().elapsed_time();
            self.num_pix = 0.0;
            self.util.borrow_mut().set_wait_mode(false);
        }
        if self.refresh.borrow().status() || self.filters.borrow().cur_item() != self.cur_filter {
            self.util.borrow_mut().start_timer();
            self.time1 = 0.;
            self.time2 = 0.;
            self.num_pix = 0.0;
            self.cur_angle = 0.0;
            self.util.borrow_mut().copy_img_to_img(1, 2);
            self.transform_image(0.0);
            self.refresh.borrow_mut().set_status(false);
            self.cur_filter = self.filters.borrow().cur_item();
            self.num_steps = 0;
            //return Draw::Yes;
        }
    }

    fn on_idle(&mut self) -> Draw {
        if self.run.borrow().status() {
            if self.cur_angle < 360.0 {
                self.cur_angle += self.step.borrow().value();
                self.util.borrow_mut().copy_img_to_img(1, 0);
                self.util.borrow_mut().start_timer();
                let t = self.step.borrow().value();
                self.transform_image(t);
                self.time2 += self.util.borrow_mut().elapsed_time();
                self.num_steps += 1;
            } else {
                self.cur_angle = 0.0;
                //self.time2 = clock();
                self.util.borrow_mut().set_wait_mode(true);
                self.run.borrow_mut().set_status(false);
            }
            return Draw::Yes;
        } else {
            self.util.borrow_mut().set_wait_mode(true);
        }
        Draw::No
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "spheres";
	
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
	let ext = plat.app().util.borrow().img_ext().to_string();
    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("spheres") {
            buf = format!(
                "File not found: {}{}. Download http://www.antigrain.com/{}{}
				or copy it from another directory if available.",
                img_name, ext, img_name, ext
            );
        } else {
            buf = format!("File not found: {}{}", img_name, ext);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
        return;
    }

    plat.set_caption("Image transformation filters comparison");

    let mut w = plat.app_mut().util.borrow_mut().rbuf_img(0).width() + 110;
    let mut h = plat.app_mut().util.borrow_mut().rbuf_img(0).height() + 40;

    if w < 305 {
        w = 305;
    }
    if h < 325 {
        h = 325;
    }

    if plat.init(w, h, WindowFlag::Resize as u32) {
        plat.app_mut().util.borrow_mut().copy_img_to_img(1, 0);
        plat.app_mut().util.borrow_mut().copy_img_to_img(2, 0);
        plat.app_mut().transform_image(0.0);
        plat.run();
    }
}

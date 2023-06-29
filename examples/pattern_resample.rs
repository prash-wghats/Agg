use crate::platform::*;

use agg::{RasterScanLine, RenderBuf, RendererScanlineColor, Transformer};

mod ctrl;
mod platform;

use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

mod misc;
use misc::interactive_polygon::InteractivePolygon;

struct Application {
    gamma_lut: agg::GammaLut,
    quad: InteractivePolygon<'static>,
    trans_type: Ptr<Rbox<'static, agg::Rgba8>>,
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    blur: Ptr<Slider<'static, agg::Rgba8>>,
    old_gamma: f64,
    scanline: agg::ScanlineU8,
    rasterizer: agg::RasterizerScanlineAa,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }
    // Constructor
    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let gamma_lut = agg::GammaLut::new_with_gamma(2.0);
        let quad = InteractivePolygon::new(4, 5.0);
        let mut trans_type = Rbox::new(400.0, 5.0, 430.0 + 170.0, 100.0, !flip_y);
        trans_type.set_text_size(7., 0.);
        trans_type.add_item("Affine No Resample");
        trans_type.add_item("Affine Resample");
        trans_type.add_item("Perspective No Resample LERP");
        trans_type.add_item("Perspective No Resample Exact");
        trans_type.add_item("Perspective Resample LERP");
        trans_type.add_item("Perspective Resample Exact");
        trans_type.set_cur_item(4);
        let mut gamma = Slider::new(5.0, 5.0 + 15. * 0., 400.0 - 5.0, 10.0 + 15. * 0., !flip_y);
        gamma.set_range(0.5, 3.0);
        gamma.set_value(2.0);
        gamma.set_label("Gamma=%.3f");
        let mut blur = Slider::new(5.0, 5.0 + 15. * 1., 400.0 - 5.0, 10.0 + 15. * 1., !flip_y);
        blur.set_range(0.5, 2.0);
        blur.set_value(1.0);
        blur.set_label("Blur=%.3f");
        let trans_type = ctrl_ptr(trans_type);
        let gamma = ctrl_ptr(gamma);
        let blur = ctrl_ptr(blur);
        Self {
            gamma_lut,
            quad,
            trans_type: trans_type.clone(),
            gamma: gamma.clone(),
            blur: blur.clone(),
            old_gamma: 2.0,
            x1: 0.,
            x2: 0.,
            y1: 0.,
            y2: 0.,
            rasterizer: agg::RasterizerScanlineAa::new(),
            scanline: agg::ScanlineU8::new(),
            ctrls: CtrlContainer {
                ctrl: vec![trans_type, gamma, blur],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        }
    }

    // On init
    fn on_init(&mut self) {
        self.x1 = -150.;
        self.y1 = -150.;
        self.x2 = 150.;
        self.y2 = 150.;

        let trans_x1 = -200.;
        let trans_y1 = -200.;
        let trans_x2 = 200.;
        let trans_y2 = 200.;

        let dx = self.util.borrow().width() / 2.0 - (trans_x2 + trans_x1) / 2.0;
        let dy = self.util.borrow().height() / 2.0 - (trans_y2 + trans_y1) / 2.0;
        *self.quad.xn_mut(0) = (trans_x1 + dx).floor();
        *self.quad.yn_mut(0) = (trans_y1 + dy).floor();
        *self.quad.xn_mut(1) = (trans_x2 + dx).floor();
        *self.quad.yn_mut(1) = (trans_y1 + dy).floor();
        *self.quad.xn_mut(2) = (trans_x2 + dx).floor();
        *self.quad.yn_mut(2) = (trans_y2 + dy).floor();
        *self.quad.xn_mut(3) = (trans_x1 + dx).floor();
        *self.quad.yn_mut(3) = (trans_y2 + dy).floor();

        let mut pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        pixf.apply_gamma_dir(&self.gamma_lut);
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.quad.on_mouse_button_down(x as f64, y as f64) {
                return Draw::Yes;
            }
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.quad.on_mouse_move(x as f64, y as f64) {
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
        if self.quad.on_mouse_button_up(x as f64, y as f64) {
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        if key == ' ' as u32 {
            let cx = (self.quad.xn(0) + self.quad.xn(1) + self.quad.xn(2) + self.quad.xn(3)) / 4.0;
            let cy = (self.quad.yn(0) + self.quad.yn(1) + self.quad.yn(2) + self.quad.yn(3)) / 4.0;
            let mut tr = agg::TransAffine::trans_affine_translation(-cx, -cy);
            tr *= agg::TransAffine::trans_affine_rotation(PI / 2.0);
            tr *= agg::TransAffine::trans_affine_translation(cx, cy);
            tr.transform(&mut self.quad.xn(0), &mut self.quad.yn(0));
            tr.transform(&mut self.quad.xn(1), &mut self.quad.yn(1));
            tr.transform(&mut self.quad.xn(2), &mut self.quad.yn(2));
            tr.transform(&mut self.quad.xn(3), &mut self.quad.yn(3));
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        if self.gamma.borrow().value() != self.old_gamma {
            self.gamma_lut.set_gamma(self.gamma.borrow().value());
            self.util.borrow_mut().load_img(0, "agg");
            let mut pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
            pixf.apply_gamma_dir(&self.gamma_lut);
            self.old_gamma = self.gamma.borrow().value();
        }

        let mut pixf = agg::PixBgr24::new_owned(rbuf.clone());
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut pixf_pre = agg::PixBgr24Pre::new_borrowed(rbuf);
        let mut rb_pre = agg::RendererBase::new_borrowed(&mut pixf_pre);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        if self.trans_type.borrow().cur_item() < 2 {
            // For the affine parallelogram transformations we
            // calculate the 4-th (implicit) point of the parallelogram
            *self.quad.xn_mut(3) = self.quad.xn(0) + (self.quad.xn(2) - self.quad.xn(1));
            *self.quad.yn_mut(3) = self.quad.yn(0) + (self.quad.yn(2) - self.quad.yn(1));
        }

        //--------------------------
        // Render the "quad" tool and controls
        self.rasterizer.add_path(&mut self.quad, 0);
        r.set_color(agg::Rgba8::new_params(0, 75, 125, 25));
        agg::render_scanlines(&mut self.rasterizer, &mut self.scanline, &mut r);

        // Prepare the polygon to rasterize. Here we need to fill
        // the destination (transformed) polygon.
        self.rasterizer.clip_box(
            0.,
            0.,
            self.util.borrow().width(),
            self.util.borrow().height(),
        );
        self.rasterizer.reset();
        let b = 0.;
        self.rasterizer
            .move_to_d(self.quad.xn(0) - b, self.quad.yn(0) - b);
        self.rasterizer
            .line_to_d(self.quad.xn(1) + b, self.quad.yn(1) - b);
        self.rasterizer
            .line_to_d(self.quad.xn(2) + b, self.quad.yn(2) + b);
        self.rasterizer
            .line_to_d(self.quad.xn(3) - b, self.quad.yn(3) + b);

        let mut sa = agg::VecSpan::new();
        let filter_kernel = agg::ImageFilterHanning::new();
        let filter = agg::ImageFilterLut::new_filter(&filter_kernel, true);

        //let wrap_type = agg::WrapModeReflectAutoPow2::new(0);

        let img_pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        let mut img_src: agg::ImageAccessorWrap<
            '_,
            _,
            agg::WrapModeReflectAutoPow2,
            agg::WrapModeReflectAutoPow2,
        > = agg::ImageAccessorWrap::new(img_pixf);

        self.util.borrow_mut().start_timer();

        match self.trans_type.borrow().cur_item() {
            0 => {
                let tr = agg::TransAffine::new_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );

                let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(tr);

                let mut sg: agg::SpanImageFilterRgb2x2<'_, _, _> =
                    agg::SpanImageFilterRgb2x2::new(&mut img_src, &mut interpolator, filter);
                agg::render_scanlines_aa(
                    &mut self.rasterizer,
                    &mut self.scanline,
                    &mut rb_pre,
                    &mut sa,
                    &mut sg,
                );
            }
            1 => {
                let tr = agg::TransAffine::new_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );

                let mut interpolator = agg::SpanIpLinear::new(tr);
                let mut sg: agg::SpanImageResampleRgbAffine<'_, _> =
                    agg::SpanImageResampleRgbAffine::new(&mut img_src, &mut interpolator, filter);
                sg.base_mut().set_blur(self.blur.borrow().value());
                agg::render_scanlines_aa(
                    &mut self.rasterizer,
                    &mut self.scanline,
                    &mut rb_pre,
                    &mut sa,
                    &mut sg,
                );
            }
            2 => {
                let tr = agg::TransPerspective::new_quad_to_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );
                if tr.is_valid(agg::trans_affine::AFFINE_EPSILON) {
                    let mut interpolator: agg::SpanIpLinearSubdiv<_> =
                        agg::SpanIpLinearSubdiv::new(tr);

                    let mut sg: agg::SpanImageFilterRgb2x2<'_, _, _> =
                        agg::SpanImageFilterRgb2x2::new(&mut img_src, &mut interpolator, filter);
                    agg::render_scanlines_aa(
                        &mut self.rasterizer,
                        &mut self.scanline,
                        &mut rb_pre,
                        &mut sa,
                        &mut sg,
                    );
                }
            }
            3 => {
                let tr = agg::TransPerspective::new_quad_to_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );
                if tr.is_valid(agg::trans_affine::AFFINE_EPSILON) {
                    let mut interpolator: agg::SpanIpTrans<_> = agg::SpanIpTrans::new(tr);

                    let mut sg: agg::SpanImageFilterRgb2x2<'_, _, _> =
                        agg::SpanImageFilterRgb2x2::new(&mut img_src, &mut interpolator, filter);
                    agg::render_scanlines_aa(
                        &mut self.rasterizer,
                        &mut self.scanline,
                        &mut rb_pre,
                        &mut sa,
                        &mut sg,
                    );
                }
            }
            4 => {
                let interpolator: agg::SpanIpPerspLerp = agg::SpanIpPerspLerp::new_quad_to_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );
                let mut subdiv_adaptor: agg::SpanSubdivAdaptor<_> =
                    agg::SpanSubdivAdaptor::new(interpolator, 4);
                if subdiv_adaptor.interpolator().is_valid() {
                    let mut sg: agg::SpanImageResampleRgb<'_, _, _> =
                        agg::SpanImageResampleRgb::new(&mut img_src, &mut subdiv_adaptor, filter);
                    sg.base_mut().set_blur(self.blur.borrow().value());
                    agg::render_scanlines_aa(
                        &mut self.rasterizer,
                        &mut self.scanline,
                        &mut rb_pre,
                        &mut sa,
                        &mut sg,
                    );
                }
            }
            5 => {
                let interpolator: agg::SpanIpPerspExact = agg::SpanIpPerspExact::new_quad_to_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );
                let mut subdiv_adaptor: agg::SpanSubdivAdaptor<_> =
                    agg::SpanSubdivAdaptor::new(interpolator, 4);
                if subdiv_adaptor.interpolator().is_valid() {
                    let mut sg: agg::SpanImageResampleRgb<'_, _, _> =
                        agg::SpanImageResampleRgb::new(&mut img_src, &mut subdiv_adaptor, filter);
                    sg.base_mut().set_blur(self.blur.borrow().value());
                    agg::render_scanlines_aa(
                        &mut self.rasterizer,
                        &mut self.scanline,
                        &mut rb_pre,
                        &mut sa,
                        &mut sg,
                    );
                }
            }
            _ => {}
        }

        let tm = self.util.borrow_mut().elapsed_time();
        r.ren_mut().ren_mut().apply_gamma_inv(&self.gamma_lut);
        //pixf.apply_gamma_inv(&self.gamma_lut);

        let mut t = agg::GsvText::new();
        t.set_size(10.0, 0.);
        let buf = format!("{:3.2} ms", tm);
        t.set_start_point(10.0, 70.0);
        t.set_text(&buf);

        let mut pt: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(t);
        pt.set_width(1.5);

        self.rasterizer.add_path(&mut pt, 0);
        r.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        agg::render_scanlines(&mut self.rasterizer, &mut self.scanline, &mut r);

        //--------------------------
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.trans_type.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.gamma.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.blur.borrow_mut(),
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "agg.bmp";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);

    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("agg.bmp") {
            buf = format!(
                "File not found: {}. Download http://www.antigrain.com/{}
				or copy it from another directory if available.",
                img_name, img_name
            );
        } else {
            buf = format!("File not found: {}", img_name);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
        return;
    }

    plat.set_caption("AGG Example. Pattern Transformations with Resampling");

    if plat.init(600, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

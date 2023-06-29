use crate::platform::*;
use agg::{RasterScanLine, RenderBuf};

mod ctrl;
mod platform;

use crate::ctrl::rbox::Rbox;

use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::interactive_polygon::InteractivePolygon;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

struct Application {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    rasterizer: agg::RasterizerScanlineAa,
    scanline: agg::ScanlineU8,
    quad: InteractivePolygon<'static>,
    trans_type: Ptr<Rbox<'static, agg::Rgba8>>,
    test_flag: bool,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let trans_type = ctrl_ptr(Rbox::new(460., 5.0, 420. + 170.0, 60.0, !flip_y));
        trans_type.borrow_mut().set_text_size(8., 0.);
        trans_type.borrow_mut().set_text_thickness(1.);
        trans_type.borrow_mut().add_item("Affine");
        trans_type.borrow_mut().add_item("Bilinear");
        trans_type.borrow_mut().add_item("Perspective");
        trans_type.borrow_mut().set_cur_item(2);

        Application {
            rasterizer: agg::RasterizerScanlineAa::new(),
            scanline: agg::ScanlineU8::new(),
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            quad: InteractivePolygon::new(4, 5.0),
            trans_type: trans_type.clone(),
            test_flag: false,
            ctrls: CtrlContainer {
                ctrl: vec![trans_type],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        }
    }

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
    }

    fn on_mouse_button_down(&mut self, rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.quad.on_mouse_button_down(x as f64, y as f64) {
                return Draw::Yes;
            } else {
                self.util.borrow_mut().start_timer();
                self.test_flag = true;
                self.on_draw(rb);
                self.on_draw(rb);
                self.on_draw(rb);
                self.on_draw(rb);
                let buf = format!("time={:.3}", self.util.borrow_mut().elapsed_time());
                self.test_flag = false;

                self.util.borrow_mut().message(&buf);
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

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgr24::new_owned(rbuf.clone());
        let mut pixf_pre = agg::PixBgr24Pre::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut rb_pre = agg::RendererBase::new_borrowed(&mut pixf_pre);

        if !self.test_flag {
            rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        }

        if self.trans_type.borrow().cur_item() == 0 {
            // For the affine parallelogram transformations we
            // calculate the 4-th (implicit) point of the parallelogram
            *self.quad.xn_mut(3) = self.quad.xn(0) + (self.quad.xn(2) - self.quad.xn(1));
            *self.quad.yn_mut(3) = self.quad.yn(0) + (self.quad.yn(2) - self.quad.yn(1));
        }

        if !self.test_flag {
            //--------------------------
            // Render the "quad" tool and controls
            self.rasterizer.add_path(&mut self.quad, 0);
            agg::render_scanlines_aa_solid(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut rb,
                &agg::Rgba8::new_params(0, 75, 125, 150),
            );

            //--------------------------
            ctrl::render_ctrl(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut rb,
                &mut *self.trans_type.borrow_mut(),
            );
        }

        // Prepare the polygon to rasterize. Here we need to fill
        // the destination (transformed) polygon.
        self.rasterizer.clip_box(
            0.,
            0.,
            self.util.borrow().width(),
            self.util.borrow().height(),
        );
        self.rasterizer.reset();
        self.rasterizer.move_to_d(self.quad.xn(0), self.quad.yn(0));
        self.rasterizer.line_to_d(self.quad.xn(1), self.quad.yn(1));
        self.rasterizer.line_to_d(self.quad.xn(2), self.quad.yn(2));
        self.rasterizer.line_to_d(self.quad.xn(3), self.quad.yn(3));

        let mut sa = agg::VecSpan::new();
        // let filter = agg::ImageFilter::new(agg::ImageFilterHanning::new());
        let filter_kernel = agg::ImageFilterHanning::new();
        let filter = agg::ImageFilterLut::new_filter(&filter_kernel, false);
        let img_pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        let mut img_src: agg::ImageAccessorWrap<
            '_,
            _,
            agg::WrapModeReflectAutoPow2,
            agg::WrapModeReflectAutoPow2,
        > = agg::ImageAccessorWrap::new(img_pixf);

        let subdiv_shift = 2;

        match self.trans_type.borrow().cur_item() {
            0 => {
                // Note that we consruct an affine matrix that transforms
                // a parallelogram to a rectangle, i.e., it's inverted.
                // It's actually the same as:
                // tr(self.x1,self.y1,self.x2,self.y2, m_triangle.polygon());
                // tr.invert();
                let tr = agg::TransAffine::new_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );

                // Also note that we can use the linear interpolator instead of
                // arbitrary span_interpolator_trans. It works much faster,
                // but the transformations must be linear and parellel.

                let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(tr);

                let mut sg =
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
                let tr = agg::TransBilinear::new_quad_to_rect(
                    self.quad.polygon(),
                    self.x1,
                    self.y1,
                    self.x2,
                    self.y2,
                );
                if tr.is_valid() {
                    let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(tr);

                    let mut sg =
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

                    let mut sg =
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
            _ => {}
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "agg";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);

    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("agg") {
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

    plat.set_caption("AGG Example. Pattern Perspective Transformations");

    if plat.init(600, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

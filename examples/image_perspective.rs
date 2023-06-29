use crate::platform::*;

use agg::{RasterScanLine, RenderBuf, RenderBuffer};

mod ctrl;
mod platform;

use crate::ctrl::rbox::Rbox;

use std::cell::RefCell;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

mod misc;
use misc::interactive_polygon::InteractivePolygon;

const FLIP_Y: bool = true;

struct Application {
    scanline: agg::ScanlineU8,
    rasterizer: agg::RasterizerScanlineAa,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    quad: InteractivePolygon<'static>,
    trans_type: Ptr<Rbox<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let trans_type = ctrl_ptr(Rbox::new(420.0, 5.0, 420.0 + 170.0, 70.0, !flip_y));
        trans_type.borrow_mut().add_item("Affine Parallelogram");
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
            ctrls: CtrlContainer {
                ctrl: vec![trans_type],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        let d = 0.0;
        self.x1 = d;
        self.y1 = d;
        self.x2 = self.util.borrow_mut().rbuf_img(0).width() as f64 - d;
        self.y2 = self.util.borrow_mut().rbuf_img(0).height() as f64 - d;

        *self.quad.xn_mut(0) = 100.;
        *self.quad.yn_mut(0) = 100.;
        *self.quad.xn_mut(1) = self.util.borrow().width() - 100.;
        *self.quad.yn_mut(1) = 100.;
        *self.quad.xn_mut(2) = self.util.borrow().width() - 100.;
        *self.quad.yn_mut(2) = self.util.borrow().height() - 100.;
        *self.quad.xn_mut(3) = 100.;
        *self.quad.yn_mut(3) = self.util.borrow().height() - 100.;
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

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgra32::new_owned(rbuf.clone());
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut pixf_pre = agg::PixBgra32Pre::new_borrowed(rbuf);
        let mut rb_pre = agg::RendererBase::new_borrowed(&mut pixf_pre);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        //let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        if self.trans_type.borrow().cur_item() == 0 {
            *self.quad.xn_mut(3) = self.quad.xn(0) + (self.quad.xn(2) - self.quad.xn(1));
            *self.quad.yn_mut(3) = self.quad.yn(0) + (self.quad.yn(2) - self.quad.yn(1));
        }

        //--------------------------
        // Render the "quad" tool and controls
        self.rasterizer.add_path(&mut self.quad, 0);
        agg::render_scanlines_aa_solid(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &agg::Rgba8::new_params(0, 75, 125, 150),
        );

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
        let filter_kernel = agg::ImageFilterBilinear::new();
        let filter = agg::ImageFilterLut::new_filter(&filter_kernel, false);

        let pixf_img = agg::PixBgra32::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());
        let mut ia = agg::ImageAccessorClone::new(pixf_img);

        self.util.borrow_mut().start_timer();
        match self.trans_type.borrow().cur_item() {
            0 => {
                // Note that we consruct an affine matrix that transforms
                // a parallelogram to a rectangle, i.e., it's inverted.
                // It's actually the same as:
                // tr(self.x1, self.y1, self.x2, self.y2, m_triangle.polygon());
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

                let mut sg = agg::SpanImageFilterRgbaNn::new(&mut ia, &mut interpolator);
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
                        agg::SpanImageFilterRgba2x2::new(&mut ia, &mut interpolator, filter);
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
                    // Subdivision and linear interpolation (faster, but less accurate)
                    //-----------------------
                    //let interpolator = SpanInterpolatorLinear::<TransPerspective>::new(tr);
                    //let subdiv_adaptor = SpanSubdivAdaptor::<InterpolatorType>::new(interpolator);
                    //let sg = SpanImageFilterRgba2x2::<ImgAccessorType, SpanSubdivAdaptor<InterpolatorType>>::new(ia, subdiv_adaptor, filter);
                    //-----------------------

                    // Direct calculations of the coordinates
                    //-----------------------
                    let mut interpolator = agg::SpanIpTrans::<agg::TransPerspective>::new(tr);
                    let mut sg =
                        agg::SpanImageFilterRgba2x2::new(&mut ia, &mut interpolator, filter);
                    //-----------------------

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

        let mut t = agg::GsvText::new();
        t.set_size(10.0, 0.);
        let buf = format!("{:3.2} ms", tm);
        t.set_start_point(10.0, 10.0);
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

        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.trans_type.borrow_mut(),
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "spheres.bmp";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);

    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("spheres.bmp") {
            buf = format!(
                "File not found: {}. Download http://www.antigrain.com/{}
				or copy it from another directory if available.",
                img_name, img_name
            );
        } else {
            buf = format!("File not found: {}", img_name);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
    }

    plat.set_caption("AGG Example. Image Perspective Transformations");

    if plat.init(600, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

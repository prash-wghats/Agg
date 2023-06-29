use agg::image_filters::{ImageFilterKaiser, ImageFilterSpline36};
use agg::rendering_buffer::*;
use agg::trans_affine::*;
use agg::RasterScanLine;
use agg::RenderBuffer;

use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}
struct Application {
    angle: Ptr<Slider<'static, agg::Rgba8>>,
    scale: Ptr<Slider<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let angle = ctrl_ptr(Slider::<agg::Rgba8>::new(5., 5., 300., 12., !flip_y));
        let scale = ctrl_ptr(Slider::<agg::Rgba8>::new(
            5.,
            5. + 15.,
            300.,
            12. + 15.,
            !flip_y,
        ));

        angle.borrow_mut().set_label("Angle=%3.2f");
        scale.borrow_mut().set_label("Scale=%3.2f");
        angle.borrow_mut().set_range(-180.0, 180.0);
        angle.borrow_mut().set_value(0.0);
        scale.borrow_mut().set_range(0.1, 5.0);
        scale.borrow_mut().set_value(1.0);
        Application {
            angle: angle.clone(),
            scale: scale.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![angle, scale],
                cur_ctrl: -1,
                num_ctrl: 2,
            },
            util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = agg::PixBgr24::new_owned(rbuf.clone());
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pixf);

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineU8::new();

        let mut pixf_pre = agg::PixBgr24Pre::new_borrowed(rbuf);
        let mut rb_pre = agg::RendererBase::<agg::PixBgr24Pre>::new_borrowed(&mut pixf_pre);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut src_mtx = agg::TransAffine::new_default();
        src_mtx *= TransAffine::trans_affine_translation(
            -self.util.borrow().initial_width() / 2.0 - 10.0,
            -self.util.borrow().initial_height() / 2.0 - 20.0 - 10.0,
        );
        src_mtx *= TransAffine::trans_affine_rotation(self.angle.borrow().value() * PI / 180.0);
        src_mtx *= TransAffine::trans_affine_scaling_eq(self.scale.borrow().value());
        src_mtx *= TransAffine::trans_affine_translation(
            self.util.borrow().initial_width() / 2.0,
            self.util.borrow().initial_height() / 2.0 + 20.0,
        );
        src_mtx *= *self.util.borrow().trans_affine_resizing();

        let mut img_mtx = agg::TransAffine::new_default();
        img_mtx *= TransAffine::trans_affine_translation(
            -self.util.borrow().initial_width() / 2.0 + 10.0,
            -self.util.borrow().initial_height() / 2.0 + 20.0 + 10.0,
        );
        img_mtx *= TransAffine::trans_affine_rotation(self.angle.borrow().value() * PI / 180.0);
        img_mtx *= TransAffine::trans_affine_scaling_eq(self.scale.borrow().value());
        img_mtx *= TransAffine::trans_affine_translation(
            self.util.borrow().initial_width() / 2.0,
            self.util.borrow().initial_height() / 2.0 + 20.0,
        );
        img_mtx *= *self.util.borrow().trans_affine_resizing();
        img_mtx.invert();

        let mut sa = agg::VecSpan::<agg::Rgba8>::new();

        let mut interpolator: agg::SpanIpLinear<agg::TransAffine> = agg::SpanIpLinear::new(img_mtx);

        let img_pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());

        let mut img_src =
            agg::ImageAccessorClip::new(img_pixf, &agg::Rgba8::new_params(0, 100, 0, 127));

        /*
               // Version without filtering (nearest neighbor)
               let mut sg = agg::SpanImageFilterRgbNn::new(img_src, interpolator);
        */
        /*
                // Version with "hardcoded" bilinear filter and without
                // image_accessor (direct filter, the old variant)
                //------------------------------------------
                let mut sg = agg::SpanImageFilterRgbBilinearClip::new(
                    img_pixf,
                    agg::Rgba8::new_params(0, 100, 0, 127),
                    interpolator,
                );
        */
        /*
                // Version with arbitrary 2x2 filter
                let mut sg = agg::SpanImageFilterRgb2x2::new(
                    img_src,
                    interpolator,
                    agg::ImageFilterLut::new_filter(&ImageFilterKaiser::new(), true),
                );
        */

        // Version with arbitrary filter
        let mut sg = agg::SpanImageFilterRgb::new(
            &mut img_src,
            &mut interpolator,
            agg::ImageFilterLut::new_filter(&ImageFilterSpline36::new(), true),
        );

        ras.clip_box(
            0.,
            0.,
            self.util.borrow().width(),
            self.util.borrow().height(),
        );

        let mut r = self.util.borrow().initial_width();
        let h = self.util.borrow().initial_height();
        r = if h - 60. < r { h - 60. } else { r };
        let ell = agg::Ellipse::new_ellipse(
            self.util.borrow().initial_width() / 2.0 + 10.0,
            self.util.borrow().initial_height() / 2.0 + 20.0 + 10.0,
            r / 2.0 + 16.0,
            r / 2.0 + 16.0,
            200,
            false,
        );

        let mut tr = agg::ConvTransform::new_owned(ell, src_mtx);
        ras.add_path(&mut tr, 0);
        agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb_pre, &mut sa, &mut sg);

        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.angle.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.scale.borrow_mut());
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "spheres.bmp";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);

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

    plat.set_caption("Image Affine Transformations with filtering");

    let w = plat.app().util.borrow().rbuf_img(0).width();
    let h = plat.app().util.borrow().rbuf_img(0).height();
    //if plat.init(600, 400, WindowFlag::Resize as u32) {
    if plat.init(w + 20, h + 40 + 20, WindowFlag::Resize as u32) {
        plat.run();
    }
}

use crate::platform::*;

use agg::{ RasterScanLine, RenderBuf};

mod ctrl;
mod platform;

use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

// Static Global Variable
static mut IMAGE: [u8; 64] = [
    0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255, 255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 0,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 255,
    255, 255, 0, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255, 0, 255, 0, 255,
];

struct Application {
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    radius: Ptr<Slider<'static, agg::Rgba8>>,
    filters: Ptr<Rbox<'static, agg::Rgba8>>,
    normalize: Ptr<Cbox<'static, agg::Rgba8>>,
    cur_angle: f64,
    cur_filter: i32,
    num_steps: i32,
    num_pix: f64,
    time1: f64,
    time2: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let gamma = ctrl_ptr(Slider::new(115., 5., 500. - 5., 11., !flip_y));
        let radius = ctrl_ptr(Slider::new(115., 5. + 15., 500. - 5., 11. + 15., !flip_y));
        let filters = ctrl_ptr(Rbox::new(0.0, 0.0, 110.0, 210.0, !flip_y));
        let normalize = ctrl_ptr(Cbox::new(8.0, 215.0, "Normalize Filter", !flip_y));

        normalize.borrow_mut().set_text_size(7.5, 0.);
        normalize.borrow_mut().set_status(true);
        radius.borrow_mut().set_label("Filter Radius=%.3f");
        radius.borrow_mut().set_range(2.0, 8.0);
        radius.borrow_mut().set_value(4.0);
        gamma.borrow_mut().set_label("Gamma=%.3f");
        gamma.borrow_mut().set_range(0.5, 3.0);
        gamma.borrow_mut().set_value(1.0);
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
            gamma: gamma.clone(),
            radius: radius.clone(),
            filters: filters.clone(),
            normalize: normalize.clone(),
            cur_angle: 0.0,
            cur_filter: 1,
            num_steps: 0,
            num_pix: 0.0,
            time1: 0.,
            time2: 0.,
            ctrls: CtrlContainer {
                ctrl: vec![gamma, radius, filters, normalize],
                cur_ctrl: -1,
                num_ctrl: 4,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = agg::PixBgra32::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgra32>::new_borrowed(&mut pixf);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        rb.copy_from(self.util.borrow_mut().rbuf_img(0), None, 110, 35);

        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let img_rbuf = agg::RenderBuf::new(unsafe { IMAGE.as_mut_ptr() }, 4, 4, 4 * 4);

        let para = [
            200.,
            40.,
            200. + 300.,
            40.,
            200. + 300.,
            40. + 300.,
            200.,
            40. + 300.,
        ];
        let img_mtx = agg::TransAffine::new_rect(&para, 0., 0., 4., 4.);
        let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(img_mtx);
        let mut sa = agg::VecSpan::new();

        let img_pixf = agg::PixBgra32::new_owned(img_rbuf);
        let mut source = agg::ImageAccessorClone::new(img_pixf);

        ras.reset();
        ras.move_to_d(para[0], para[1]);
        ras.line_to_d(para[2], para[3]);
        ras.line_to_d(para[4], para[5]);
        ras.line_to_d(para[6], para[7]);

        let cur_item = self.filters.borrow().cur_item();
        match cur_item {
            0 => {
                let mut sg = agg::SpanImageFilterRgbaNn::new(&mut source, &mut interpolator);
                agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb, &mut sa, &mut sg);
            }
            _ => {
                let mut filter = agg::ImageFilterLut::new();
                let norm = self.normalize.borrow().status();
                match cur_item {
                    1 => filter.calculate(&agg::ImageFilterBilinear::new(), norm),
                    2 => filter.calculate(&agg::ImageFilterBicubic::new(), norm),
                    3 => filter.calculate(&agg::ImageFilterSpline16::new(), norm),
                    4 => filter.calculate(&agg::ImageFilterSpline36::new(), norm),
                    5 => filter.calculate(&agg::ImageFilterHanning::new(), norm),
                    6 => filter.calculate(&agg::ImageFilterHamming::new(), norm),
                    7 => filter.calculate(&agg::ImageFilterHermite::new(), norm),
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
                    _ => panic!("unrecognized item {}", cur_item),
                }

                let mut sg = agg::SpanImageFilterRgba::new(&mut source, &mut interpolator, filter);
                agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb, &mut sa, &mut sg);

                let gamma: agg::GammaLut =
                    agg::GammaLut::new_with_gamma(self.gamma.borrow().value());
                rb.ren_mut().apply_gamma_inv(&gamma);

                let x_start = 5.0;
                let x_end = 195.0;
                let y_start = 235.0;
                let y_end = self.util.borrow().initial_height() - 5.0;
                let x_center = (x_start + x_end) / 2.0;

                let p = agg::PathStorage::new();
                let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(p);
                stroke.set_width(0.8);

                for i in 0..=16 {
                    let x = x_start + (x_end - x_start) * i as f64 / 16.0;
                    stroke.source_mut().remove_all();
                    stroke.source_mut().move_to(x + 0.5, y_start);
                    stroke.source_mut().line_to(x + 0.5, y_end);
                    ras.add_path(&mut stroke, 0);
                    agg::render_scanlines_aa_solid(
                        &mut ras,
                        &mut sl,
                        &mut rb,
                        &agg::Rgba8::new_params(0, 0, 0, if i == 8 { 255 } else { 100 }),
                    );
                }

                let ys = y_start + (y_end - y_start) / 6.0;
                stroke.source_mut().remove_all();
                stroke.source_mut().move_to(x_start, ys);
                stroke.source_mut().line_to(x_end, ys);
                ras.add_path(&mut stroke, 0);
                agg::render_scanlines_aa_solid(
                    &mut ras,
                    &mut sl,
                    &mut rb,
                    &agg::Rgba8::new_params(0, 0, 0, 255),
                );

                let radius = sg.base_mut().filter().radius();
                let n = (radius * 256. * 2.) as usize;
                let dx = (x_end - x_start) * radius / 8.0;
                let dy = y_end - ys;

				let dia = sg.base_mut().filter().diameter();
				let weights = sg.base_mut().filter().weight_array();
                let xs =
                    (x_end + x_start) / 2.0 - (dia as f64 * (x_end - x_start) / 32.0);
                let nn = dia * 256;
                stroke.source_mut().remove_all();
                stroke.source_mut().move_to(
                    xs + 0.5,
                    ys + dy * weights[0] as f64 / agg::ImageFilterScale::Scale as u32 as f64,
                );
                for i in 1..nn as usize {
                    stroke.source_mut().line_to(
                        xs + dx * i as f64 / n as f64 + 0.5,
                        ys + dy * weights[i] as f64 / agg::ImageFilterScale::Scale as u32 as f64,
                    );
                }
                ras.add_path(&mut stroke, 0);
                agg::render_scanlines_aa_solid(
                    &mut ras,
                    &mut sl,
                    &mut rb,
                    &agg::Rgba8::new_params(100, 0, 0, 255),
                );
            }
        }

        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.gamma.borrow_mut());
        if self.filters.borrow().cur_item() >= 14 {
            ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.radius.borrow_mut());
        }
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.filters.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.normalize.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);
    plat.set_caption("Image transformation filters comparison");

    if plat.init(500, 340, 0) {
        plat.run();
    }
}

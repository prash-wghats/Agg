
use agg::image_filters::*;
use agg::path_storage::*;
use agg::rendering_buffer::*;

use agg::{FilterF, RasterScanLine, RendererScanlineColor};

use crate::ctrl::cbox::Cbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

#[allow(dead_code)]
struct Application {
    radius: Ptr<Slider<'static, agg::Rgba8>>,
    bilinear: Ptr<Cbox<'static, agg::Rgba8>>,
    bicubic: Ptr<Cbox<'static, agg::Rgba8>>,
    spline16: Ptr<Cbox<'static, agg::Rgba8>>,
    spline36: Ptr<Cbox<'static, agg::Rgba8>>,
    hanning: Ptr<Cbox<'static, agg::Rgba8>>,
    hamming: Ptr<Cbox<'static, agg::Rgba8>>,
    hermite: Ptr<Cbox<'static, agg::Rgba8>>,
    kaiser: Ptr<Cbox<'static, agg::Rgba8>>,
    quadric: Ptr<Cbox<'static, agg::Rgba8>>,
    catrom: Ptr<Cbox<'static, agg::Rgba8>>,
    gaussian: Ptr<Cbox<'static, agg::Rgba8>>,
    bessel: Ptr<Cbox<'static, agg::Rgba8>>,
    mitchell: Ptr<Cbox<'static, agg::Rgba8>>,
    sinc: Ptr<Cbox<'static, agg::Rgba8>>,
    lanczos: Ptr<Cbox<'static, agg::Rgba8>>,
    blackman: Ptr<Cbox<'static, agg::Rgba8>>,
    filters: [Ptr<Cbox<'static,agg::Rgba8>>; 16],
    filter_func: [Box<dyn FilterF>; 16],
    num_filters: u32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let radius = ctrl_ptr(Slider::new(5.0, 5.0, 780. - 5., 10.0, !flip_y));
        let bilinear = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 0., "bilinear", !flip_y));
        let bicubic = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 1., "bicubic ", !flip_y));
        let spline16 = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 2., "spline16", !flip_y));
        let spline36 = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 3., "spline36", !flip_y));
        let hanning = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 4., "hanning ", !flip_y));
        let hamming = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 5., "hamming ", !flip_y));
        let hermite = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 6., "hermite ", !flip_y));
        let kaiser = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 7., "kaiser  ", !flip_y));
        let quadric = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 8., "quadric ", !flip_y));
        let catrom = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 9., "catrom  ", !flip_y));
        let gaussian = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 10., "gaussian", !flip_y));
        let bessel = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 11., "bessel  ", !flip_y));
        let mitchell = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 12., "mitchell", !flip_y));
        let sinc = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 13., "sinc    ", !flip_y));
        let lanczos = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 14., "lanczos ", !flip_y));
        let blackman = ctrl_ptr(Cbox::new(8.0, 30.0 + 15. * 15., "blackman", !flip_y));
        radius.borrow_mut().set_range(2.0, 8.0);
        radius.borrow_mut().set_value(4.0);
        radius.borrow_mut().set_label("Radius=%0.3f");
		
        Self {
            radius: radius.clone(),
            bilinear: bilinear.clone(),
            bicubic: bicubic.clone(),
            spline16: spline16.clone(),
            spline36: spline36.clone(),
            hanning: hanning.clone(),
            hamming: hamming.clone(),
            hermite: hermite.clone(),
            kaiser: kaiser.clone(),
            quadric: quadric.clone(),
            catrom: catrom.clone(),
            gaussian: gaussian.clone(),
            bessel: bessel.clone(),
            mitchell: mitchell.clone(),
            blackman: blackman.clone(),
            sinc: sinc.clone(),
            lanczos: lanczos.clone(),
            num_filters: 16,
            filters: [
                bilinear.clone(),
                bicubic.clone(),
                spline16.clone(),
                spline36.clone(),
                hanning.clone(),
                hamming.clone(),
                hermite.clone(),
                kaiser.clone(),
                quadric.clone(),
                catrom.clone(),
                gaussian.clone(),
                bessel.clone(),
                mitchell.clone(),
                sinc.clone(),
                lanczos.clone(),
                blackman.clone(),
            ],
            filter_func: [
                Box::new(ImageFilterBilinear::new()),
                Box::new(ImageFilterBicubic::new()),
                Box::new(ImageFilterSpline16::new()),
                Box::new(ImageFilterSpline36::new()),
                Box::new(ImageFilterHanning::new()),
                Box::new(ImageFilterHamming::new()),
                Box::new(ImageFilterHermite::new()),
                Box::new(ImageFilterKaiser::new()),
                Box::new(ImageFilterQuadric::new()),
                Box::new(ImageFilterCatrom::new()),
                Box::new(ImageFilterGaussian::new()),
                Box::new(ImageFilterBessel::new()),
                Box::new(ImageFilterMitchell::new()),
                Box::new(ImageFilterSinc::new()),
                Box::new(ImageFilterLanczos::new()),
                Box::new(ImageFilterBlackman::new()),
            ],
            ctrls: CtrlContainer {
                ctrl: vec![
                    radius, bilinear, bicubic, spline16, spline36, hanning, hamming, hermite,
                    kaiser, quadric, catrom, gaussian, bessel, mitchell, sinc, lanczos, blackman,
                ],
                cur_ctrl: -1,
                num_ctrl: 17,
            },
            util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut pixf = agg::PixBgr24::new_borrowed(rb);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pixf);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
        let mut sl = agg::ScanlineP8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let x_start = 125.0;
        let x_end = self.util.borrow().initial_width() - 15.0;
        let y_start = 10.0;
        let y_end = self.util.borrow().initial_height() - 10.0;
        let x_center = (x_start + x_end) / 2.0;
        let p = PathStorage::new();
        let pl:agg::ConvStroke<'_,_> = agg::ConvStroke::new_owned(p);
        let mut tr = agg::ConvTransform::new_owned(pl, *self.util.borrow().trans_affine_resizing());

        for i in 0..=16 {
            let x = x_start + (x_end - x_start) * i as f64 / 16.0;

            tr.source_mut().source_mut().remove_all();
            tr.source_mut().source_mut().move_to(x + 0.5, y_start);
            tr.source_mut().source_mut().line_to(x + 0.5, y_end);

            ras.add_path(&mut tr, 0);
            rs.set_color(agg::Rgba8::new_params(
                0,
                0,
                0,
                if i == 8 { 255 } else { 100 },
            ));
            agg::render_scanlines(&mut ras, &mut sl, &mut rs);
        }

        let ys = y_start + (y_end - y_start) / 6.0;

        tr.source_mut().source_mut().remove_all();
        tr.source_mut().source_mut().move_to(x_start, ys);
        tr.source_mut().source_mut().line_to(x_end, ys);

        ras.add_path(&mut tr, 0);
        rs.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);

        tr.source_mut().set_width(1.0);

        for i in 0..self.num_filters {
            if self.filters[i as usize].borrow().status() {
                self.filter_func[i as usize].set_radius(self.radius.borrow().value());
                let radius = self.filter_func[i as usize].radius();
                let n = (radius * 256.0 * 2.0) as u32;
                let dy = y_end - ys;

                let mut xs = (x_end + x_start) / 2.0 - (radius * (x_end - x_start) / 16.0);
                let dx = (x_end - x_start) * radius / 8.0;

                tr.source_mut().source_mut().remove_all();
                tr.source_mut().source_mut().move_to(
                    xs + 0.5,
                    ys + dy * self.filter_func[i as usize].calc_weight((-radius).abs()),
                );
                for j in 1..n {
                    tr.source_mut().source_mut().line_to(
                        xs + dx * j as f64 / n as f64 + 0.5,
                        ys + dy
                            * self.filter_func[i as usize]
                                .calc_weight((j as f64 / 256.0 - radius).abs()),
                    );
                }

                ras.add_path(&mut tr, 0);
                rs.set_color(agg::Rgba8::new_params(100, 0, 0, 255));
                agg::render_scanlines(&mut ras, &mut sl, &mut rs);

                tr.source_mut().source_mut().remove_all();

                let ir = (radius.ceil() + 0.1) as i32;
                for xint in 0..256 {
                    let mut sum = 0.0;
                    for xfract in -ir..ir {
                        let xf = xint as f64 / 256.0 + xfract as f64;
                        if xf >= -radius || xf <= radius {
                            sum += self.filter_func[i as usize].calc_weight(xf.abs());
                        }
                    }
                    let x = x_center
                        + ((-128.0 + xint as f64) / 128.0) * radius * (x_end - x_start) / 16.0;
                    let y = ys + sum * 256.0 - 256.0;

                    if xint == 0 {
                        tr.source_mut().source_mut().move_to(x, y);
                    } else {
                        tr.source_mut().source_mut().line_to(x, y);
                    }
                }
                ras.add_path(&mut tr, 0);
                rs.set_color(agg::Rgba8::new_params(0, 100, 0, 255));
                agg::render_scanlines(&mut ras, &mut sl, &mut rs);

                let normalized =
                    ImageFilterLut::new_filter_dyn(self.filter_func[i as usize].as_ref(), true);
                let weights = normalized.weight_array();

                xs = (x_end + x_start) / 2.0
                    - (normalized.diameter() as f64 * (x_end - x_start) / 32.0);
                let nn = normalized.diameter() * 256;

                tr.source_mut().source_mut().remove_all();
                tr.source_mut().source_mut().move_to(
                    xs + 0.5,
                    ys + dy * weights[0] as f64 / ImageFilterScale::Scale as i32 as f64,
                );
                for j in 1..nn {
                    tr.source_mut().source_mut().line_to(
                        xs + dx * j as f64 / n as f64 + 0.5,
                        ys + dy * weights[j as usize] as f64
                            / ImageFilterScale::Scale as i32 as f64,
                    );
                }
                ras.add_path(&mut tr, 0);
                rs.set_color(agg::Rgba8::new_params(0, 0, 100, 255));
                agg::render_scanlines(&mut ras, &mut sl, &mut rs);
            }
        }

        for i in 0..self.num_filters as usize {
            ctrl::render_ctrl(
                &mut ras,
                &mut sl,
                &mut rb,
                &mut *self.filters[i].borrow_mut(),
            );
        }

        if self.sinc.borrow().status()
            || self.lanczos.borrow().status()
            || self.blackman.borrow().status()
        {
            ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.radius.borrow_mut());
        }
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("Image filters' shape comparison");

    if plat.init(780, 300, WindowFlag::Resize as u32) {
        plat.run();
    }
}

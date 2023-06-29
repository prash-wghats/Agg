use crate::platform::*;

use agg::rendering_buffer::RenderBuf;

use agg::{Color, ColorFn, Distortion, Interpolator, RasterScanLine, RenderBuffer, Transformer};

use crate::ctrl::rbox::*;

mod ctrl;
mod platform;
use crate::ctrl::slider::Slider;

use core::f64::consts::PI;
use core::ops::Deref;
use core::ops::Index;
use core::ops::IndexMut;
use std::cell::RefCell;

use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

const IMG_SCALE: f64 = agg::ImageSubpixelScale::Scale as i32 as f64;
struct PeriodicDistortion {
    cx: f64,
    cy: f64,
    period: f64,
    amplitude: f64,
    phase: f64,
}

impl PeriodicDistortion {
    fn new() -> Self {
        PeriodicDistortion {
            cx: 0.0,
            cy: 0.0,
            period: 0.5,
            amplitude: 0.5,
            phase: 0.0,
        }
    }

    fn center(&mut self, x: f64, y: f64) {
        self.cx = x;
        self.cy = y;
    }

    fn period(&mut self, v: f64) {
        self.period = v;
    }

    fn amplitude(&mut self, v: f64) {
        self.amplitude = 1.0 / v;
    }

    fn phase(&mut self, v: f64) {
        self.phase = v;
    }
}

fn calculate_wave(
    x: &mut i32, y: &mut i32, cx: f64, cy: f64, period: f64, amplitude: f64, phase: f64,
) {
    let xd = *x as f64 / IMG_SCALE - cx;
    let yd = *y as f64 / IMG_SCALE - cy;
    let d = (xd * xd + yd * yd).sqrt();
    if d > 1.0 {
        let a = (d / (16.0 * period) - phase).cos() * (1.0 / (amplitude * d)) + 1.0;
        *x = ((xd * a + cx) * IMG_SCALE) as i32;
        *y = ((yd * a + cy) * IMG_SCALE) as i32;
    }
}

fn calculate_swirl(x: &mut i32, y: &mut i32, cx: f64, cy: f64, amplitude: f64, phase: f64) {
    let xd = *x as f64 / IMG_SCALE - cx;
    let yd = *y as f64 / IMG_SCALE - cy;
    let a = (100.0 - (xd * xd + yd * yd).sqrt()) / 100.0 * (0.1 / -amplitude);
    let sa = (a - phase / 25.0).sin();
    let ca = (a - phase / 25.0).cos();
    *x = ((xd * ca - yd * sa + cx) * IMG_SCALE) as i32;
    *y = ((xd * sa + yd * ca + cy) * IMG_SCALE) as i32;
}

struct DistortionWave {
    distortion: PeriodicDistortion,
}
impl Deref for DistortionWave {
    type Target = PeriodicDistortion;
    fn deref(&self) -> &PeriodicDistortion {
        &self.distortion
    }
}

impl DistortionWave {
    fn new() -> Self {
        DistortionWave {
            distortion: PeriodicDistortion::new(),
        }
    }
}

impl Distortion for DistortionWave {
    fn calculate(&self, x: &mut i32, y: &mut i32) {
        calculate_wave(
            x,
            y,
            self.distortion.cx,
            self.distortion.cy,
            self.distortion.period,
            self.distortion.amplitude,
            self.distortion.phase,
        );
    }
}

struct DistortionSwirl {
    distortion: PeriodicDistortion,
}

impl DistortionSwirl {
    fn new() -> Self {
        DistortionSwirl {
            distortion: PeriodicDistortion::new(),
        }
    }
}

impl Deref for DistortionSwirl {
    type Target = PeriodicDistortion;
    fn deref(&self) -> &PeriodicDistortion {
        &self.distortion
    }
}

impl Distortion for DistortionSwirl {
    fn calculate(&self, x: &mut i32, y: &mut i32) {
        calculate_swirl(
            x,
            y,
            self.distortion.cx,
            self.distortion.cy,
            self.distortion.amplitude,
            self.distortion.phase,
        );
    }
}

struct DistortionSwirlWave {
    distortion: PeriodicDistortion,
}

impl DistortionSwirlWave {
    fn new() -> Self {
        DistortionSwirlWave {
            distortion: PeriodicDistortion::new(),
        }
    }
}

impl Deref for DistortionSwirlWave {
    type Target = PeriodicDistortion;
    fn deref(&self) -> &PeriodicDistortion {
        &self.distortion
    }
}

impl Distortion for DistortionSwirlWave {
    fn calculate(&self, x: &mut i32, y: &mut i32) {
        calculate_swirl(
            x,
            y,
            self.distortion.cx,
            self.distortion.cy,
            self.distortion.amplitude,
            self.distortion.phase,
        );
        calculate_wave(
            x,
            y,
            self.cx,
            self.cy,
            self.period,
            self.amplitude,
            self.phase,
        );
    }
}

struct DistortionWaveSwirl {
    distortion: PeriodicDistortion,
}

impl DistortionWaveSwirl {
    fn new() -> Self {
        DistortionWaveSwirl {
            distortion: PeriodicDistortion::new(),
        }
    }
}

impl Deref for DistortionWaveSwirl {
    type Target = PeriodicDistortion;
    fn deref(&self) -> &PeriodicDistortion {
        &self.distortion
    }
}

impl Distortion for DistortionWaveSwirl {
    fn calculate(&self, x: &mut i32, y: &mut i32) {
        calculate_wave(
            x,
            y,
            self.cx,
            self.cy,
            self.period,
            self.amplitude,
            self.phase,
        );
        calculate_swirl(
            x,
            y,
            self.distortion.cx,
            self.distortion.cy,
            self.distortion.amplitude,
            self.distortion.phase,
        );
    }
}

struct ColorFunc<C: Color>([C; 256]);
impl<C: Color> ColorFunc<C> {
    pub fn new() -> Self {
        Self([C::new(); 256])
    }
}

impl<C: Color> ColorFn<C> for ColorFunc<C> {
    fn size(&self) -> u32 {
        self.0.len() as u32
    }
    fn get(&mut self, i: u32) -> C {
        self.0[i as usize]
    }
}

impl<C: Color> Index<usize> for ColorFunc<C> {
    type Output = C;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}
impl<C: Color> IndexMut<usize> for ColorFunc<C> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

struct Application {
    angle: Ptr<Slider<'static, agg::Rgba8>>,
    scale: Ptr<Slider<'static, agg::Rgba8>>,
    amplitude: Ptr<Slider<'static, agg::Rgba8>>,
    period: Ptr<Slider<'static, agg::Rgba8>>,
    distortion: Ptr<Rbox<'static, agg::Rgba8>>,
    center_x: f64,
    center_y: f64,
    phase: f64,
    gradient_colors: ColorFunc<agg::Rgba8>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn render_dist<D: Distortion>(&mut self, mut dis: D, rbuf: &mut agg::RenderBuf) {
        let img_width = self.util.borrow().rbuf_img(0).width() as f64;
        let img_height = self.util.borrow().rbuf_img(0).height() as f64;
        let pdis = &mut dis as *const D as *mut PeriodicDistortion;
        let dist = unsafe { &mut *pdis };
        let mut pixf = agg::PixBgr24::new_borrowed(rbuf);
        //let mut tmp = *self.util.borrow_mut().rbuf_img_mut(0);
        let mut img_pixf = agg::PixBgr24::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());

        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pixf);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut src_mtx = agg::TransAffine::new_default();
        src_mtx *= agg::TransAffine::trans_affine_translation(-img_width / 2.0, -img_height / 2.0);
        src_mtx *=
            agg::TransAffine::trans_affine_rotation(self.angle.borrow().value() * PI / 180.0);
        src_mtx *= agg::TransAffine::trans_affine_translation(
            img_width / 2.0 + 10.0,
            img_height / 2.0 + 10.0 + 40.0,
        );
        src_mtx *= *self.util.borrow().trans_affine_resizing();

        let mut img_mtx = agg::TransAffine::new_default();
        img_mtx *= agg::TransAffine::trans_affine_translation(-img_width / 2.0, -img_height / 2.0);
        img_mtx *=
            agg::TransAffine::trans_affine_rotation(self.angle.borrow().value() * PI / 180.0);
        img_mtx *= agg::TransAffine::trans_affine_scaling_eq(self.scale.borrow().value());
        img_mtx *= agg::TransAffine::trans_affine_translation(
            img_width / 2.0 + 10.0,
            img_height / 2.0 + 10.0 + 40.0,
        );
        img_mtx *= *self.util.borrow().trans_affine_resizing();
        img_mtx.invert();

        let mut sa = agg::VecSpan::<ColorType>::new();

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineU8::new();

        dist.period(self.period.borrow().value());
        dist.amplitude(self.amplitude.borrow().value());
        dist.phase(self.phase);
        let mut cx = self.center_x;
        let mut cy = self.center_y;
        img_mtx.transform(&mut cx, &mut cy);
        dist.center(cx, cy);

        let mut interpolator: agg::SpanIpAdaptor<_, _> =
            agg::SpanIpAdaptor::new(agg::SpanIpLinear::<_>::new(img_mtx), dis);
        //let img_src = agg::ImageAccessorClip::new(img_pixf, &agg::Rgba8::new_params(255, 255, 255, 255));

        let mut sg: agg::SpanImageFilterRgbBilinearClip<_, _, _> =
            agg::SpanImageFilterRgbBilinearClip::new(
                &mut img_pixf,
                &mut interpolator,
                agg::Rgba8::new_params(255, 255, 255, 255),
            );

        /*let mut sg = agg::SpanImageFilterRgbBilinearClip::new(
            img_src,
            interpolator,
            agg::ImageFilterLut::new_filter(&ImageFilterSpline36::new(), true),
        );*/

        let mut r = img_width;
        if img_height < r {
            r = img_height;
        }
        let mut ell = agg::Ellipse::new_ellipse(
            img_width / 2.0,
            img_height / 2.0,
            r / 2.0 - 20.0,
            r / 2.0 - 20.0,
            200,
            false,
        );

        {
            let mut tr = agg::ConvTransform::new_borrowed(&mut ell, src_mtx);

            ras.add_path(&mut tr, 0);
            agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb, &mut sa, &mut sg);

            let mut t = *self.util.borrow_mut().trans_affine_resizing();
            t.invert();
            *tr.trans_mut() *= t;
            *tr.trans_mut() *=
                agg::TransAffine::trans_affine_translation(img_width - img_width / 10.0, 0.0);
            *tr.trans_mut() *= *self.util.borrow().trans_affine_resizing();

            ras.add_path(&mut tr, 0);
            agg::render_scanlines_aa_solid(
                &mut ras,
                &mut sl,
                &mut rb,
                &agg::Rgba8::new_params(0, 0, 0, 255),
            );
        }

        let mut sa = agg::VecSpan::<ColorType>::new();
        let mut gradient_func = agg::GradientCircle {};
        let mut span_gradient = agg::SpanGradient::<
            agg::Rgba8,
            agg::SpanIpAdaptor<agg::SpanIpLinear<agg::TransAffine>, _>,
            agg::GradientCircle,
            ColorFunc<agg::Rgba8>,
        >::new(
            &mut interpolator,
            &mut gradient_func,
            &mut self.gradient_colors,
            0.,
            180.,
        );

        let mut gr1_mtx = agg::TransAffine::new_default();
        gr1_mtx *= agg::TransAffine::trans_affine_translation(-img_width / 2.0, -img_height / 2.0);
        gr1_mtx *= agg::TransAffine::trans_affine_scaling_eq(0.8);
        gr1_mtx *=
            agg::TransAffine::trans_affine_rotation(self.angle.borrow().value() * PI / 180.0);
        gr1_mtx *= agg::TransAffine::trans_affine_translation(
            img_width - img_width / 10.0 + img_width / 2.0 + 10.0,
            img_height / 2.0 + 10.0 + 40.0,
        );
        gr1_mtx *= *self.util.borrow().trans_affine_resizing();

        let mut gr2_mtx = agg::TransAffine::new_default();
        gr2_mtx *=
            agg::TransAffine::trans_affine_rotation(self.angle.borrow().value() * PI / 180.0);
        gr2_mtx *= agg::TransAffine::trans_affine_scaling_eq(self.scale.borrow().value());
        gr2_mtx *= agg::TransAffine::trans_affine_translation(
            img_width - img_width / 10.0 + img_width / 2.0 + 10.0 + 50.0,
            img_height / 2.0 + 10.0 + 40.0 + 50.0,
        );
        gr2_mtx *= *self.util.borrow().trans_affine_resizing();
        gr2_mtx.invert();

        cx = self.center_x + img_width - img_width / 10.;
        cy = self.center_y;
        gr2_mtx.transform(&mut cx, &mut cy);
        unsafe {
            (*(span_gradient.interpolator_mut().distortion_mut() as *const D
                as *mut PeriodicDistortion))
                .center(cx, cy);
        }
        span_gradient.interpolator_mut().set_transformer(gr2_mtx);

        let mut tr2 = agg::ConvTransform::new_owned(ell, gr1_mtx);

        ras.add_path(&mut tr2, 0);
        agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb, &mut sa, &mut span_gradient);

        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.angle.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.scale.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.amplitude.borrow_mut(),
        );
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.period.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.distortion.borrow_mut(),
        );
    }
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let mut angle = Slider::new(5., 5., 150., 12., !flip_y);
        let mut scale = Slider::new(5., 5. + 15., 150., 12. + 15., !flip_y);
        let mut period = Slider::new(5. + 170., 5., 150. + 170., 12., !flip_y);
        let mut amplitude = Slider::new(5. + 170., 5. + 15., 150. + 170., 12. + 15., !flip_y);
        let mut distortion = Rbox::new(480., 5., 600., 90., !flip_y);

        angle.set_label("Angle=%3.2f");
        scale.set_label("Scale=%3.2f");
        angle.set_range(-180.0, 180.0);
        angle.set_value(20.0);
        scale.set_range(0.1, 5.0);
        scale.set_value(1.0);

        amplitude.set_label("Amplitude=%3.2f");
        period.set_label("Period=%3.2f");
        amplitude.set_range(0.1, 40.0);
        period.set_range(0.1, 2.0);
        amplitude.set_value(10.0);
        period.set_value(1.0);

        distortion.add_item("Wave");
        distortion.add_item("Swirl");
        distortion.add_item("Wave-Swirl");
        distortion.add_item("Swirl-Wave");
        distortion.set_cur_item(0);

        let angle = ctrl_ptr(angle);
        let scale = ctrl_ptr(scale);
        let period = ctrl_ptr(period);
        let amplitude = ctrl_ptr(amplitude);
        let distortion = ctrl_ptr(distortion);
        Application {
            angle: angle.clone(),
            scale: scale.clone(),
            period: period.clone(),
            amplitude: amplitude.clone(),
            distortion: distortion.clone(),
            center_x: 0.0,
            center_y: 0.0,
            phase: 0.0,
            // create a array of colors
            gradient_colors: {
                let mut colors = ColorFunc::new();
                let p = GRADIENT_COLOR;
                for i in 0..256 {
                    let j = i * 4;
                    colors[i] = agg::Rgba8::new_params(
                        p[j] as u32,
                        p[j + 1] as u32,
                        p[j + 2] as u32,
                        p[j + 3] as u32,
                    );
                }
                colors
            },
            ctrls: CtrlContainer {
                ctrl: vec![angle, scale, period, amplitude, distortion],
                cur_ctrl: -1,
                num_ctrl: 5,
            },
            util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_init(&mut self) {
        self.center_x = self.util.borrow().rbuf_img(0).width() as f64 / 2.0 + 10.0;
        self.center_y = self.util.borrow().rbuf_img(0).height() as f64 / 2.0 + 10. + 40.;
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags != 0 {
            self.center_x = x as f64;
            self.center_y = y as f64;
            return Draw::Yes;
        }
        return Draw::No;
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if (flags & 1) != 0 {
            self.center_x = x as f64;
            self.center_y = y as f64;
            return Draw::Yes;
        }
        return Draw::No;
    }

    fn on_idle(&mut self) -> Draw {
        self.phase += 15.0 * PI / 180.0;
        if self.phase > PI * 200.0 {
            self.phase -= PI * 200.0;
        }
        Draw::Yes
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let dist_wave = DistortionWave::new();
        let dist_swirl = DistortionSwirl::new();
        let dist_wave_swirl = DistortionWaveSwirl::new();
        let dist_swirl_wave = DistortionSwirlWave::new();
        let it = self.distortion.borrow().cur_item();
        match it {
            0 => self.render_dist(dist_wave, rbuf),
            1 => self.render_dist(dist_swirl, rbuf),
            2 => self.render_dist(dist_wave_swirl, rbuf),
            3 => self.render_dist(dist_swirl_wave, rbuf),
            _ => (),
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "spheres";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);

    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("spheres") {
			let ext = (plat.app_mut().util.borrow().img_ext()).to_string();
            buf = format!(
                "File not found: {}{}. Download http://www.antigrain.com/{}{}
				or copy it from another directory if available.",
                img_name, ext, img_name, ext
            );
        } else {
            buf = format!("File not found: {}", img_name);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
    }

    plat.set_caption("Image and Gradient Distortions");

    let w = plat.app().util.borrow().rbuf_img(0).width();
    let h = plat.app().util.borrow().rbuf_img(0).height();
    //if plat.init(600, 400, WindowFlag::Resize as u32) {
    if plat.init(w + 300, h + 40 + 20, WindowFlag::Resize as u32) {
        plat.app().util.borrow_mut().set_wait_mode(false);
        plat.run();
    }
}

const GRADIENT_COLOR: [u8; 1024] = [
    255, 255, 255, 255, 255, 255, 254, 255, 255, 255, 254, 255, 255, 255, 254, 255, 255, 255, 253,
    255, 255, 255, 253, 255, 255, 255, 252, 255, 255, 255, 251, 255, 255, 255, 250, 255, 255, 255,
    248, 255, 255, 255, 246, 255, 255, 255, 244, 255, 255, 255, 241, 255, 255, 255, 238, 255, 255,
    255, 235, 255, 255, 255, 231, 255, 255, 255, 227, 255, 255, 255, 222, 255, 255, 255, 217, 255,
    255, 255, 211, 255, 255, 255, 206, 255, 255, 255, 200, 255, 255, 254, 194, 255, 255, 253, 188,
    255, 255, 252, 182, 255, 255, 250, 176, 255, 255, 249, 170, 255, 255, 247, 164, 255, 255, 246,
    158, 255, 255, 244, 152, 255, 254, 242, 146, 255, 254, 240, 141, 255, 254, 238, 136, 255, 254,
    236, 131, 255, 253, 234, 126, 255, 253, 232, 121, 255, 253, 229, 116, 255, 252, 227, 112, 255,
    252, 224, 108, 255, 251, 222, 104, 255, 251, 219, 100, 255, 251, 216, 96, 255, 250, 214, 93,
    255, 250, 211, 89, 255, 249, 208, 86, 255, 249, 205, 83, 255, 248, 202, 80, 255, 247, 199, 77,
    255, 247, 196, 74, 255, 246, 193, 72, 255, 246, 190, 69, 255, 245, 187, 67, 255, 244, 183, 64,
    255, 244, 180, 62, 255, 243, 177, 60, 255, 242, 174, 58, 255, 242, 170, 56, 255, 241, 167, 54,
    255, 240, 164, 52, 255, 239, 161, 51, 255, 239, 157, 49, 255, 238, 154, 47, 255, 237, 151, 46,
    255, 236, 147, 44, 255, 235, 144, 43, 255, 235, 141, 41, 255, 234, 138, 40, 255, 233, 134, 39,
    255, 232, 131, 37, 255, 231, 128, 36, 255, 230, 125, 35, 255, 229, 122, 34, 255, 228, 119, 33,
    255, 227, 116, 31, 255, 226, 113, 30, 255, 225, 110, 29, 255, 224, 107, 28, 255, 223, 104, 27,
    255, 222, 101, 26, 255, 221, 99, 25, 255, 220, 96, 24, 255, 219, 93, 23, 255, 218, 91, 22, 255,
    217, 88, 21, 255, 216, 86, 20, 255, 215, 83, 19, 255, 214, 81, 18, 255, 213, 79, 17, 255, 212,
    77, 17, 255, 211, 74, 16, 255, 210, 72, 15, 255, 209, 70, 14, 255, 207, 68, 13, 255, 206, 66,
    13, 255, 205, 64, 12, 255, 204, 62, 11, 255, 203, 60, 10, 255, 202, 58, 10, 255, 201, 56, 9,
    255, 199, 55, 9, 255, 198, 53, 8, 255, 197, 51, 7, 255, 196, 50, 7, 255, 195, 48, 6, 255, 193,
    46, 6, 255, 192, 45, 5, 255, 191, 43, 5, 255, 190, 42, 4, 255, 188, 41, 4, 255, 187, 39, 3,
    255, 186, 38, 3, 255, 185, 37, 2, 255, 183, 35, 2, 255, 182, 34, 1, 255, 181, 33, 1, 255, 179,
    32, 1, 255, 178, 30, 0, 255, 177, 29, 0, 255, 175, 28, 0, 255, 174, 27, 0, 255, 173, 26, 0,
    255, 171, 25, 0, 255, 170, 24, 0, 255, 168, 23, 0, 255, 167, 22, 0, 255, 165, 21, 0, 255, 164,
    21, 0, 255, 163, 20, 0, 255, 161, 19, 0, 255, 160, 18, 0, 255, 158, 17, 0, 255, 156, 17, 0,
    255, 155, 16, 0, 255, 153, 15, 0, 255, 152, 14, 0, 255, 150, 14, 0, 255, 149, 13, 0, 255, 147,
    12, 0, 255, 145, 12, 0, 255, 144, 11, 0, 255, 142, 11, 0, 255, 140, 10, 0, 255, 139, 10, 0,
    255, 137, 9, 0, 255, 135, 9, 0, 255, 134, 8, 0, 255, 132, 8, 0, 255, 130, 7, 0, 255, 128, 7, 0,
    255, 126, 6, 0, 255, 125, 6, 0, 255, 123, 5, 0, 255, 121, 5, 0, 255, 119, 4, 0, 255, 117, 4, 0,
    255, 115, 4, 0, 255, 113, 3, 0, 255, 111, 3, 0, 255, 109, 2, 0, 255, 107, 2, 0, 255, 105, 2, 0,
    255, 103, 1, 0, 255, 101, 1, 0, 255, 99, 1, 0, 255, 97, 0, 0, 255, 95, 0, 0, 255, 93, 0, 0,
    255, 91, 0, 0, 255, 90, 0, 0, 255, 88, 0, 0, 255, 86, 0, 0, 255, 84, 0, 0, 255, 82, 0, 0, 255,
    80, 0, 0, 255, 78, 0, 0, 255, 77, 0, 0, 255, 75, 0, 0, 255, 73, 0, 0, 255, 72, 0, 0, 255, 70,
    0, 0, 255, 68, 0, 0, 255, 67, 0, 0, 255, 65, 0, 0, 255, 64, 0, 0, 255, 63, 0, 0, 255, 61, 0, 0,
    255, 60, 0, 0, 255, 59, 0, 0, 255, 58, 0, 0, 255, 57, 0, 0, 255, 56, 0, 0, 255, 55, 0, 0, 255,
    54, 0, 0, 255, 53, 0, 0, 255, 53, 0, 0, 255, 52, 0, 0, 255, 52, 0, 0, 255, 51, 0, 0, 255, 51,
    0, 0, 255, 51, 0, 0, 255, 50, 0, 0, 255, 50, 0, 0, 255, 51, 0, 0, 255, 51, 0, 0, 255, 51, 0, 0,
    255, 51, 0, 0, 255, 52, 0, 0, 255, 52, 0, 0, 255, 53, 0, 0, 255, 54, 1, 0, 255, 55, 2, 0, 255,
    56, 3, 0, 255, 57, 4, 0, 255, 58, 5, 0, 255, 59, 6, 0, 255, 60, 7, 0, 255, 62, 8, 0, 255, 63,
    9, 0, 255, 64, 11, 0, 255, 66, 12, 0, 255, 68, 13, 0, 255, 69, 14, 0, 255, 71, 16, 0, 255, 73,
    17, 0, 255, 75, 18, 0, 255, 77, 20, 0, 255, 79, 21, 0, 255, 81, 23, 0, 255, 83, 24, 0, 255, 85,
    26, 0, 255, 87, 28, 0, 255, 90, 29, 0, 255, 92, 31, 0, 255, 94, 33, 0, 255, 97, 34, 0, 255, 99,
    36, 0, 255, 102, 38, 0, 255, 104, 40, 0, 255, 107, 41, 0, 255, 109, 43, 0, 255, 112, 45, 0,
    255, 115, 47, 0, 255, 117, 49, 0, 255, 120, 51, 0, 255, 123, 52, 0, 255, 126, 54, 0, 255, 128,
    56, 0, 255, 131, 58, 0, 255, 134, 60, 0, 255, 137, 62, 0, 255, 140, 64, 0, 255, 143, 66, 0,
    255, 145, 68, 0, 255, 148, 70, 0, 255, 151, 72, 0, 255, 154, 74, 0, 255,
];

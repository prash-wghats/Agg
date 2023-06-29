use crate::platform::*;
use agg::{RasterScanLine, Transformer};

mod ctrl;
mod platform;

use crate::ctrl::cbox::Cbox;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::f64::consts::PI;

use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = false;

#[derive(Clone, Copy, Default)]
struct PathAttributes {
    index: u32,
    fill_color: agg::Rgba8,
    stroke_color: agg::Rgba8,
    stroke_width: f64,
}

impl PathAttributes {
    pub fn new_default() -> Self {
        PathAttributes {
            ..Default::default()
        }
    }
    pub fn new(idx: u32, fill: agg::Rgba8, stroke: agg::Rgba8, width: f64) -> PathAttributes {
        PathAttributes {
            index: idx,
            fill_color: fill,
            stroke_color: stroke,
            stroke_width: width,
        }
    }
}

static POLY_BULB: [f64; 30] = [
    -6.0, -67.0, -6.0, -71.0, -7.0, -74.0, -8.0, -76.0, -10.0, -79.0, -10.0, -82.0, -9.0, -84.0,
    -6.0, -86.0, -4.0, -87.0, -2.0, -86.0, -1.0, -86.0, 1.0, -84.0, 2.0, -82.0, 2.0, -79.0, 0.0,
    -77.0,
];

static POLY_BEAM1: [f64; 10] = [
    -14.0, -84.0, -22.0, -85.0, -23.0, -87.0, -22.0, -88.0, -21.0, -88.0,
];

static POLY_BEAM2: [f64; 10] = [
    -10.0, -92.0, -14.0, -96.0, -14.0, -98.0, -12.0, -99.0, -11.0, -97.0,
];

static POLY_BEAM3: [f64; 10] = [
    -1.0, -92.0, -2.0, -98.0, 0.0, -100.0, 2.0, -100.0, 1.0, -98.0,
];

static POLY_BEAM4: [f64; 10] = [
    5.0, -89.0, 11.0, -94.0, 13.0, -93.0, 13.0, -92.0, 12.0, -91.0,
];

static POLY_FIG1: [f64; 42] = [
    1.0, -48.0, -3.0, -54.0, -7.0, -58.0, -12.0, -58.0, -17.0, -55.0, -20.0, -52.0, -21.0, -47.0,
    -20.0, -40.0, -17.0, -33.0, -11.0, -28.0, -6.0, -26.0, -2.0, -25.0, 2.0, -26.0, 4.0, -28.0,
    5.0, -33.0, 5.0, -39.0, 3.0, -44.0, 12.0, -48.0, 12.0, -50.0, 12.0, -51.0, 3.0, -46.0,
];

static POLY_FIG2: [f64; 76] = [
    11.0, -27.0, 6.0, -23.0, 4.0, -22.0, 3.0, -19.0, 5.0, -16.0, 6.0, -15.0, 11.0, -17.0, 19.0,
    -23.0, 25.0, -30.0, 32.0, -38.0, 32.0, -41.0, 32.0, -50.0, 30.0, -64.0, 32.0, -72.0, 32.0,
    -75.0, 31.0, -77.0, 28.0, -78.0, 26.0, -80.0, 28.0, -87.0, 27.0, -89.0, 25.0, -88.0, 24.0,
    -79.0, 24.0, -76.0, 23.0, -75.0, 20.0, -76.0, 17.0, -76.0, 17.0, -74.0, 19.0, -73.0, 22.0,
    -73.0, 24.0, -71.0, 26.0, -69.0, 27.0, -64.0, 28.0, -55.0, 28.0, -47.0, 28.0, -40.0, 26.0,
    -38.0, 20.0, -33.0, 14.0, -30.0,
];

static POLY_FIG3: [f64; 70] = [
    -6.0, -20.0, -9.0, -21.0, -15.0, -21.0, -20.0, -17.0, -28.0, -8.0, -32.0, -1.0, -32.0, 1.0,
    -30.0, 6.0, -26.0, 8.0, -20.0, 10.0, -16.0, 12.0, -14.0, 14.0, -15.0, 16.0, -18.0, 20.0, -22.0,
    20.0, -25.0, 19.0, -27.0, 20.0, -26.0, 22.0, -23.0, 23.0, -18.0, 23.0, -14.0, 22.0, -11.0,
    20.0, -10.0, 17.0, -9.0, 14.0, -11.0, 11.0, -16.0, 9.0, -22.0, 8.0, -26.0, 5.0, -28.0, 2.0,
    -27.0, -2.0, -23.0, -8.0, -19.0, -11.0, -12.0, -14.0, -6.0, -15.0, -6.0, -18.0,
];

static POLY_FIG4: [f64; 40] = [
    11.0, -6.0, 8.0, -16.0, 5.0, -21.0, -1.0, -23.0, -7.0, -22.0, -10.0, -17.0, -9.0, -10.0, -8.0,
    0.0, -8.0, 10.0, -10.0, 18.0, -11.0, 22.0, -10.0, 26.0, -7.0, 28.0, -3.0, 30.0, 0.0, 31.0, 5.0,
    31.0, 10.0, 27.0, 14.0, 18.0, 14.0, 11.0, 11.0, 2.0,
];

const POLY_FIG5: [f64; 56] = [
    0.0, 22.0, -5.0, 21.0, -8.0, 22.0, -9.0, 26.0, -8.0, 49.0, -8.0, 54.0, -10.0, 64.0, -10.0,
    75.0, -9.0, 81.0, -10.0, 84.0, -16.0, 89.0, -18.0, 95.0, -18.0, 97.0, -13.0, 100.0, -12.0,
    99.0, -12.0, 95.0, -10.0, 90.0, -8.0, 87.0, -6.0, 86.0, -4.0, 83.0, -3.0, 82.0, -5.0, 80.0,
    -6.0, 79.0, -7.0, 74.0, -6.0, 63.0, -3.0, 52.0, 0.0, 42.0, 1.0, 31.0,
];

const POLY_FIG6: [f64; 62] = [
    12.0, 31.0, 12.0, 24.0, 8.0, 21.0, 3.0, 21.0, 2.0, 24.0, 3.0, 30.0, 5.0, 40.0, 8.0, 47.0, 10.0,
    56.0, 11.0, 64.0, 11.0, 71.0, 10.0, 76.0, 8.0, 77.0, 8.0, 79.0, 10.0, 81.0, 13.0, 82.0, 17.0,
    82.0, 26.0, 84.0, 28.0, 87.0, 32.0, 86.0, 33.0, 81.0, 32.0, 80.0, 25.0, 79.0, 17.0, 79.0, 14.0,
    79.0, 13.0, 76.0, 14.0, 72.0, 14.0, 64.0, 13.0, 55.0, 12.0, 44.0, 12.0, 34.0,
];

struct TransRoundoff;
impl Transformer for TransRoundoff {
    fn transform(&self, x: &mut f64, y: &mut f64) {
        *x = x.floor() + 0.5;
        *y = y.floor() + 0.5;
    }
    fn scaling_abs(&self, _x: &mut f64, _y: &mut f64) {}
}

struct Application {
    dx: f64,
    dy: f64,
    rotate: Ptr<Cbox<'static, agg::Rgba8>>,
    even_odd: Ptr<Cbox<'static, agg::Rgba8>>,
    draft: Ptr<Cbox<'static, agg::Rgba8>>,
    roundoff: Ptr<Cbox<'static, agg::Rgba8>>,
    angle_delta: Ptr<Slider<'static, agg::Rgba8>>,
    redraw_flag: bool,
    scanline: agg::ScanlineU8,
    rasterizer: agg::RasterizerScanlineAa,
    path: agg::PathStorage,
    npaths: u32,
    pflag: agg::FillingRule,
    angle: f64,
    attr: [PathAttributes; 3],
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let rotate = ctrl_ptr(Cbox::new(10.0, 3.0, "Rotate", !flip_y));
        let even_odd = ctrl_ptr(Cbox::new(60.0, 3.0, "Even-Odd", !flip_y));
        let draft = ctrl_ptr(Cbox::new(130.0, 3.0, "Draft", !flip_y));
        let roundoff = ctrl_ptr(Cbox::new(175.0, 3.0, "Roundoff", !flip_y));
        let angle_delta = ctrl_ptr(Slider::new(10.0, 21.0, 250.0 - 10.0, 27.0, !flip_y));

        angle_delta.borrow_mut().set_label("Step=%4.3f degree");
        rotate.borrow_mut().set_text_size(7., 0.);
        even_odd.borrow_mut().set_text_size(7., 0.);
        draft.borrow_mut().set_text_size(7., 0.);
        roundoff.borrow_mut().set_text_size(7., 0.);
        angle_delta.borrow_mut().set_value(0.01);

        let mut app = Self {
            dx: 0.0,
            dy: 0.0,
            rotate: rotate.clone(),
            even_odd: even_odd.clone(),
            draft: draft.clone(),
            roundoff: roundoff.clone(),
            angle_delta: angle_delta.clone(),
            redraw_flag: false,
            scanline: agg::ScanlineU8::new(),
            rasterizer: agg::RasterizerScanlineAa::new(),
            path: agg::PathStorage::new(),
            npaths: 0,
            pflag: agg::FillingRule::FillEvenOdd,
            angle: 0.,
            attr: [PathAttributes::new_default(); 3],
            ctrls: CtrlContainer {
                ctrl: vec![rotate, even_odd, draft, roundoff, angle_delta],
                cur_ctrl: -1,
                num_ctrl: 5,
            },
            util: util,
        };

        app.attr[app.npaths as usize] = PathAttributes::new(
            app.path.start_new_path(),
            agg::Rgba8::new_params(255, 255, 0, 255),
            agg::Rgba8::new_params(0, 0, 0, 255),
            1.0,
        );
        app.npaths += 1;

        app.path.concat_poly(&POLY_BULB, POLY_BULB.len() / 2, true);

        app.attr[app.npaths as usize] = PathAttributes::new(
            app.path.start_new_path(),
            agg::Rgba8::new_params(255, 255, 200, 255),
            agg::Rgba8::new_params(90, 0, 0, 255),
            0.7,
        );
        app.npaths += 1;

        app.path
            .concat_poly(&POLY_BEAM1, POLY_BEAM1.len() / 2, true);
        app.path
            .concat_poly(&POLY_BEAM2, POLY_BEAM2.len() / 2, true);
        app.path
            .concat_poly(&POLY_BEAM3, POLY_BEAM3.len() / 2, true);
        app.path
            .concat_poly(&POLY_BEAM4, POLY_BEAM4.len() / 2, true);

        app.attr[app.npaths as usize] = PathAttributes::new(
            app.path.start_new_path(),
            agg::Rgba8::new_params(0, 0, 0, 255),
            agg::Rgba8::new_params(0, 0, 0, 255),
            0.0,
        );
        app.npaths += 1;

        app.path.concat_poly(&POLY_FIG1, POLY_FIG1.len() / 2, true);
        app.path.concat_poly(&POLY_FIG2, POLY_FIG2.len() / 2, true);
        app.path.concat_poly(&POLY_FIG3, POLY_FIG3.len() / 2, true);
        app.path.concat_poly(&POLY_FIG4, POLY_FIG4.len() / 2, true);
        app.path.concat_poly(&POLY_FIG5, POLY_FIG5.len() / 2, true);
        app.path.concat_poly(&POLY_FIG6, POLY_FIG6.len() / 2, true);

        app
    }

    fn on_init(&mut self) {
        self.dx = self.util.borrow().width();
        self.dy = self.util.borrow().height();
    }

    fn on_resize(&mut self, _: i32, _: i32) {
        self.redraw_flag = true;
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgr24::new_borrowed(rbuf);
        let mut rbase = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pixf);

        let roundoff = TransRoundoff {};

        let width = self.util.borrow().width();
        let height = self.util.borrow().height();
        if self.redraw_flag {
            self.rasterizer.set_gamma(&agg::GammaNone::new());
            rbase.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
            self.rasterizer
                .set_filling_rule(agg::FillingRule::FillNonZero);
            ctrl::render_ctrl(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut rbase,
                &mut *self.rotate.borrow_mut(),
            );
            ctrl::render_ctrl(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut rbase,
                &mut *self.even_odd.borrow_mut(),
            );
            ctrl::render_ctrl(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut rbase,
                &mut *self.draft.borrow_mut(),
            );
            ctrl::render_ctrl(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut rbase,
                &mut *self.roundoff.borrow_mut(),
            );
            ctrl::render_ctrl(
                &mut self.rasterizer,
                &mut self.scanline,
                &mut rbase,
                &mut *self.angle_delta.borrow_mut(),
            );
            self.redraw_flag = false;
        } else {
            rbase.copy_bar(
                0,
                (32.0 * self.util.borrow().height() / self.dy) as i32,
                width as i32,
                height as i32,
                &agg::Rgba8::new_params(255, 255, 255, 255),
            );
        }

        if self.draft.borrow().status() {
            self.rasterizer
                .set_gamma(&agg::GammaThreshold::new_with_threshold(0.4));
        }

        let mut mtx = agg::TransAffine::new_default();
        mtx.reset();
        mtx *= agg::TransAffine::trans_affine_rotation(self.angle * PI / 180.0);
        mtx *= agg::TransAffine::trans_affine_translation(self.dx / 2.0, self.dy / 2.0 + 10.0);
        mtx *= agg::TransAffine::trans_affine_scaling(width / self.dx, height / self.dy);

        let mut fill = agg::ConvTransform::new_borrowed(&mut self.path, mtx);

        self.pflag = if self.even_odd.borrow().status() {
            agg::FillingRule::FillEvenOdd
        } else {
            agg::FillingRule::FillNonZero
        };

        let mut fill_roundoff = agg::ConvTransform::new_borrowed(&mut fill, roundoff);

        for i in 0..self.npaths as usize {
            self.rasterizer.set_filling_rule(self.pflag);
            if self.roundoff.borrow().status() {
                self.rasterizer
                    .add_path(&mut fill_roundoff, self.attr[i].index);
            } else {
                self.rasterizer
                    .add_path(fill_roundoff.source_mut(), self.attr[i].index);
            }

            if self.draft.borrow().status() {
                agg::render_scanlines_bin_solid(
                    &mut self.rasterizer,
                    &mut self.scanline,
                    &mut rbase,
                    &self.attr[i].fill_color,
                );
            } else {
                agg::render_scanlines_aa_solid(
                    &mut self.rasterizer,
                    &mut self.scanline,
                    &mut rbase,
                    &self.attr[i].fill_color,
                );
            }

            if self.attr[i].stroke_width > 0.001 {
                if self.roundoff.borrow().status() {
                    let mut stroke_roundoff: agg::ConvStroke<'_, _> =
                        agg::ConvStroke::new_borrowed(&mut fill_roundoff);
                    stroke_roundoff.set_width(self.attr[i].stroke_width * mtx.scale());
                    self.rasterizer
                        .add_path(&mut stroke_roundoff, self.attr[i].index);
                } else {
                    let mut stroke: agg::ConvStroke<'_, _> =
                        agg::ConvStroke::new_borrowed(fill_roundoff.source_mut());
                    stroke.set_width(self.attr[i].stroke_width * mtx.scale());
                    self.rasterizer.add_path(&mut stroke, self.attr[i].index);
                }
                if self.draft.borrow().status() {
                    agg::render_scanlines_bin_solid(
                        &mut self.rasterizer,
                        &mut self.scanline,
                        &mut rbase,
                        &self.attr[i].stroke_color,
                    );
                } else {
                    agg::render_scanlines_aa_solid(
                        &mut self.rasterizer,
                        &mut self.scanline,
                        &mut rbase,
                        &self.attr[i].stroke_color,
                    );
                }
            }
        }
    }

    fn on_idle(&mut self) -> Draw {
        self.angle += self.angle_delta.borrow().value();
        if self.angle > 360.0 {
            self.angle -= 360.0;
        }
        Draw::Yes
    }

    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {
        self.util
            .borrow_mut()
            .set_wait_mode(!self.rotate.borrow().status());
        self.redraw_flag = true;
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Idea");

    if plat.init(250, 280, WindowFlag::Resize as u32) {
        plat.run();
    }
}

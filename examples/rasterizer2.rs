use agg::basics::{deg2rad, PathCmd};
use agg::color_rgba::*;
use agg::rasterizer_outline_aa::OutlineAaJoin;

use agg::{
    ImagePattern, PatternFilter, Pixel, RasterScanLine, RenderPrim, RendererOutline,
    RendererScanlineColor, Transformer, VertexSource,
};

use crate::ctrl::cbox::Cbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;
mod misc;
use misc::pixel_formats::*;

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

static PIXMAP_CHAIN: [u32; 114] = [
    16, 7, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0xb4c29999, 0xff9a5757, 0xff9a5757,
    0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xb4c29999, 0x00ffffff, 0x00ffffff, 0x00ffffff,
    0x00ffffff, 0x00ffffff, 0x00ffffff, 0x0cfbf9f9, 0xff9a5757, 0xff660000, 0xff660000, 0xff660000,
    0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xb4c29999, 0x00ffffff, 0x00ffffff,
    0x00ffffff, 0x00ffffff, 0x5ae0cccc, 0xffa46767, 0xff660000, 0xff975252, 0x7ed4b8b8, 0x5ae0cccc,
    0x5ae0cccc, 0x5ae0cccc, 0x5ae0cccc, 0xa8c6a0a0, 0xff7f2929, 0xff670202, 0x9ecaa6a6, 0x5ae0cccc,
    0x00ffffff, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xa4c7a2a2,
    0x3affff00, 0x3affff00, 0xff975151, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000,
    0xff660000, 0x00ffffff, 0x5ae0cccc, 0xffa46767, 0xff660000, 0xff954f4f, 0x7ed4b8b8, 0x5ae0cccc,
    0x5ae0cccc, 0x5ae0cccc, 0x5ae0cccc, 0xa8c6a0a0, 0xff7f2929, 0xff670202, 0x9ecaa6a6, 0x5ae0cccc,
    0x00ffffff, 0x00ffffff, 0x00ffffff, 0x0cfbf9f9, 0xff9a5757, 0xff660000, 0xff660000, 0xff660000,
    0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xff660000, 0xb4c29999, 0x00ffffff, 0x00ffffff,
    0x00ffffff, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0x00ffffff, 0xb4c29999, 0xff9a5757, 0xff9a5757,
    0xff9a5757, 0xff9a5757, 0xff9a5757, 0xff9a5757, 0xb4c29999, 0x00ffffff, 0x00ffffff, 0x00ffffff,
    0x00ffffff,
];

struct PatternPixmapArgb32 {
    pixmap: *const u32,
}

impl PatternPixmapArgb32 {
    pub fn new(pixmap: *const u32) -> PatternPixmapArgb32 {
        PatternPixmapArgb32 { pixmap: pixmap }
    }
}
impl Pixel for PatternPixmapArgb32 {
    type ColorType = agg::Rgba8;

    fn width(&self) -> f64 {
        (unsafe { *self.pixmap }) as f64
    }

    fn height(&self) -> f64 {
        (unsafe { *self.pixmap.offset(1) }) as f64
    }

    fn pixel(&self, x: i32, y: i32) -> agg::Rgba8 {
        let index = (y * self.width() as i32 + x + 2) as usize;
        let p = unsafe { *self.pixmap.offset(index as isize) };

        agg::Rgba8::new_params((p >> 16) & 0xFF, (p >> 8) & 0xFF, p & 0xFF, p >> 24)
    }
}

struct Spiral {
    x: f64,
    y: f64,
    r1: f64,
    r2: f64,
    step: f64,
    start_angle: f64,
    angle: f64,
    da: f64,
    dr: f64,
    curr_r: f64,
    start: bool,
}

impl Spiral {
    pub fn new(x: f64, y: f64, r1: f64, r2: f64, step: f64, start_angle: f64) -> Spiral {
        let da = deg2rad(8.0);
        let dr = step / 45.0;

        Spiral {
            x: x,
            y: y,
            r1: r1,
            r2: r2,
            step: step,
            start_angle: start_angle,
            angle: start_angle,
            da: da,
            dr: dr,
            curr_r: 0.,
            start: false,
        }
    }
}
impl VertexSource for Spiral {
    fn rewind(&mut self, _: u32) {
        self.angle = self.start_angle;
        self.curr_r = self.r1;
        self.start = true;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.curr_r > self.r2 {
            return PathCmd::Stop as u32;
        }

        *x = self.x + self.angle.cos() * self.curr_r;
        *y = self.y + self.angle.sin() * self.curr_r;
        self.curr_r += self.dr;
        self.angle += self.da;
        if self.start {
            self.start = false;
            return PathCmd::MoveTo as u32;
        }
        return PathCmd::LineTo as u32;
    }
}

struct Roundoff;
impl Transformer for Roundoff {
    fn transform(&self, x: &mut f64, y: &mut f64) {
        *x = x.floor();
        *y = y.floor();
    }
    fn scaling_abs(&self, _x: &mut f64, _y: &mut f64) {}
}

type RB<'a> = agg::RendererBase<'a, Pixfmt<'a>>;
type RAA<'a, 'b> = agg::RendererScanlineAASolid<'a, RB<'b>>;
type RPrim<'a, 'b> = agg::RendererPrimitives<'a, RB<'b>>;
type RasO<'a, 'b> = agg::RasterizerOutline<'a, RPrim<'a, 'b>>;
type RasS = agg::RasterizerScanlineAa;
type SL = agg::ScanlineP8;
type ROAa<'a, 'b> = agg::RendererOutlineAa<'a, RB<'b>>;
type PF = agg::PatternFilterBilinearRgba8;
type IP<'a> = agg::LineImagePatternPow2<'a, PF>;
type ROI<'a, 'b> = agg::RendererOutlineImage<'a, RB<'b>, IP<'a>>;
type RasOAa<'a, 'b> = agg::RasterizerOutlineAa<'a, ROAa<'a, 'b>>;
type RasOI<'a, 'b> = agg::RasterizerOutlineAa<'a, ROI<'a, 'b>>;

struct Application {
    step: Ptr<Slider<'static, agg::Rgba8>>,
    width: Ptr<Slider<'static, agg::Rgba8>>,
    test: Ptr<Cbox<'static, agg::Rgba8>>,
    rotate: Ptr<Cbox<'static, agg::Rgba8>>,
    accurate_joins: Ptr<Cbox<'static, agg::Rgba8>>,
    scale_pattern: Ptr<Cbox<'static, agg::Rgba8>>,
    start_angle: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    pub fn draw_aliased_pix_accuracy(&self, ras: &mut RasO) {
        let s1 = Spiral::new(
            self.util.borrow().width() / 5.,
            self.util.borrow().height() / 4. + 50.,
            5.0,
            70.0,
            8.0,
            self.start_angle,
        );
        let rn = Roundoff {};
        let mut trans = agg::ConvTransform::new_owned(s1, rn);
        ras.ren_mut()
            .set_line_color(Rgba8::new_params(100, 75, 25, 255));
        ras.add_path(&mut trans, 0);
    }

    pub fn draw_aliased_subpix_accuracy(&self, ras: &mut RasO) {
        let mut s2 = Spiral::new(
            self.util.borrow().width() / 2.,
            self.util.borrow().height() / 4. + 50.,
            5.0,
            70.0,
            8.0,
            self.start_angle,
        );
        ras.ren_mut()
            .set_line_color(Rgba8::new_params(100, 75, 25, 255));
        ras.add_path(&mut s2, 0);
    }

    pub fn draw_anti_aliased_outline(&self, ras: &mut RasOAa) {
        let mut s3 = Spiral::new(
            self.util.borrow().width() / 5.,
            self.util.borrow().height() - self.util.borrow().height() / 4. + 20.,
            5.0,
            70.0,
            8.0,
            self.start_angle,
        );
        ras.ren_mut().set_color(Rgba8::new_params(100, 75, 25, 255));
        ras.add_path(&mut s3, 0);
    }

    pub fn draw_anti_aliased_scanline(&self, ras: &mut RasS, sl: &mut SL, ren: &mut RAA) {
        let s4 = Spiral::new(
            self.util.borrow().width() / 2.,
            self.util.borrow().height() - self.util.borrow().height() / 4. + 20.,
            5.,
            70.,
            8.,
            self.start_angle,
        );
        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(s4);
        stroke.set_width(self.width.borrow().value());
        stroke.set_line_cap(agg::math_stroke::LineCap::Round);
        ren.set_color(Rgba8::new_params(100, 75, 25, 255));
        ras.add_path(&mut stroke, 0);
        agg::render_scanlines(ras, sl, ren);
    }

    pub fn draw_anti_aliased_outline_img(&self, ras: &mut RasOI) {
        let mut s5 = Spiral::new(
            self.util.borrow().width() - self.util.borrow().width() / 5.,
            self.util.borrow().height() - self.util.borrow().height() / 4. + 20.,
            5.,
            70.,
            8.,
            self.start_angle,
        );
        ras.add_path(&mut s5, 0);
    }

    pub fn text(&self, ras: &mut RasS, sl: &mut SL, ren: &mut RAA, x: f64, y: f64, txt: &str) {
        let mut t = agg::GsvText::new();
        t.set_size(8., 0.);
        t.set_text(txt);
        t.set_start_point(x, y);
        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(t);
        stroke.set_width(0.7);
        ras.add_path(&mut stroke, 0);
        ren.set_color(Rgba8::new_params(0, 0, 0, 255));
        agg::render_scanlines(ras, sl, ren);
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let step = ctrl_ptr(Slider::new(
            10.0,
            10.0 + 4.0,
            150.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let width = ctrl_ptr(Slider::new(
            150.0 + 10.0,
            10.0 + 4.0,
            400. - 10.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let test = ctrl_ptr(Cbox::new(
            10.0,
            10.0 + 4.0 + 16.0,
            "Test Performance",
            !flip_y,
        ));
        let rotate = ctrl_ptr(Cbox::new(130. + 10.0, 10.0 + 4.0 + 16.0, "Rotate", !flip_y));
        let accurate_joins = ctrl_ptr(Cbox::new(
            200. + 10.0,
            10.0 + 4.0 + 16.0,
            "Accurate Joins",
            !flip_y,
        ));
        let scale_pattern = ctrl_ptr(Cbox::new(
            310. + 10.0,
            10.0 + 4.0 + 16.0,
            "Scale Pattern",
            !flip_y,
        ));

        step.borrow_mut().set_range(0.0, 2.0);
        step.borrow_mut().set_value(0.1);
        step.borrow_mut().set_label("Step=%1.2f");
        step.borrow_mut().no_transform();

        width.borrow_mut().set_range(0.0, 7.0);
        width.borrow_mut().set_value(3.0);
        width.borrow_mut().set_label("Width=%1.2f");
        width.borrow_mut().no_transform();

        test.borrow_mut().set_text_size(9.0, 7.0);
        test.borrow_mut().no_transform();

        rotate.borrow_mut().set_text_size(9.0, 7.0);
        rotate.borrow_mut().no_transform();

        accurate_joins.borrow_mut().set_text_size(9.0, 7.0);
        accurate_joins.borrow_mut().no_transform();

        scale_pattern.borrow_mut().set_text_size(9.0, 7.0);
        scale_pattern.borrow_mut().no_transform();
        scale_pattern.borrow_mut().set_status(true);

        Application {
            step: step.clone(),
            width: width.clone(),
            test: test.clone(),
            rotate: rotate.clone(),
            accurate_joins: accurate_joins.clone(),
            scale_pattern: scale_pattern.clone(),
            start_angle: 0.0,
            ctrls: CtrlContainer {
                ctrl: vec![step, width, test, rotate, accurate_joins, scale_pattern],
                cur_ctrl: -1,
                num_ctrl: 6,
            },
            util: util,
        }
    }

    fn on_idle(&mut self) -> Draw {
        self.start_angle += deg2rad(self.step.borrow().value());
        if self.start_angle > deg2rad(360.0) {
            self.start_angle -= deg2rad(360.0);
        }
        Draw::Yes
        //return true
    }

    fn on_ctrl_change(&mut self, rb: &mut agg::RenderBuf) {
        self.util
            .borrow_mut()
            .set_wait_mode(!self.rotate.borrow().status());

        if self.test.borrow().status() {
            let (t2, t3, t4, t5): (f64, f64, f64, f64);
            self.on_draw(rb);
            //self.update_window();

            let mut pf = Pixfmt::new_borrowed(rb);
            let mut ren_base = RB::new_borrowed(&mut pf);
            ren_base.clear(&Rgba8::new_params(255, 255, 240, 255));

            {
                let mut ren_prim = RPrim::new(&mut ren_base);
                let mut ras_al = RasO::new(&mut ren_prim);
                self.util.borrow_mut().start_timer();

                for _i in 0..200 {
                    self.draw_aliased_subpix_accuracy(&mut ras_al); //4
                    self.start_angle += deg2rad(self.step.borrow().value());
                }
                t2 = self.util.borrow_mut().elapsed_time();
            }

            {
                let mut prof = agg::LineProfileAA::new();
                prof.set_width(self.width.borrow().value());
                let mut ren_oaa = ROAa::new(&mut ren_base, prof);
                let mut ras_oaa = RasOAa::new(&mut ren_oaa);
                ras_oaa.set_line_join(if self.accurate_joins.borrow().status() {
                    OutlineAaJoin::MiterAccurate
                } else {
                    OutlineAaJoin::Round
                });
                ras_oaa.set_round_cap(true);
                for _i in 0..200 {
                    self.draw_anti_aliased_outline(&mut ras_oaa); //0
                    self.start_angle += deg2rad(self.step.borrow().value());
                }
                t3 = self.util.borrow_mut().elapsed_time();
            }

            {
                let mut ras_aa = RasS::new();
                let mut ren_aa = RAA::new_borrowed(&mut ren_base);
                let mut sl = SL::new();
                for _i in 0..200 {
                    self.draw_anti_aliased_scanline(&mut ras_aa, &mut sl, &mut ren_aa); //1
                    self.start_angle += deg2rad(self.step.borrow().value());
                }
                t4 = self.util.borrow_mut().elapsed_time();
            }

            {
                let filter = PF::new();
                let src = PatternPixmapArgb32::new(PIXMAP_CHAIN.as_ptr());
                let src_ht = src.height();
                let mut pattern = IP::new(filter);
                if self.scale_pattern.borrow().status() {
                    let src_scaled = agg::LineImageScale::new(src, self.width.borrow().value());
                    pattern.create(&src_scaled);
                } else {
                    pattern.create(&src);
                }
                let mut ren_img = ROI::new(&mut ren_base, pattern);
                if self.scale_pattern.borrow().status() {
                    ren_img.set_scale_x(self.width.borrow().value() / src_ht);
                }
                let mut ras_img = RasOI::new(&mut ren_img);
                for _i in 0..200 {
                    self.draw_anti_aliased_outline_img(&mut ras_img); //2
                    self.start_angle += deg2rad(self.step.borrow().value());
                }
                t5 = self.util.borrow_mut().elapsed_time();
            }

            self.test.borrow_mut().set_status(false);
            //self.return true
            let buf = format!(
                "Aliased={:.2}ms, Anti-Aliased={:.2}ms, Scanline={:.2}ms, Image-Pattern={:.2}ms",
                t2, t3, t4, t5
            );
            self.util.borrow_mut().message(&buf);
        }
    }

    fn on_draw(&mut self, rb: &mut agg::RenderBuf) {
        let mut pf = Pixfmt::new_borrowed(rb);
        let mut ren_base = RB::new_borrowed(&mut pf);
        ren_base.clear(&Rgba8::new_params(255, 255, 240, 255));

        {
            let mut ren_prim = RPrim::new(&mut ren_base);
            let mut ras_al = RasO::new(&mut ren_prim);
            self.draw_aliased_pix_accuracy(&mut ras_al); //3
            self.draw_aliased_subpix_accuracy(&mut ras_al); //4
        }

        {
            let mut prof = agg::LineProfileAA::new();
            prof.set_width(self.width.borrow().value());
            let mut ren_oaa = ROAa::new(&mut ren_base, prof);
            let mut ras_oaa = RasOAa::new(&mut ren_oaa);
            ras_oaa.set_line_join(if self.accurate_joins.borrow().status() {
                OutlineAaJoin::MiterAccurate
            } else {
                OutlineAaJoin::Round
            });
            ras_oaa.set_round_cap(true);
            self.draw_anti_aliased_outline(&mut ras_oaa); //0
        }

        {
            let mut ras_aa = RasS::new();
            let mut ren_aa = RAA::new_borrowed(&mut ren_base);
            let mut sl = SL::new();
            self.draw_anti_aliased_scanline(&mut ras_aa, &mut sl, &mut ren_aa); //1
        }

        {
            let filter = PF::new();
            let src = PatternPixmapArgb32::new(PIXMAP_CHAIN.as_ptr());
            let src_ht = src.height();
            let mut pattern = IP::new(filter);
            if self.scale_pattern.borrow().status() {
                let src_scaled = agg::LineImageScale::new(src, self.width.borrow().value());
                pattern.create(&src_scaled);
            } else {
                pattern.create(&src);
            }
            let mut ren_img = ROI::new(&mut ren_base, pattern);
            if self.scale_pattern.borrow().status() {
                ren_img.set_scale_x(self.width.borrow().value() / src_ht);
            }
            let mut ras_img = RasOI::new(&mut ren_img);
            self.draw_anti_aliased_outline_img(&mut ras_img); //2
        }

        let mut ras_aa = RasS::new();
        let mut ren_aa = RAA::new_borrowed(&mut ren_base);
        let mut sl = SL::new();
        self.text(
            &mut ras_aa,
            &mut sl,
            &mut ren_aa,
            50.,
            80.,
            "Bresenham lines,\n\nregular accuracy",
        );
        self.text(
            &mut ras_aa,
            &mut sl,
            &mut ren_aa,
            (self.util.borrow().width() / 2.) - 50.,
            80.,
            "Bresenham lines,\n\nsubpixel accuracy",
        );
        self.text(
            &mut ras_aa,
            &mut sl,
            &mut ren_aa,
            50.,
            (self.util.borrow().height() / 2.) + 50.,
            "Anti-aliased lines",
        );
        self.text(
            &mut ras_aa,
            &mut sl,
            &mut ren_aa,
            (self.util.borrow().width() / 2.) - 50.,
            (self.util.borrow().height() / 2.) + 50.,
            "Scanline rasterizer",
        );
        self.text(
            &mut ras_aa,
            &mut sl,
            &mut ren_aa,
            (self.util.borrow().width() - (self.util.borrow().width() / 5.)) - 50.,
            (self.util.borrow().height() / 2.) + 50.,
            "Arbitrary Image Pattern",
        );

        ctrl::render_ctrl(
            &mut ras_aa,
            &mut sl,
            &mut ren_base,
            &mut *self.step.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras_aa,
            &mut sl,
            &mut ren_base,
            &mut *self.width.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras_aa,
            &mut sl,
            &mut ren_base,
            &mut *self.test.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras_aa,
            &mut sl,
            &mut ren_base,
            &mut *self.rotate.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras_aa,
            &mut sl,
            &mut ren_base,
            &mut *self.accurate_joins.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras_aa,
            &mut sl,
            &mut ren_base,
            &mut *self.scale_pattern.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Line Join");

    if plat.init(500, 450, WindowFlag::Resize as u32) {
        plat.run();
    }
}

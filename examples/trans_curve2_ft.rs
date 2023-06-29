use crate::ctrl::cbox::Cbox;
use crate::ctrl::slider::Slider;
use crate::platform::{InputFlag, *};
use agg::color_rgba::*;
use agg::conv_bspline::*;
use agg::conv_curve::*;
use agg::conv_segmentator::*;
use agg::conv_transform::ConvTransform;
use agg::font_cache_manager::*;
use agg::font_freetype::FreetypeBase;
use agg::path_storage_integer::*;
use agg::rendering_buffer::*;
use agg::trans_double_path::TransDoublePath;
use agg::{Color, RasterScanLine, RendererScanlineColor};
mod ctrl;
mod platform;

use libc::*;
use std::cell::RefCell;
use std::rc::Rc;
const FLIP_Y: bool = true;
mod misc;
use misc::interactive_polygon::{InteractivePolygon, PolySrc};

fn frand() -> i32 {
    unsafe { rand() }
}

const TEXT: &str = "Anti-Grain Geometry is designed as a set of loosely coupled \
    algorithms and class templates united with a common idea, \
    so that all the components can be easily combined. Also, \
    the template based design allows you to replace any part of \
    the library without the necessity to modify a single byte in \
    the existing code. ";

type FontEngineType = FreetypeBase<'static, i16>;

pub struct Application {
    pub fman: FontCacheManager<FontEngineType>,
    pub poly1: InteractivePolygon<'static>,
    pub poly2: InteractivePolygon<'static>,
    pub num_points: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    pub preserve_x_scale: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    pub fixed_len: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    pub animate: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    pub dx1: [f64; 6],
    pub dy1: [f64; 6],
    pub dx2: [f64; 6],
    pub dy2: [f64; 6],
    pub prev_animate: bool,
    pub ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn move_point(&self, x: &mut f64, y: &mut f64, dx: &mut f64, dy: &mut f64) {
        if *x < 0.0 {
            *x = 0.0;
            *dx = -*dx;
        }
        if *x > self.util.borrow().width() {
            *x = self.util.borrow().width();
            *dx = -*dx;
        }
        if *y < 0.0 {
            *y = 0.0;
            *dy = -*dy;
        }
        if *y > self.util.borrow().height() {
            *y = self.util.borrow().height();
            *dy = -*dy;
        }
        *x += *dx;
        *y += *dy;
    }
    fn normalize_point(&mut self, i: usize) {
        let d = agg::math::calc_distance(
            self.poly1.xn(i),
            self.poly1.yn(i),
            self.poly2.xn(i),
            self.poly2.yn(i),
        );
        // 28.8 is 20 * sqrt(2)
        if d > 28.28 {
            *self.poly2.xn_mut(i) =
                self.poly1.xn(i) + (self.poly2.xn(i) - self.poly1.xn(i)) * 28.28 / d;
            *self.poly2.yn_mut(i) =
                self.poly1.yn(i) + (self.poly2.yn(i) - self.poly1.yn(i)) * 28.28 / d;
        }
    }
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let s0 = Rc::new(RefCell::new(Slider::new(5.0, 5.0, 340.0, 12.0, !flip_y)));
        let c0 = Rc::new(RefCell::new(Cbox::new(
            465.,
            5.0,
            "Preserve X scale",
            !flip_y,
        )));
        let c1 = Rc::new(RefCell::new(Cbox::new(350., 5.0, "Fixed Length", !flip_y)));
        let c2 = Rc::new(RefCell::new(Cbox::new(350., 25.0, "Animate", !flip_y)));

        let eng = FontEngineType::new_with_flags(false, 32);
        let mut app = Application {
            ctrls: CtrlContainer {
                ctrl: vec![s0.clone(), c0.clone(), c1.clone(), c2.clone()],
                cur_ctrl: -1,
                num_ctrl: 4,
            },

            fman: FontCacheManager::new(eng, 32),
            poly1: InteractivePolygon::new(6, 5.0),
            poly2: InteractivePolygon::new(6, 5.0),
            num_points: s0,

            preserve_x_scale: c0,
            fixed_len: c1,
            animate: c2,
            dx1: [0.0; 6],
            dy1: [0.0; 6],
            dx2: [0.0; 6],
            dy2: [0.0; 6],
            prev_animate: false,

            util: util,
        };

        app.preserve_x_scale.borrow_mut().set_status(true);
        app.fixed_len.borrow_mut().set_status(true);
        app.num_points.borrow_mut().set_range(10.0, 400.0);
        app.num_points.borrow_mut().set_value(200.0);
        app.num_points
            .borrow_mut()
            .set_label("Number of intermediate Points = %0.3f");
        app.poly1.set_close(false);
        app.poly2.set_close(false);
        app
    }
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_init(&mut self) {
        *self.poly1.xn_mut(0) = 10. + 50.;
        *self.poly1.yn_mut(0) = -10. + 50.;
        *self.poly1.xn_mut(1) = 10. + 150. + 20.;
        *self.poly1.yn_mut(1) = -10. + 150. - 20.;
        *self.poly1.xn_mut(2) = 10. + 250. - 20.;
        *self.poly1.yn_mut(2) = -10. + 250. + 20.;
        *self.poly1.xn_mut(3) = 10. + 350. + 20.;
        *self.poly1.yn_mut(3) = -10. + 350. - 20.;
        *self.poly1.xn_mut(4) = 10. + 450. - 20.;
        *self.poly1.yn_mut(4) = -10. + 450. + 20.;
        *self.poly1.xn_mut(5) = 10. + 550.;
        *self.poly1.yn_mut(5) = -10. + 550.;

        *self.poly2.xn_mut(0) = -10. + 50.;
        *self.poly2.yn_mut(0) = 10. + 50.;
        *self.poly2.xn_mut(1) = -10. + 150. + 20.;
        *self.poly2.yn_mut(1) = 10. + 150. - 20.;
        *self.poly2.xn_mut(2) = -10. + 250. - 20.;
        *self.poly2.yn_mut(2) = 10. + 250. + 20.;
        *self.poly2.xn_mut(3) = -10. + 350. + 20.;
        *self.poly2.yn_mut(3) = 10. + 350. - 20.;
        *self.poly2.xn_mut(4) = -10. + 450. - 20.;
        *self.poly2.yn_mut(4) = 10. + 450. + 20.;
        *self.poly2.xn_mut(5) = -10. + 550.;
        *self.poly2.yn_mut(5) = 10. + 550.;
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.poly1.on_mouse_button_down(x as f64, y as f64) {
                return Draw::Yes;
            }
            if self.poly2.on_mouse_button_down(x as f64, y as f64) {
                return Draw::Yes;
            }
        }
        return Draw::No;
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.poly1.on_mouse_move(x as f64, y as f64) {
                return Draw::Yes;
            }
            if self.poly2.on_mouse_move(x as f64, y as f64) {
                return Draw::Yes;
            }
        }
        if flags & InputFlag::MouseLeft as u32 == 0 {
            self.on_mouse_button_up(_rb, x, y, flags);
        }
        return Draw::No;
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, _flags: u32,
    ) -> Draw {
        if self.poly1.on_mouse_button_up(x as f64, y as f64) {
            return Draw::Yes;
        }
        if self.poly2.on_mouse_button_up(x as f64, y as f64) {
            return Draw::Yes;
        }
        return Draw::No;
    }

    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {
        if self.animate.borrow().status() != self.prev_animate {
            if self.animate.borrow().status() {
                self.on_init();
                for i in 0..6 {
                    self.dx1[i] = ((frand() % 1000) - 500) as f64 * 0.01;
                    self.dy1[i] = ((frand() % 1000) - 500) as f64 * 0.01;
                    self.dx2[i] = ((frand() % 1000) - 500) as f64 * 0.01;
                    self.dy2[i] = ((frand() % 1000) - 500) as f64 * 0.01;
                }
                self.util.borrow_mut().set_wait_mode(false);
            } else {
                self.util.borrow_mut().set_wait_mode(true);
            }
            self.prev_animate = self.animate.borrow().status();
        }
    }
    fn on_idle(&mut self) -> Draw {
        for i in 0..6 {
            let (mut x, mut y, mut dx, mut dy) =
                (self.poly1.xn(i), self.poly1.yn(i), self.dx1[i], self.dy1[i]);
            self.move_point(&mut x, &mut y, &mut dx, &mut dy);
            *self.poly1.xn_mut(i) = x;
            *self.poly1.yn_mut(i) = y;
            self.dx1[i] = dx;
            self.dy1[i] = dy;

            let (mut x, mut y, mut dx, mut dy) =
                (self.poly2.xn(i), self.poly2.yn(i), self.dx2[i], self.dy2[i]);
            self.move_point(&mut x, &mut y, &mut dx, &mut dy);
            *self.poly2.xn_mut(i) = x;
            *self.poly2.yn_mut(i) = y;
            self.dx2[i] = dx;
            self.dy2[i] = dy;
            self.normalize_point(i)
        }
        return Draw::Yes;
    }
    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rb);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);

        let path1 = PolySrc::new(
            self.poly1.polygon().as_ptr(),
            self.poly1.num_points(),
            false,
            false,
        );
        let path2 = PolySrc::new(
            self.poly2.polygon().as_ptr(),
            self.poly2.num_points(),
            false,
            false,
        );
        let mut bspline1 = ConvBspline::new_owned(path1);
        bspline1.set_interpolation_step(1.0 / self.num_points.borrow().value());
        let mut bspline2 = ConvBspline::new_owned(path2);
        bspline2.set_interpolation_step(1.0 / self.num_points.borrow().value());

        let mut tcurve = TransDoublePath::new();
        let mut fcurves: ConvCurve<SerializedIntegerPathAdaptor<i16>> =
            ConvCurve::new_owned(self.fman.path_adaptor());
        fcurves.set_approximation_scale(5.0);
        let mut fsegm = ConvSegmentator::new_owned(fcurves);
        fsegm.set_approximation_scale(3.0);

        tcurve.set_preserve_x_scale(self.preserve_x_scale.borrow().status());
        if self.fixed_len.borrow().status() {
            tcurve.set_base_length(1140.);
        }
        tcurve.set_base_height(30.);
        tcurve.add_paths(&mut bspline1, &mut bspline2, 0, 0);

        let mut ftrans = ConvTransform::new_owned(fsegm, tcurve);

        if self
            .fman
            .engine_mut()
            .load_font("timesi.ttf\u{0}", 0, GlyphRender::Outline, &[], 0)
        {
            let mut x = 0.0;
            let mut y = 3.0;
            let p = TEXT.as_bytes();
            let mut i = 0;
            self.fman.engine_mut().set_hinting(false);
            self.fman.engine_mut().set_height(40.);

            while p[i] != 0 {
                let glyph = self.fman.glyph(p[i] as u32);
                if !glyph.is_null() {
                    if x > ftrans.trans().total_length1() {
                        break;
                    }

                    self.fman.add_kerning(&mut x, &mut y);
                    //self.fman.init_embedded_adaptors(Some(unsafe { &*glyph }), x, y, 1.0);
                    let gl = unsafe { &*glyph };
                    if unsafe { (*glyph).data_type } == GlyphDataType::Outline {
                        ftrans.source_mut().source_mut().source_mut().init(
                            gl.data.as_ptr(),
                            gl.data_size as usize,
                            x,
                            y,
                            1.0,
                        );
                        ras.reset();
                        ras.add_path(&mut ftrans, 0);
                        r.set_color(Rgba8::new_params(0, 0, 0, 255));
                        agg::render_scanlines(&mut ras, &mut sl, &mut r);
                    }

                    unsafe {
                        // increment pen position
                        x += (*glyph).advance_x;
                        y += (*glyph).advance_y;
                    }
                }
                i += 1;
            }
        } else {
            self.util.borrow_mut().message(
                "Please copy file timesi.ttf to the current directory\n \
                            or download it from http://www.antigrain.com/timesi.zip",
            );
        }

        let mut stroke1 = agg::ConvStroke::<_>::new_owned(bspline1);
        let mut stroke2 = agg::ConvStroke::<_>::new_owned(bspline2);
        stroke1.set_width(2.0);
        stroke2.set_width(2.0);

        r.set_color(Rgba8::new_params(170, 50, 20, 100));
        ras.add_path(&mut stroke1, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut r);

        ras.add_path(&mut stroke2, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut r);
        //--------------------------
        // Render the "poly" tool and controls
        r.set_color(Rgba8::new_from_rgba(&Rgba::new_params(0., 0.3, 0.5, 0.3)));
        ras.add_path(&mut self.poly1, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut r);

        ras.add_path(&mut self.poly2, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut r);

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.num_points.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.preserve_x_scale.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.fixed_len.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.animate.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption(r#"AGG Example. Non-linear \"Along-A-Curve\" Transformer"#);

    if plat.init(600, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

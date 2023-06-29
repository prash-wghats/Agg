use agg::color_rgba::*;
use agg::conv_smooth_poly1::*;
use agg::image_accessors::*;
use agg::span_allocator::VecSpan;
use agg::span_pattern_rgba::*;
use agg::AggPrimitive;

use agg::{Color, RasterScanLine, RenderBuffer, RendererScanlineColor};

use crate::ctrl::cbox::Cbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

struct Application {
    polygon_angle: Ptr<Slider<'static, agg::Rgba8>>,
    polygon_scale: Ptr<Slider<'static, agg::Rgba8>>,
    pattern_angle: Ptr<Slider<'static, agg::Rgba8>>,
    pattern_size: Ptr<Slider<'static, agg::Rgba8>>,
    pattern_alpha: Ptr<Slider<'static, agg::Rgba8>>,
    rotate_polygon: Ptr<Cbox<'static, agg::Rgba8>>,
    rotate_pattern: Ptr<Cbox<'static, agg::Rgba8>>,
    tie_pattern: Ptr<Cbox<'static, agg::Rgba8>>,
    polygon_cx: f64,
    polygon_cy: f64,
    dx: f64,
    dy: f64,
    flag: i32,
    pattern: *mut u8,
    pattern_rbuf: agg::RenderBuf,
    ras: agg::RasterizerScanlineAa,
    sl: agg::ScanlineP8,
    ps: agg::PathStorage,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    // Create new application
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let polygon_angle = ctrl_ptr(Slider::new(5., 5., 145., 12., !flip_y));
        let polygon_scale = ctrl_ptr(Slider::new(5., 5. + 14., 145., 12. + 14., !flip_y));
        let pattern_angle = ctrl_ptr(Slider::new(155., 5., 300., 12., !flip_y));
        let pattern_size = ctrl_ptr(Slider::new(155., 5. + 14., 300., 12. + 14., !flip_y));
        let pattern_alpha = ctrl_ptr(Slider::new(310., 5., 460., 12., !flip_y));
        let rotate_polygon = ctrl_ptr(Cbox::new(5., 5. + 14. + 14., "Rotate Polygon", !flip_y));
        let rotate_pattern = ctrl_ptr(Cbox::new(
            5.,
            5. + 14. + 14. + 14.,
            "Rotate Pattern",
            !flip_y,
        ));
        let tie_pattern = ctrl_ptr(Cbox::new(
            155.,
            5. + 14. + 14.,
            "Tie pattern to polygon",
            !flip_y,
        ));
        polygon_angle.borrow_mut().set_label("Polygon Angle=%3.2f");
        polygon_angle.borrow_mut().set_range(-180.0, 180.0);

        polygon_scale.borrow_mut().set_label("Polygon Scale=%3.2f");
        polygon_scale.borrow_mut().set_range(0.1, 5.0);
        polygon_scale.borrow_mut().set_value(1.0);

        pattern_angle.borrow_mut().set_label("Pattern Angle=%3.2f");
        pattern_angle.borrow_mut().set_range(-180.0, 180.0);

        pattern_size.borrow_mut().set_label("Pattern Size=%3.2f");
        pattern_size.borrow_mut().set_range(10., 40.);
        pattern_size.borrow_mut().set_value(30.);

        pattern_alpha
            .borrow_mut()
            .set_label("Background Alpha=%.2f");
        pattern_alpha.borrow_mut().set_value(0.1);

        Application {
            polygon_angle: polygon_angle.clone(),
            polygon_scale: polygon_scale.clone(),
            pattern_angle: pattern_angle.clone(),
            pattern_size: pattern_size.clone(),
            pattern_alpha: pattern_alpha.clone(),
            rotate_polygon: rotate_polygon.clone(),
            rotate_pattern: rotate_pattern.clone(),
            tie_pattern: tie_pattern.clone(),
            polygon_cx: 0.0,
            polygon_cy: 0.0,
            dx: 0.0,
            dy: 0.0,
            flag: 0,
            pattern: std::ptr::null_mut(),
            pattern_rbuf: agg::RenderBuf::new_default(),
            ras: agg::RasterizerScanlineAa::new(),
            sl: agg::ScanlineP8::new(),
            ps: agg::PathStorage::new(),
            ctrls: CtrlContainer {
                ctrl: vec![
                    polygon_angle,
                    polygon_scale,
                    pattern_angle,
                    pattern_size,
                    pattern_alpha,
                    rotate_polygon,
                    rotate_pattern,
                    tie_pattern,
                ],
                cur_ctrl: -1,
                num_ctrl: 8,
            },
            util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    // Initialize application
    fn on_init(&mut self) {
        self.polygon_cx = self.util.borrow().initial_width() / 2.0;
        self.polygon_cy = self.util.borrow().initial_height() / 2.0;
        self.generate_pattern();
    }

    fn on_idle(&mut self) -> Draw {
        let mut redraw = false;
        if self.rotate_polygon.borrow().status() {
            let v = self.polygon_angle.borrow().value();
            self.polygon_angle.borrow_mut().set_value(v + 0.5);
            if self.polygon_angle.borrow().value() >= 180.0 {
                let v = self.polygon_angle.borrow().value();
                self.polygon_angle.borrow_mut().set_value(v - 360.0);
            }
            redraw = true;
        }

        if self.rotate_pattern.borrow().status() {
            let v = self.pattern_angle.borrow().value();
            self.pattern_angle.borrow_mut().set_value(v - 0.5);
            if self.pattern_angle.borrow().value() <= -180.0 {
                let v = self.pattern_angle.borrow().value();
                self.pattern_angle.borrow_mut().set_value(v + 360.0);
            }
            self.generate_pattern();
            redraw = true;
        }

        if redraw {
            Draw::Yes
        } else {
            Draw::No
        }
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32,
    ) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let mut polygon_mtx = agg::TransAffine::new_default();

            polygon_mtx *=
                agg::TransAffine::trans_affine_translation(-self.polygon_cx, -self.polygon_cy);
            polygon_mtx *= agg::TransAffine::trans_affine_rotation(
                self.polygon_angle.borrow().value() * PI / 180.0,
            );
            polygon_mtx *= agg::TransAffine::trans_affine_scaling(
                self.polygon_scale.borrow().value(),
                self.polygon_scale.borrow().value(),
            );
            polygon_mtx *=
                agg::TransAffine::trans_affine_translation(self.polygon_cx, self.polygon_cy);

            let r = self.util.borrow().initial_width() / 3.0 - 8.0;
            self.create_star(self.polygon_cx, self.polygon_cy, r, r / 1.45, 14, 0.);

            let mut tr = agg::ConvTransform::new_borrowed(&mut self.ps, polygon_mtx);
            self.ras.add_path(&mut tr, 0);
            if self.ras.hit_test(x, y) {
                self.dx = x as f64 - self.polygon_cx;
                self.dy = y as f64 - self.polygon_cy;
                self.flag = 1;
            }
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.flag != 0 {
                self.polygon_cx = x as f64 - self.dx;
                self.polygon_cy = y as f64 - self.dy;
                return Draw::Yes;
            }
            Draw::No
        } else {
            return self.on_mouse_button_up(_rb, x, y, flags);
        }
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.flag = 0;
        Draw::No
    }

    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {
        if self.rotate_polygon.borrow().status() || self.rotate_pattern.borrow().status() {
            self.util.borrow_mut().set_wait_mode(false);
        } else {
            self.util.borrow_mut().set_wait_mode(true);
        }

        self.generate_pattern();
        //self.return true
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let width = rbuf.width();
        let height = rbuf.height();

        let mut pixf = agg::PixBgra32::new_owned(rbuf.clone());
        let mut pixf_pre = agg::PixBgra32Pre::new_borrowed(rbuf);

        let mut rb = agg::RendererBase::<agg::PixBgra32>::new_borrowed(&mut pixf);
        let mut rb_pre = agg::RendererBase::<agg::PixBgra32Pre>::new_borrowed(&mut pixf_pre);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut polygon_mtx = agg::TransAffine::new_default();

        polygon_mtx *=
            agg::TransAffine::trans_affine_translation(-self.polygon_cx, -self.polygon_cy);
        polygon_mtx *= agg::TransAffine::trans_affine_rotation(
            self.polygon_angle.borrow().value() * PI / 180.0,
        );
        polygon_mtx *=
            agg::TransAffine::trans_affine_scaling_eq(self.polygon_scale.borrow().value());
        polygon_mtx *= agg::TransAffine::trans_affine_translation(self.polygon_cx, self.polygon_cy);

        let r = (self.util.borrow().initial_width() / 3.0) - 8.0;
        self.create_star(self.polygon_cx, self.polygon_cy, r, r / 1.45, 14, 0.);

        let mut tr = agg::ConvTransform::new_borrowed(&mut self.ps, polygon_mtx);

        type WrapXType = WrapModeReflectAutoPow2;
        type WrapYType = WrapModeReflectAutoPow2;
        type ImgSourceType<'a> = ImageAccessorWrap<'a, agg::PixBgra32<'a>, WrapXType, WrapYType>;
        type SpanGenType<'a> = SpanPatternRgba<ImgSourceType<'a>>;

        let mut offset_x = 0;
        let mut offset_y = 0;

        if self.tie_pattern.borrow().status() {
            offset_x = (width as f64 - self.polygon_cx) as u32;
            offset_y = (height as f64 - self.polygon_cy) as u32;
        }

        let mut sa = VecSpan::<ColorType>::new();
        let img_pixf = agg::PixBgra32::new_borrowed(&mut self.pattern_rbuf);
        let img_src = ImgSourceType::new(img_pixf);
        let mut sg = SpanGenType::new(img_src, offset_x, offset_y);

        sg.set_alpha(AggPrimitive::from_f64(
            self.pattern_alpha.borrow().value() * 255.0,
        ));

        self.ras.add_path(&mut tr, 0);
        agg::render_scanlines_aa(&mut self.ras, &mut self.sl, &mut rb_pre, &mut sa, &mut sg);

        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.polygon_angle.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.polygon_scale.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.pattern_angle.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.pattern_size.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.pattern_alpha.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.rotate_polygon.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.rotate_pattern.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut rb,
            &mut *self.tie_pattern.borrow_mut(),
        );
    }
}

impl Application {
    fn generate_pattern(&mut self) {
        let size = self.pattern_size.borrow().value() as usize;
        let pa = self.pattern_angle.borrow().value();
        self.create_star(
            size as f64 / 2.0,
            size as f64 / 2.0,
            size as f64 / 2.5,
            size as f64 / 6.0,
            6,
            pa,
        );

        let mut smooth = ConvSmoothPoly1Curve::<agg::PathStorage>::new_borrowed(&mut self.ps);

        smooth.set_smooth_value(1.0);
        smooth.set_approximation_scale(4.0);
        let mut stroke: agg::ConvStroke<'_, _> =
            agg::ConvStroke::<ConvSmoothPoly1Curve<agg::PathStorage>>::new_owned(smooth);
        stroke.set_width(self.pattern_size.borrow().value() / 15.0);

        //self.pattern = vec![0; (size * size * 4) as usize];
        self.pattern = unsafe {
            std::alloc::alloc(std::alloc::Layout::from_size_align(size * size * 4, 1).unwrap())
        };
        self.pattern_rbuf
            .attach(self.pattern, size as u32, size as u32, size as i32 * 4);

        let mut pixf = agg::PixBgra32::new_borrowed(&mut self.pattern_rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);

        rb.clear(&Rgba8::new_from_rgba(&rgba_pre(
            0.4,
            0.0,
            0.1,
            self.pattern_alpha.borrow().value(),
        ))); // Pattern background color

        let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        self.ras.add_path(stroke.source_mut(), 0);
        rs.set_color(agg::Rgba8::new_params(110, 130, 50, 255));
        agg::render_scanlines(&mut self.ras, &mut self.sl, &mut rs);

        self.ras.add_path(&mut stroke, 0);
        rs.set_color(agg::Rgba8::new_params(0, 50, 80, 255));
        agg::render_scanlines(&mut self.ras, &mut self.sl, &mut rs);
    }

    fn create_star(&mut self, xc: f64, yc: f64, r1: f64, r2: f64, n: usize, start_angle: f64) {
        self.ps.remove_all();
        let start_angle = start_angle * PI / 180.0;

        for i in 0..n {
            let a = PI * 2.0 * i as f64 / n as f64 - PI / 2.0;
            let dx = (a + start_angle).cos();
            let dy = (a + start_angle).sin();

            if i & 1 != 0 {
                self.ps.line_to(xc + dx * r1, yc + dy * r1);
            } else {
                if i != 0 {
                    self.ps.line_to(xc + dx * r2, yc + dy * r2);
                } else {
                    self.ps.move_to(xc + dx * r2, yc + dy * r2);
                }
            }
        }
        self.ps.close_polygon(0);
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);
    plat.set_caption("AGG Example: Pattern Filling");

    if plat.init(640, 480, WindowFlag::Resize as u32) {
        plat.run();
    }
}

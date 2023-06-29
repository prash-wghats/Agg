use crate::ctrl::bezier::*;
use crate::ctrl::cbox::*;
use crate::ctrl::rbox::*;
use crate::ctrl::slider::*;
use crate::platform::*;

use agg::basics::{deg2rad, is_stop, is_vertex};
use agg::color_rgba::*;
use agg::conv_stroke::*;
use agg::curves::*;
use agg::ellipse::Ellipse;
use agg::gsv_text::*;
use agg::math::{calc_distance, calc_line_point_distance};
use agg::math_stroke::{LineCap, LineJoin};
use agg::path_storage::*;
use agg::renderer_scanline::render_scanlines;
use agg::rendering_buffer::RenderBuf;
use agg::vertex_sequence::VertexDist;
use agg::{Color, CurveBase, CurveType4, RasterScanLine, RendererScanlineColor, VertexSource};
use ctrl::render_ctrl;
use sprintf::sprintf;
mod ctrl;
mod platform;

use core::f64::consts::PI;
use libc::*;
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

const FLIP_Y: bool = true;

fn frand() -> i32 {
    unsafe { rand() }
}

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

fn bezier4_point(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64, mu: f64, x: &mut f64,
    y: &mut f64,
) {
    let mum1 = 1.0 - mu;
    let mum13 = mum1 * mum1 * mum1;
    let mu3 = mu * mu * mu;

    *x = mum13 * x1 + 3.0 * mu * mum1 * mum1 * x2 + 3.0 * mu * mu * mum1 * x3 + mu3 * x4;
    *y = mum13 * y1 + 3.0 * mu * mum1 * mum1 * y2 + 3.0 * mu * mu * mum1 * y3 + mu3 * y4;
}

struct Application {
    _ctrl_color: Rgba8,
    curve1: Rc<RefCell<Bezier<'static, agg::Rgba8>>>,
    angle_tolerance: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    approximation_scale: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    cusp_limit: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    width: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    show_points: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    show_outline: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    curve_type: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    case_type: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    inner_join: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    line_join: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    line_cap: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    cur_case_type: i32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn measure_time<Curv: CurveType4>(&self, curve: &mut Curv) -> f64 {
        self.util.borrow_mut().start_timer();
        for _ in 0..100 {
            let mut x = 0.0;
            let mut y = 0.0;
            curve.init(
                self.curve1.borrow().x1(),
                self.curve1.borrow().y1(),
                self.curve1.borrow().x2(),
                self.curve1.borrow().y2(),
                self.curve1.borrow().x3(),
                self.curve1.borrow().y3(),
                self.curve1.borrow().x4(),
                self.curve1.borrow().y4(),
            );
            curve.rewind(0);
            while !is_stop(curve.vertex(&mut x, &mut y)) {}
        }
        self.util.borrow_mut().elapsed_time() * 10.0
    }

    fn find_point(path: &Vec<VertexDist>, dist: f64, i: &mut usize, j: &mut usize) -> bool {
        *j = path.len() - 1;
        *i = 0;
        while (*j - *i) > 1 {
            if dist < path[(*i + *j) >> 1].dist {
                *j = (*i + *j) >> 1;
            } else {
                *i = (*i + *j) >> 1;
            }
        }
        true
    }

    fn calc_max_error<Curv: CurveType4>(
        &self, curve: &mut Curv, scale: f64, max_angle_error: &mut f64,
    ) -> f64 {
        curve.set_approximation_scale(self.approximation_scale.borrow().value() * scale);
        curve.init(
            self.curve1.borrow().x1(),
            self.curve1.borrow().y1(),
            self.curve1.borrow().x2(),
            self.curve1.borrow().y2(),
            self.curve1.borrow().x3(),
            self.curve1.borrow().y3(),
            self.curve1.borrow().x4(),
            self.curve1.borrow().y4(),
        );

        let mut curve_points: Vec<VertexDist> = Vec::new();
        let mut cmd;
        let mut x = 0.0;
        let mut y = 0.0;
        curve.rewind(0);

        loop {
            cmd = curve.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_vertex(cmd) {
                curve_points.push(VertexDist::new(x, y));
            }
        }
        let mut curve_dist = 0.0;
        for i in 1..curve_points.len() {
            curve_points[i - 1].dist = curve_dist;
            curve_dist += calc_distance(
                curve_points[i - 1].x,
                curve_points[i - 1].y,
                curve_points[i].x,
                curve_points[i].y,
            );
        }
        let len = curve_points.len();
        curve_points[len - 1].dist = curve_dist;

        let mut reference_points: Vec<CurvePoint> = Vec::new();
        for i in 0..4096 {
            let mu = i as f64 / 4095.0;
            bezier4_point(
                self.curve1.borrow().x1(),
                self.curve1.borrow().y1(),
                self.curve1.borrow().x2(),
                self.curve1.borrow().y2(),
                self.curve1.borrow().x3(),
                self.curve1.borrow().y3(),
                self.curve1.borrow().x4(),
                self.curve1.borrow().y4(),
                mu,
                &mut x,
                &mut y,
            );
            reference_points.push(CurvePoint {
                x: x,
                y: y,
                //mu: mu,
                dist: 0.,
            });
        }

        let mut reference_dist = 0.0;
        for i in 1..reference_points.len() {
            reference_points[i - 1].dist = reference_dist;
            reference_dist += calc_distance(
                reference_points[i - 1].x,
                reference_points[i - 1].y,
                reference_points[i].x,
                reference_points[i].y,
            );
        }
        let len = reference_points.len();
        reference_points[len - 1].dist = reference_dist;

        let mut idx1 = 0;
        let mut idx2 = 1;
        let mut max_error = 0.0;
        for i in 0..reference_points.len() {
            if Self::find_point(
                &curve_points,
                reference_points[i].dist,
                &mut idx1,
                &mut idx2,
            ) {
                let err = calc_line_point_distance(
                    curve_points[idx1].x,
                    curve_points[idx1].y,
                    curve_points[idx2].x,
                    curve_points[idx2].y,
                    reference_points[i].x,
                    reference_points[i].y,
                )
                .abs();
                if err > max_error {
                    max_error = err;
                }
            }
        }

        let mut aerr = 0.0;
        for i in 2..curve_points.len() {
            let a1 = (curve_points[i - 1].y - curve_points[i - 2].y)
                .atan2(curve_points[i - 1].x - curve_points[i - 2].x);
            let a2 = (curve_points[i].y - curve_points[i - 1].y)
                .atan2(curve_points[i].x - curve_points[i - 1].x);

            let mut da = (a1 - a2).abs();
            if da >= PI {
                da = 2.0 * PI - da;
            }
            if da > aerr {
                aerr = da;
            }
        }

        *max_angle_error = aerr * 180.0 / PI;
        max_error * scale
    }
}
impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Application {
        let ctrl_color = Rgba8::new_from_rgba(&Rgba::new_params(0., 0.3, 0.5, 0.8));
        let mut curve1 = Bezier::new();
        let mut angle_tolerance = Slider::new(5.0, 5.0, 240.0, 12.0, !flip_y);
        let mut approximation_scale = Slider::new(5.0, 17. + 5.0, 240.0, 17. + 12.0, !flip_y);
        let mut cusp_limit = Slider::new(5.0, 17. + 17. + 5.0, 240.0, 17. + 17. + 12.0, !flip_y);
        let mut width = Slider::new(245.0, 5.0, 495.0, 12.0, !flip_y);
        let mut show_points = Cbox::new(250.0, 15. + 5., "Show Points", !flip_y);
        let mut show_outline = Cbox::new(250.0, 30. + 5., "Show Stroke Outline", !flip_y);
        let mut curve_type = Rbox::new(535.0, 5.0, 535.0 + 115.0, 55.0, !flip_y);
        let mut case_type = Rbox::new(535.0, 60.0, 535.0 + 115.0, 195.0, !flip_y);
        let mut inner_join = Rbox::new(535.0, 200.0, 535.0 + 115.0, 290.0, !flip_y);
        let mut line_join = Rbox::new(535.0, 295.0, 535.0 + 115.0, 385.0, !flip_y);
        let mut line_cap = Rbox::new(535.0, 395.0, 535.0 + 115.0, 455.0, !flip_y);

        curve1.set_line_color(&ctrl_color);
        curve1.set_curve(170., 424., 13., 87., 488., 423., 26., 333.);
        curve1.no_transform();

        angle_tolerance.set_label("Angle Tolerance=%.1f deg");
        angle_tolerance.set_range(0., 90.);
        angle_tolerance.set_value(15.);

        angle_tolerance.no_transform();

        approximation_scale.set_label("Approximation Scale=%.3f");
        approximation_scale.set_range(0.1, 5.);
        approximation_scale.set_value(1.0);

        approximation_scale.no_transform();

        cusp_limit.set_label("Cusp Limit=%.1f deg");
        cusp_limit.set_range(0., 90.);
        cusp_limit.set_value(0.);

        cusp_limit.no_transform();

        width.set_label("Width=%.2f");
        width.set_range(-50., 100.);
        width.set_value(50.0);

        width.no_transform();

        show_points.no_transform();
        show_points.set_status(true);

        show_outline.no_transform();
        show_outline.set_status(true);

        curve_type.add_item("Incremental");
        curve_type.add_item("Subdiv");
        curve_type.set_cur_item(1);

        curve_type.no_transform();

        case_type.set_text_size(7., 0.);
        case_type.set_text_thickness(1.0);
        case_type.add_item("Random");
        case_type.add_item("13---24");
        case_type.add_item("Smooth Cusp 1");
        case_type.add_item("Smooth Cusp 2");
        case_type.add_item("Real Cusp 1");
        case_type.add_item("Real Cusp 2");
        case_type.add_item("Fancy Stroke");
        case_type.add_item("Jaw");
        case_type.add_item("Ugly Jaw");

        case_type.no_transform();

        inner_join.set_text_size(8., 0.);
        inner_join.add_item("Inner Bevel");
        inner_join.add_item("Inner Miter");
        inner_join.add_item("Inner Jag");
        inner_join.add_item("Inner Round");
        inner_join.set_cur_item(3);

        inner_join.no_transform();

        line_join.set_text_size(8., 0.);
        line_join.add_item("Miter Join");
        line_join.add_item("Miter Revert");
        line_join.add_item("Round Join");
        line_join.add_item("Bevel Join");
        line_join.add_item("Miter Round");

        line_join.set_cur_item(1);

        line_join.no_transform();

        line_cap.set_text_size(8., 0.);
        line_cap.add_item("Butt Cap");
        line_cap.add_item("Square Cap");
        line_cap.add_item("Round Cap");
        line_cap.set_cur_item(0);

        line_cap.no_transform();

        //let ctrl_color = ctrl_ptr(ctrl_color);
        let curve1 = ctrl_ptr(curve1);
        let angle_tolerance = ctrl_ptr(angle_tolerance);
        let approximation_scale = ctrl_ptr(approximation_scale);
        let cusp_limit = ctrl_ptr(cusp_limit);
        let width = ctrl_ptr(width);
        let show_points = ctrl_ptr(show_points);
        let show_outline = ctrl_ptr(show_outline);
        let curve_type = ctrl_ptr(curve_type);
        let case_type = ctrl_ptr(case_type);
        let inner_join = ctrl_ptr(inner_join);
        let line_cap = ctrl_ptr(line_cap);
        let line_join = ctrl_ptr(line_join);
        let app = Application {
            _ctrl_color: ctrl_color,
            curve1: curve1.clone(),
            angle_tolerance: angle_tolerance.clone(),
            approximation_scale: approximation_scale.clone(),
            cusp_limit: cusp_limit.clone(),
            width: width.clone(),
            show_points: show_points.clone(),
            show_outline: show_outline.clone(),
            curve_type: curve_type.clone(),
            case_type: case_type.clone(),
            inner_join: inner_join.clone(),
            line_cap: line_cap.clone(),
            line_join: line_join.clone(),

            cur_case_type: -1,
            ctrls: CtrlContainer {
                ctrl: vec![
                    curve1,
                    angle_tolerance,
                    approximation_scale,
                    cusp_limit,
                    width,
                    show_points,
                    show_outline,
                    curve_type,
                    case_type,
                    inner_join,
                    line_cap,
                    line_join,
                ],
                cur_ctrl: -1,
                num_ctrl: 12,
            },
            util: util,
        };

        app
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 240, 255));
        let mut ren_sl = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let mut path = PathStorage::new();

        let mut curve = Curve4::new();
        curve.set_approximation_method(unsafe {
            std::mem::transmute(self.curve_type.borrow().cur_item() as u8)
        });
        curve.set_approximation_scale(self.approximation_scale.borrow().value());
        curve.set_angle_tolerance(deg2rad(self.angle_tolerance.borrow().value()));
        curve.set_cusp_limit(deg2rad(self.cusp_limit.borrow().value()));
        let curve_time = self.measure_time(&mut curve);
        let mut max_angle_error_01 = 0.0;
        let mut max_angle_error_1 = 0.0;
        let mut max_angle_error1 = 0.0;
        let mut max_angle_error_10 = 0.0;
        let mut max_angle_error_100 = 0.0;
        let max_error_01;
        let max_error_1;
        let max_error1;
        let max_error_10;
        let max_error_100;

        max_error_01 = self.calc_max_error(&mut curve, 0.01, &mut max_angle_error_01);
        max_error_1 = self.calc_max_error(&mut curve, 0.1, &mut max_angle_error_1);
        max_error1 = self.calc_max_error(&mut curve, 1.0, &mut max_angle_error1);
        max_error_10 = self.calc_max_error(&mut curve, 10.0, &mut max_angle_error_10);
        max_error_100 = self.calc_max_error(&mut curve, 100.0, &mut max_angle_error_100);

        curve.set_approximation_scale(self.approximation_scale.borrow().value());
        curve.set_angle_tolerance(deg2rad(self.angle_tolerance.borrow().value()));
        curve.set_cusp_limit(deg2rad(self.cusp_limit.borrow().value()));
        curve.init(
            self.curve1.borrow().x1(),
            self.curve1.borrow().y1(),
            self.curve1.borrow().x2(),
            self.curve1.borrow().y2(),
            self.curve1.borrow().x3(),
            self.curve1.borrow().y3(),
            self.curve1.borrow().x4(),
            self.curve1.borrow().y4(),
        );

        path.concat_path(&mut curve, 0);

        let mut stroke: ConvStroke<'_, _> = ConvStroke::new_owned(path);
        stroke.set_width(self.width.borrow().value());
        stroke.set_line_join(unsafe {
            std::mem::transmute(self.line_join.borrow().cur_item() as u8)
        });
        stroke
            .set_line_cap(unsafe { std::mem::transmute(self.line_cap.borrow().cur_item() as u8) });
        stroke.set_inner_join(unsafe {
            std::mem::transmute(self.inner_join.borrow().cur_item() as u8)
        });
        stroke.set_inner_miter_limit(1.01);

        ras.add_path(&mut stroke, 0);
        ren_sl.set_color(Rgba8::new_params(0, 127, 0, 127));
        render_scanlines(&mut ras, &mut sl, &mut ren_sl);

        let mut cmd;
        let mut num_points1 = 0;
        let (mut x, mut y) = (0., 0.);

        stroke.source_mut().rewind(0);
        loop {
            cmd = stroke.source_mut().vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if self.show_points.borrow().status() {
                let mut ell = Ellipse::new_ellipse(x, y, 1.5, 1.5, 8, false);
                ras.add_path(&mut ell, 0);
                ren_sl.set_color(Rgba8::new_params(0, 0, 0, 127));
                render_scanlines(&mut ras, &mut sl, &mut ren_sl);
            }
            num_points1 += 1;
        }

        if self.show_outline.borrow().status() {
            // Draw a stroke of the stroke to see the internals
            //--------------
            let mut stroke2: ConvStroke<'_, _> = ConvStroke::new_owned(stroke);
            ras.add_path(&mut stroke2, 0);
            ren_sl.set_color(Rgba8::new_params(0, 0, 0, 127));
            render_scanlines(&mut ras, &mut sl, &mut ren_sl);
        }

        let mut t = GsvText::new();
        t.set_size(8.0, 0.);

        let mut pt: ConvStroke<'_, _> = ConvStroke::new_owned(t);
        pt.set_line_cap(LineCap::Round);
        pt.set_line_join(LineJoin::Round);
        pt.set_width(1.5);

        let buf = sprintf!("Num Points=%d Time=%.2fmks\n\nDist Error: x0.01=%.5f x0.1=%.5f x1=%.5f x10=%.5f x100=%.5f\n\n Angle Error: x0.01=%.1f x0.1=%.1f x1=%.1f x10=%.1f x100=%.1f",
                num_points1, curve_time,
                max_error_01,
                max_error_1,
                max_error1,
                max_error_10,
                max_error_100,
                max_angle_error_01,
                max_angle_error_1,
                max_angle_error1,
                max_angle_error_10,
                max_angle_error_100).unwrap();

        pt.source_mut().set_start_point(10.0, 85.0);
        pt.source_mut().set_text(&buf);

        ras.add_path(&mut pt, 0);
        ren_sl.set_color(Rgba8::new_params(0, 0, 0, 255));
        render_scanlines(&mut ras, &mut sl, &mut ren_sl);

        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve1.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.angle_tolerance.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.approximation_scale.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.cusp_limit.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.width.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.show_points.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.show_outline.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve_type.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.case_type.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.inner_join.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.line_join.borrow_mut(),
        );
        render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.line_cap.borrow_mut(),
        );
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        if key == ' ' as u32 {
            let mut fd = File::create("coord").unwrap();
            write!(
                fd,
                "{:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}, {:.3}",
                self.curve1.borrow().x1(),
                self.curve1.borrow().y1(),
                self.curve1.borrow().x2(),
                self.curve1.borrow().y2(),
                self.curve1.borrow().x3(),
                self.curve1.borrow().y3(),
                self.curve1.borrow().x4(),
                self.curve1.borrow().y4()
            )
            .unwrap();
        }
        Draw::No
    }

    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {
        if self.case_type.borrow().cur_item() != self.cur_case_type {
            match self.case_type.borrow().cur_item() {
                0 => {
                    let w = (self.util.borrow().width() - 120.) as i32;
                    let h = (self.util.borrow().height() - 80.) as i32;
                    self.curve1.borrow_mut().set_curve(
                        (frand() % w) as f64,
                        (frand() % h + 80) as f64,
                        (frand() % w) as f64,
                        (frand() % h + 80) as f64,
                        (frand() % w) as f64,
                        (frand() % h + 80) as f64,
                        (frand() % w) as f64,
                        (frand() % h + 80) as f64,
                    );
                }
                1 => self
                    .curve1
                    .borrow_mut()
                    .set_curve(150., 150., 350., 150., 150., 150., 350., 150.),
                2 => self
                    .curve1
                    .borrow_mut()
                    .set_curve(50., 142., 483., 251., 496., 62., 26., 333.),
                3 => self
                    .curve1
                    .borrow_mut()
                    .set_curve(50., 142., 484., 251., 496., 62., 26., 333.),
                4 => self
                    .curve1
                    .borrow_mut()
                    .set_curve(100., 100., 300., 200., 200., 200., 200., 100.),
                5 => self
                    .curve1
                    .borrow_mut()
                    .set_curve(475., 157., 200., 100., 453., 100., 222., 157.),
                6 => {
                    self.curve1
                        .borrow_mut()
                        .set_curve(129., 233., 32., 283., 258., 285., 159., 232.);
                    self.width.borrow_mut().set_value(100.);
                }
                7 => self
                    .curve1
                    .borrow_mut()
                    .set_curve(100., 100., 300., 200., 264., 286., 264., 284.),
                8 => self
                    .curve1
                    .borrow_mut()
                    .set_curve(100., 100., 413., 304., 264., 286., 264., 284.),
                _ => {}
            }
            //return true
            self.cur_case_type = self.case_type.borrow().cur_item();
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }
}

struct CurvePoint {
    x: f64,
    y: f64,
    dist: f64,
    //mu: f64,
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    //plat.app_mut().init();
    plat.set_caption("AGG Example.Bezier Div");

    if plat.init(655, 520, WindowFlag::Resize as u32) {
        plat.run();
    }
}

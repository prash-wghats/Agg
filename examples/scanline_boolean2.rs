use crate::ctrl::rbox::*;
use crate::platform::*;
use misc::{make_arrows::make_arrows, make_gb_poly::make_gb_poly, pixel_formats::*};

use agg::basics::{deg2rad, PathCmd};
use agg::rendering_buffer::RenderBuf;
use agg::scanline_boolean_algebra::*;
use agg::{RasterScanLine, RendererScanlineColor, VertexSource};

mod ctrl;
mod misc;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

struct Spiral {
    x: f64,
    y: f64,
    r1: f64,
    r2: f64,
    //step: f64,
    start_angle: f64,
    angle: f64,
    curr_r: f64,
    da: f64,
    dr: f64,
    start: bool,
}

impl Spiral {
    pub fn new(x: f64, y: f64, r1: f64, r2: f64, step: f64, start_angle: f64) -> Self {
        let da = deg2rad(4.0);
        let dr = step / 90.0;
        Spiral {
            x,
            y,
            r1,
            r2,
            //step,
            start_angle,
            angle: start_angle,
            curr_r: 0.,
            da,
            dr,
            start: true,
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

fn count_spans<Rasterizer: RasterScanLine, Scanline: agg::Scanline>(
    ras: &mut Rasterizer, sl: &mut Scanline,
) -> u32 {
    let mut n = 0;
    if ras.rewind_scanlines() {
        sl.reset(ras.min_x(), ras.max_x());
        while ras.sweep_scanline(sl) {
            n += sl.num_spans();
        }
    }
    n
}

struct Application {
    polygons: Ptr<Rbox<'static, agg::Rgba8>>,
    fill_rule: Ptr<Rbox<'static, agg::Rgba8>>,
    scanline_type: Ptr<Rbox<'static, agg::Rgba8>>,
    operation: Ptr<Rbox<'static, agg::Rgba8>>,
    x: f64,
    y: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let mut polygons = Rbox::new(5.0, 5.0, 5.0 + 205.0, 110.0, !flip_y);
        let mut fill_rule = Rbox::new(200., 5.0, 200. + 105.0, 50.0, !flip_y);
        let mut scanline_type = Rbox::new(300., 5.0, 300. + 115.0, 70.0, !flip_y);
        let mut operation = Rbox::new(535.0, 5.0, 535.0 + 115.0, 145.0, !flip_y);
        operation.add_item("None");
        operation.add_item("OR");
        operation.add_item("AND");
        operation.add_item("XOR Linear");
        operation.add_item("XOR Saddle");
        operation.add_item("A-B");
        operation.add_item("B-A");
        operation.set_cur_item(2);
        operation.no_transform();

        fill_rule.add_item("Even-Odd");
        fill_rule.add_item("Non Zero");
        fill_rule.set_cur_item(1);
        fill_rule.no_transform();

        scanline_type.add_item("scanline_p");
        scanline_type.add_item("scanline_u");
        scanline_type.add_item("scanline_bin");
        scanline_type.set_cur_item(1);
        scanline_type.no_transform();

        polygons.add_item("Two Simple Paths");
        polygons.add_item("Closed Stroke");
        polygons.add_item("Great Britain and Arrows");
        polygons.add_item("Great Britain and Spiral");
        polygons.add_item("Spiral and Glyph");
        polygons.set_cur_item(3);
        polygons.no_transform();
        let polygons = ctrl_ptr(polygons);
        let scanline_type = ctrl_ptr(scanline_type);
        let fill_rule = ctrl_ptr(fill_rule);
        let operation = ctrl_ptr(operation);

        Application {
            polygons: polygons.clone(),
            fill_rule: fill_rule.clone(),
            scanline_type: scanline_type.clone(),
            operation: operation.clone(),
            x: 0.0,
            y: 0.0,
            ctrls: CtrlContainer {
                ctrl: vec![polygons, operation, fill_rule, scanline_type],
                cur_ctrl: -1,
                num_ctrl: 4,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        self.x = self.util.borrow().width() / 2.0;
        self.y = self.util.borrow().height() / 2.0;
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut pf = Pixfmt::new_borrowed(rb);
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pf);
        let _ren_solid = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);

        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut ras2: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.polygons.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.fill_rule.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.scanline_type.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.operation.borrow_mut(),
        );

        ras.set_filling_rule(if self.fill_rule.borrow().cur_item() != 0 {
            agg::FillingRule::FillNonZero
        } else {
            agg::FillingRule::FillEvenOdd
        });
        ras2.set_filling_rule(if self.fill_rule.borrow().cur_item() != 0 {
            agg::FillingRule::FillNonZero
        } else {
            agg::FillingRule::FillEvenOdd
        });

        self.render_sbool(rb, &mut ras, &mut ras2);
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32,
    ) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.x = x as f64;
            self.y = y as f64;
            return Draw::Yes;
        }

        if flags & InputFlag::MouseRight as u32 != 0 {
            let buf = format!("{} {}", x, y);
            self.util.borrow_mut().message(&buf);
        }
        return Draw::No;
    }

    fn on_mouse_button_up(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.x = x as f64;
            self.y = y as f64;
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.x = x as f64;
            self.y = y as f64;
            return Draw::Yes;
        }
        Draw::No
    }
}

impl Application {
    fn render_scanline_boolean<Rasterizer: RasterScanLine>(
        &mut self, rbuf: &mut RenderBuf, ras1: &mut Rasterizer, ras2: &mut Rasterizer,
    ) {
        if self.operation.borrow().cur_item() > 0 {
            let op;
            match self.operation.borrow().cur_item() {
                1 => op = SBoolOp::Or,
                2 => op = SBoolOp::And,
                3 => op = SBoolOp::Xor,
                4 => op = SBoolOp::XorSaddle,
                5 => op = SBoolOp::AMinusB,
                6 => op = SBoolOp::BMinusA,
                _ => panic!("Invalid self.operation"),
            };

            let mut pixf = Pixfmt::new_borrowed(rbuf);
            let mut rb = agg::RendererBase::new_borrowed(&mut pixf);

            let t1;
            let t2;
            let num_spans;

            match self.scanline_type.borrow().cur_item() {
                0 => {
                    let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

                    let mut sl = agg::ScanlineP8::new();
                    let mut sl1 = agg::ScanlineP8::new();
                    let mut sl2 = agg::ScanlineP8::new();

                    let mut storage = agg::ScanlineStorageAA8::new();
                    let mut storage1 = agg::ScanlineStorageAA8::new();
                    let mut storage2 = agg::ScanlineStorageAA8::new();

                    agg::render_scanlines(ras1, &mut sl, &mut storage1);
                    agg::render_scanlines(ras2, &mut sl, &mut storage2);

                    self.util.borrow_mut().start_timer();
                    for _ in 0..10 {
                        sbool_combine_shapes_aa(
                            op,
                            &mut storage1,
                            &mut storage2,
                            &mut sl1,
                            &mut sl2,
                            &mut sl,
                            &mut storage,
                        );
                    }
                    t1 = self.util.borrow_mut().elapsed_time() / 10.0;

                    self.util.borrow_mut().start_timer();
                    ren.set_color(agg::Rgba8::new_params(125, 0, 0, 125));
                    agg::render_scanlines(&mut storage, &mut sl, &mut ren);
                    t2 = self.util.borrow_mut().elapsed_time();

                    num_spans = count_spans(&mut storage, &mut sl);
                }
                1 => {
                    let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

                    let mut sl = agg::ScanlineU8::new();
                    let mut sl1 = agg::ScanlineU8::new();
                    let mut sl2 = agg::ScanlineU8::new();

                    let mut storage = agg::ScanlineStorageAA8::new();
                    let mut storage1 = agg::ScanlineStorageAA8::new();
                    let mut storage2 = agg::ScanlineStorageAA8::new();

                    agg::render_scanlines(ras1, &mut sl, &mut storage1);
                    agg::render_scanlines(ras2, &mut sl, &mut storage2);

                    self.util.borrow_mut().start_timer();
                    for _ in 0..10 {
                        sbool_combine_shapes_aa(
                            op,
                            &mut storage1,
                            &mut storage2,
                            &mut sl1,
                            &mut sl2,
                            &mut sl,
                            &mut storage,
                        );
                    }
                    t1 = self.util.borrow_mut().elapsed_time() / 10.0;

                    self.util.borrow_mut().start_timer();
                    ren.set_color(agg::Rgba8::new_params(125, 0, 0, 125));
                    agg::render_scanlines(&mut storage, &mut sl, &mut ren);
                    t2 = self.util.borrow_mut().elapsed_time();

                    num_spans = count_spans(&mut storage, &mut sl);
                }
                2 => {
                    let mut ren = agg::RendererScanlineBinSolid::new_borrowed(&mut rb);

                    let mut sl = agg::ScanlineBin::new();
                    let mut sl1 = agg::ScanlineBin::new();
                    let mut sl2 = agg::ScanlineBin::new();

                    let mut storage = agg::ScanlineStorageBin::new();
                    let mut storage1 = agg::ScanlineStorageBin::new();
                    let mut storage2 = agg::ScanlineStorageBin::new();

                    agg::render_scanlines(ras1, &mut sl, &mut storage1);
                    agg::render_scanlines(ras2, &mut sl, &mut storage2);

                    self.util.borrow_mut().start_timer();
                    for _ in 0..10 {
                        sbool_combine_shapes_bin(
                            op,
                            &mut storage1,
                            &mut storage2,
                            &mut sl1,
                            &mut sl2,
                            &mut sl,
                            &mut storage,
                        );
                    }
                    t1 = self.util.borrow_mut().elapsed_time() / 10.0;

                    self.util.borrow_mut().start_timer();
                    ren.set_color(agg::Rgba8::new_params(125, 0, 0, 125));
                    agg::render_scanlines(&mut storage, &mut sl, &mut ren);
                    t2 = self.util.borrow_mut().elapsed_time();

                    num_spans = count_spans(&mut storage, &mut sl);
                }
                _ => {
                    panic!("Invalid self.scanline_type");
                }
            }

            let buf = format!(
                "Combine={:.1}ms\n\nRender={:.1}ms\n\nnum_spans={}",
                t1, t2, num_spans
            );
            let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
            let mut sl = agg::ScanlineP8::new();
            let mut txt = agg::GsvText::new();
            txt.set_size(8.0, 0.);
            txt.set_start_point(420., 40.);
            txt.set_text(&buf);
            let mut txt_stroke: agg::ConvStroke<_> = agg::ConvStroke::new_owned(txt);
            txt_stroke.set_width(1.0);
            txt_stroke.set_line_cap(agg::math_stroke::LineCap::Round);

            ras1.add_path(&mut txt_stroke, 0);
            ren.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
            agg::render_scanlines(ras1, &mut sl, &mut ren);
        }
    }

    fn render_sbool<Rasterizer: RasterScanLine>(
        &mut self, rbuf: &mut RenderBuf, ras1: &mut Rasterizer, ras2: &mut Rasterizer,
    ) -> u32 {
        let mut pf = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pf);
        let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
        let mut sl = agg::ScanlineP8::new();

        let item = self.polygons.borrow().cur_item();
        match item {
            0 => {
                //------------------------------------
                // Two simple paths
                //
                let mut ps1 = agg::PathStorage::new();
                let mut ps2 = agg::PathStorage::new();

                let x = self.x - self.util.borrow().initial_width() / 2.0 + 100.0;
                let y = self.y - self.util.borrow().initial_height() / 2.0 + 100.0;
                ps1.move_to(x + 140.0, y + 145.0);
                ps1.line_to(x + 225.0, y + 44.0);
                ps1.line_to(x + 296.0, y + 219.0);
                ps1.close_polygon(0);

                ps1.line_to(x + 226.0, y + 289.0);
                ps1.line_to(x + 82.0, y + 292.0);

                ps1.move_to(x + 220.0, y + 222.0);
                ps1.line_to(x + 363.0, y + 249.0);
                ps1.line_to(x + 265.0, y + 331.0);

                ps1.move_to(x + 242.0, y + 243.0);
                ps1.line_to(x + 325.0, y + 261.0);
                ps1.line_to(x + 268.0, y + 309.0);

                ps1.move_to(x + 259.0, y + 259.0);
                ps1.line_to(x + 273.0, y + 288.0);
                ps1.line_to(x + 298.0, y + 266.0);

                ps2.move_to(100.0 + 32.0, 100.0 + 77.0);
                ps2.line_to(100.0 + 473.0, 100.0 + 263.0);
                ps2.line_to(100.0 + 351.0, 100.0 + 290.0);
                ps2.line_to(100.0 + 354.0, 100.0 + 374.0);

                ras1.reset();
                ras1.add_path(&mut ps1, 0);
                ren.set_color(agg::Rgba8::new_params(0, 0, 0, 25));
                agg::render_scanlines(ras1, &mut sl, &mut ren);

                ras2.reset();
                ras2.add_path(&mut ps2, 0);
                ren.set_color(agg::Rgba8::new_params(0, 150, 0, 25));
                agg::render_scanlines(ras2, &mut sl, &mut ren);

                self.render_scanline_boolean(rbuf, ras1, ras2);
            }
            1 => {
                //------------------------------------
                // Closed stroke
                //
                let mut ps1 = agg::PathStorage::new();
                let mut ps2 = agg::PathStorage::new();

                let x = self.x - self.util.borrow().initial_width() / 2.0 + 100.0;
                let y = self.y - self.util.borrow().initial_height() / 2.0 + 100.0;
                ps1.move_to(x + 140.0, y + 145.0);
                ps1.line_to(x + 225.0, y + 44.0);
                ps1.line_to(x + 296.0, y + 219.0);
                ps1.close_polygon(0);

                ps1.line_to(x + 226.0, y + 289.0);
                ps1.line_to(x + 82.0, y + 292.0);

                ps1.move_to(x + 220.0 - 50.0, y + 222.0);
                ps1.line_to(x + 363.0 - 50.0, y + 249.0);
                ps1.line_to(x + 265.0 - 50.0, y + 331.0);
                ps1.close_polygon(0);

                ps2.move_to(100.0 + 32.0, 100.0 + 77.0);
                ps2.line_to(100.0 + 473.0, 100.0 + 263.0);
                ps2.line_to(100.0 + 351.0, 100.0 + 290.0);
                ps2.line_to(100.0 + 354.0, 100.0 + 374.0);
                ps2.close_polygon(0);

                ras1.reset();
                ras1.add_path(&mut ps1, 0);
                ren.set_color(agg::Rgba8::new_params(0, 0, 0, 25));
                agg::render_scanlines(ras1, &mut sl, &mut ren);

                let mut stroke: agg::ConvStroke<_> = agg::ConvStroke::new_owned(ps2);
                stroke.set_width(15.0);

                ras2.reset();
                ras2.add_path(&mut stroke, 0);
                ren.set_color(agg::Rgba8::new_params(0, 150, 0, 25));
                agg::render_scanlines(ras2, &mut sl, &mut ren);

                self.render_scanline_boolean(rbuf, ras1, ras2);
            }
            2 => {
                //------------------------------------
                // Great Britain and Arrows
                //
                let mut gb_poly = agg::PathStorage::new();
                let mut arrows = agg::PathStorage::new();
                make_gb_poly(&mut gb_poly);
                make_arrows(&mut arrows);

                let mut mtx1 = agg::TransAffine::new_default();
                let mut mtx2;
                mtx1 *= agg::TransAffine::trans_affine_translation(-1150., -1150.);
                mtx1 *= agg::TransAffine::trans_affine_scaling_eq(2.0);
                mtx2 = mtx1;
                mtx2 *= agg::TransAffine::trans_affine_translation(
                    self.x - self.util.borrow().initial_width() / 2.,
                    self.y - self.util.borrow().initial_height() / 2.,
                );

                let mut trans_gb_poly = agg::ConvTransform::new_owned(gb_poly, mtx1);
                let mut trans_arrows = agg::ConvTransform::new_owned(arrows, mtx2);

                ras2.add_path(&mut trans_gb_poly, 0);
                ren.set_color(agg::Rgba8::new_params(125, 125, 0, 25));
                agg::render_scanlines(ras2, &mut sl, &mut ren);

                let mut stroke_gb_poly: agg::ConvStroke<_> =
                    agg::ConvStroke::new_owned(trans_gb_poly);
                stroke_gb_poly.set_width(0.1);
                ras1.add_path(&mut stroke_gb_poly, 0);
                ren.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
                agg::render_scanlines(ras1, &mut sl, &mut ren);

                ras2.add_path(&mut trans_arrows, 0);
                ren.set_color(agg::Rgba8::new_params(0, 125, 125, 25));
                agg::render_scanlines(ras2, &mut sl, &mut ren);

                ras1.reset();
                ras1.add_path(stroke_gb_poly.source_mut(), 0);

                self.render_scanline_boolean(rbuf, ras1, ras2);
            }
            3 => {
                //------------------------------------
                // Great Britain and a Spiral
                //
                let sp = Spiral::new(self.x, self.y, 10., 150., 30., 0.0);
                let mut stroke: agg::ConvStroke<_> = agg::ConvStroke::new_owned(sp);
                stroke.set_width(15.0);

                let mut gb_poly = agg::PathStorage::new();
                make_gb_poly(&mut gb_poly);

                let mut mtx = agg::TransAffine::new_default();
                mtx *= agg::TransAffine::trans_affine_translation(-1150., -1150.);
                mtx *= agg::TransAffine::trans_affine_scaling_eq(2.0);
                mtx *= *self.util.borrow().trans_affine_resizing();

                let mut trans_gb_poly = agg::ConvTransform::new_owned(gb_poly, mtx);

                ras1.add_path(&mut trans_gb_poly, 0);
                ren.set_color(agg::Rgba8::new_params(125, 125, 0, 25));
                agg::render_scanlines(ras1, &mut sl, &mut ren);

                let mut stroke_gb_poly: agg::ConvStroke<_> =
                    agg::ConvStroke::new_owned(trans_gb_poly);
                stroke_gb_poly.set_width(0.1);
                ras1.reset();
                ras1.add_path(&mut stroke_gb_poly, 0);
                ren.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
                agg::render_scanlines(ras1, &mut sl, &mut ren);

                ras2.reset();
                ras2.add_path(&mut stroke, 0);
                ren.set_color(agg::Rgba8::new_params(0, 125, 125, 25));
                agg::render_scanlines(ras2, &mut sl, &mut ren);

                ras1.reset();
                ras1.add_path(stroke_gb_poly.source_mut(), 0);
                self.render_scanline_boolean(rbuf, ras1, ras2);
            }
            4 => {
                //------------------------------------
                // Spiral and glyph
                //
                let sp = Spiral::new(self.x, self.y, 10., 150., 30., 0.0);
                let mut stroke: agg::ConvStroke<_> = agg::ConvStroke::new_owned(sp);
                stroke.set_width(15.0);

                let mut glyph = agg::PathStorage::new();
                glyph.move_to(28.47, 6.45);
                glyph.curve3_ctrl(21.58, 1.12, 19.82, 0.29);
                glyph.curve3_ctrl(17.19, -0.93, 14.21, -0.93);
                glyph.curve3_ctrl(9.57, -0.93, 6.57, 2.25);
                glyph.curve3_ctrl(3.56, 5.42, 3.56, 10.60);
                glyph.curve3_ctrl(3.56, 13.87, 5.03, 16.26);
                glyph.curve3_ctrl(7.03, 19.58, 11.99, 22.51);
                glyph.curve3_ctrl(16.94, 25.44, 28.47, 29.64);
                glyph.line_to(28.47, 31.40);
                glyph.curve3_ctrl(28.47, 38.09, 26.34, 40.58);
                glyph.curve3_ctrl(24.22, 43.07, 20.17, 43.07);
                glyph.curve3_ctrl(17.09, 43.07, 15.28, 41.41);
                glyph.curve3_ctrl(13.43, 39.75, 13.43, 37.60);
                glyph.line_to(13.53, 34.77);
                glyph.curve3_ctrl(13.53, 32.52, 12.38, 31.30);
                glyph.curve3_ctrl(11.23, 30.08, 9.38, 30.08);
                glyph.curve3_ctrl(7.57, 30.08, 6.42, 31.35);
                glyph.curve3_ctrl(5.27, 32.62, 5.27, 34.81);
                glyph.curve3_ctrl(5.27, 39.01, 9.57, 42.53);
                glyph.curve3_ctrl(13.87, 46.04, 21.63, 46.04);
                glyph.curve3_ctrl(27.59, 46.04, 31.40, 44.04);
                glyph.curve3_ctrl(34.28, 42.53, 35.64, 39.31);
                glyph.curve3_ctrl(36.52, 37.21, 36.52, 30.71);
                glyph.line_to(36.52, 15.53);
                glyph.curve3_ctrl(36.52, 9.13, 36.77, 7.69);
                glyph.curve3_ctrl(37.01, 6.25, 37.57, 5.76);
                glyph.curve3_ctrl(38.13, 5.27, 38.87, 5.27);
                glyph.curve3_ctrl(39.65, 5.27, 40.23, 5.62);
                glyph.curve3_ctrl(41.26, 6.25, 44.19, 9.18);
                glyph.line_to(44.19, 6.45);
                glyph.curve3_ctrl(38.72, -0.88, 33.74, -0.88);
                glyph.curve3_ctrl(31.35, -0.88, 29.93, 0.78);
                glyph.curve3_ctrl(28.52, 2.44, 28.47, 6.45);
                glyph.close_polygon(0);

                glyph.move_to(28.47, 9.62);
                glyph.line_to(28.47, 26.66);
                glyph.curve3_ctrl(21.09, 23.73, 18.95, 22.51);
                glyph.curve3_ctrl(15.09, 20.36, 13.43, 18.02);
                glyph.curve3_ctrl(11.77, 15.67, 11.77, 12.89);
                glyph.curve3_ctrl(11.77, 9.38, 13.87, 7.06);
                glyph.curve3_ctrl(15.97, 4.74, 18.70, 4.74);
                glyph.curve3_ctrl(22.41, 4.74, 28.47, 9.62);
                glyph.close_polygon(0);

                let mut mtx = agg::TransAffine::new_default();
                mtx *= agg::TransAffine::trans_affine_scaling_eq(4.0);
                mtx *= agg::TransAffine::trans_affine_translation(220., 200.);
                let trans = agg::ConvTransform::new_owned(glyph, mtx);
                let mut curve: agg::ConvCurve<_> = agg::ConvCurve::new_owned(trans);

                ras1.reset();
                ras1.add_path(&mut stroke, 0);
                ren.set_color(agg::Rgba8::new_params(0, 0, 0, 25));
                agg::render_scanlines(ras1, &mut sl, &mut ren);

                ras2.reset();
                ras2.add_path(&mut curve, 0);
                ren.set_color(agg::Rgba8::new_params(0, 150, 0, 25));
                agg::render_scanlines(ras2, &mut sl, &mut ren);

                self.render_scanline_boolean(rbuf, ras1, ras2);
            }
            _ => {}
        }

        return 0;
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Scanline Boolean");

    if plat.init(655, 520, WindowFlag::Resize as u32) {
        plat.run();
    }
}

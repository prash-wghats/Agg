use crate::ctrl::rbox::*;
use crate::platform::*;
use misc::{make_arrows::make_arrows, make_gb_poly::make_gb_poly};

use agg::alpha_mask_u8::*;
use agg::basics::{deg2rad, PathCmd, PathFlag};
use agg::color_rgba::*;
use agg::conv_curve::*;
use agg::conv_stroke::*;
use agg::conv_transform::ConvTransform;
use agg::pixfmt_gray::*;
use agg::TransAffine;

use agg::gsv_text::*;

use agg::math_stroke::LineCap;
use agg::path_storage::*;
use agg::pixfmt_amask_adaptor::PixAmaskAdaptor;
use agg::rasterizer_scanline_aa::RasterizerScanlineAa;
use agg::renderer_scanline::render_scanlines;
use agg::rendering_buffer::RenderBuf;
use agg::scanline_p::ScanlineP8;
use agg::{RasterScanLine, RenderBuffer, RendererScanlineColor, VertexSource};
use ctrl::render_ctrl;

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
    _step: f64,
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
            _step: step,
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

struct Application {
    polygons: Rc<RefCell<Rbox<'static, Rgba8>>>,
    operation: Rc<RefCell<Rbox<'static, Rgba8>>>,
    alpha_buf: *mut u8,
    //alpha_mask_rbuf: RenderBuf,
    alpha_mask: AmaskNoClipGray8,
    ras: RasterizerScanlineAa,
    sl: ScanlineP8,
    x: f64,
    y: f64,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let mut polygons = Rbox::new(5.0, 5.0, 5.0 + 205.0, 110.0, !flip_y);
        let mut operation = Rbox::new(555.0, 5.0, 555.0 + 80.0, 55.0, !flip_y);
        operation.add_item("AND");
        operation.add_item("SUB");
        operation.set_cur_item(0);
        operation.no_transform();

        polygons.add_item("Two Simple Paths");
        polygons.add_item("Closed Stroke");
        polygons.add_item("Great Britain and Arrows");
        polygons.add_item("Great Britain and Spiral");
        polygons.add_item("Spiral and Glyph");
        polygons.set_cur_item(1);
        polygons.no_transform();

        let polygons = ctrl_ptr(polygons);
        let operation = ctrl_ptr(operation);
        Application {
            polygons: polygons.clone(),
            operation: operation.clone(),
            alpha_buf: std::ptr::null_mut(),
            //alpha_mask_rbuf: RenderBuf::new_default(),
            alpha_mask: AmaskNoClipGray8::new(RenderBuf::new_default()),
            ras: RasterizerScanlineAa::new(),
            sl: ScanlineP8::new(),
            x: 0.0,
            y: 0.0,
            ctrls: CtrlContainer {
                ctrl: vec![polygons, operation],
                cur_ctrl: -1,
                num_ctrl: 2,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        self.x = self.util.borrow().width() / 2.0;
        self.y = self.util.borrow().height() / 2.0;
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pf = agg::PixBgr24::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pf);
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        self.render(ren_base.ren_mut().rbuf_mut());

        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.polygons.borrow_mut(),
        );
        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.operation.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.x = x as f64;
            self.y = y as f64;
            //self.return true
            return Draw::Yes;
        }

        if flags & InputFlag::MouseRight as u32 != 0 {
            let buf = format!("{} {}", x, y);
            self.util.borrow().message(&buf);
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

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }
}

impl Application {
    fn render(&mut self, rbuf: &mut RenderBuf) -> u32 {
        let mut pf = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pf);
        let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        let m = self.polygons.borrow_mut().cur_item();
        match m {
            0 => {
                let mut ps1 = PathStorage::new();
                let mut ps2 = PathStorage::new();

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
                ps1.line_to(x + 268.0, y + 309.0);
                ps1.line_to(x + 325.0, y + 261.0);

                ps1.move_to(x + 259.0, y + 259.0);
                ps1.line_to(x + 273.0, y + 288.0);
                ps1.line_to(x + 298.0, y + 266.0);

                ps2.move_to(100.0 + 32.0, 100.0 + 77.0);
                ps2.line_to(100.0 + 473.0, 100.0 + 263.0);
                ps2.line_to(100.0 + 351.0, 100.0 + 290.0);
                ps2.line_to(100.0 + 354.0, 100.0 + 374.0);

                self.ras.reset();
                self.ras.add_path(&mut ps1, 0);
                ren.set_color(Rgba8::new_params(0, 0, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.ras.reset();
                self.ras.add_path(&mut ps2, 0);
                ren.set_color(Rgba8::new_params(0, 150, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.generate_alpha_mask(&mut ps1, rbuf);
                self.perform_rendering(&mut ps2, rbuf);
            }
            1 => {
                let mut ps1 = PathStorage::new();
                let mut ps2 = PathStorage::new();

                let x = self.x - self.util.borrow().initial_width() / 2.0 + 100.0;
                let y = self.y - self.util.borrow().initial_height() / 2.0 + 100.0;
                ps1.move_to(x + 140.0, y + 145.0);
                ps1.line_to(x + 225.0, y + 44.0);
                ps1.line_to(x + 296.0, y + 219.0);
                ps1.close_polygon(0);

                ps1.line_to(x + 226.0, y + 289.0);
                ps1.line_to(x + 82.0, y + 292.0);

                ps1.move_to(x + 220.0 - 50.0, y + 222.0);
                ps1.line_to(x + 265.0 - 50.0, y + 331.0);
                ps1.line_to(x + 363.0 - 50.0, y + 249.0);
                ps1.close_polygon(PathFlag::Ccw as u32);

                ps2.move_to(100.0 + 32.0, 100.0 + 77.0);
                ps2.line_to(100.0 + 473.0, 100.0 + 263.0);
                ps2.line_to(100.0 + 351.0, 100.0 + 290.0);
                ps2.line_to(100.0 + 354.0, 100.0 + 374.0);
                ps2.close_polygon(0);

                self.ras.reset();
                self.ras.add_path(&mut ps1, 0);
                ren.set_color(Rgba8::new_params(0, 0, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                let mut stroke: ConvStroke<'_, _> = ConvStroke::new_owned(ps2);
                stroke.set_width(10.0);

                self.ras.reset();
                self.ras.add_path(&mut stroke, 0);
                ren.set_color(Rgba8::new_params(0, 150, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.generate_alpha_mask(&mut ps1, rbuf);
                self.perform_rendering(&mut stroke, rbuf);
            }
            2 => {
                //------------------------------------
                // Great Britain and Arrows
                //
                let mut gb_poly = PathStorage::new();
                let mut arrows = PathStorage::new();
                make_gb_poly(&mut gb_poly);
                make_arrows(&mut arrows);

                let mut mtx1 = TransAffine::new_default();
                let mut mtx2;
                mtx1 *= TransAffine::trans_affine_translation(-1150., -1150.);
                mtx1 *= TransAffine::trans_affine_scaling_eq(2.0);
                mtx2 = mtx1;
                mtx2 *= TransAffine::trans_affine_translation(
                    self.x - self.util.borrow().initial_width() / 2.,
                    self.y - self.util.borrow().initial_height() / 2.,
                );
                let mut trans_gb_poly = ConvTransform::new_owned(gb_poly, mtx1);
                let mut trans_arrows = ConvTransform::new_owned(arrows, mtx2);

                self.ras.add_path(&mut trans_gb_poly, 0);
                ren.set_color(Rgba8::new_params(127, 127, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                let mut stroke_gb_poly: ConvStroke<'_, _> = ConvStroke::new_owned(trans_gb_poly);
                stroke_gb_poly.set_width(0.1);
                self.ras.add_path(&mut stroke_gb_poly, 0);
                ren.set_color(Rgba8::new_params(0, 0, 0, 255));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.ras.add_path(&mut trans_arrows, 0);
                ren.set_color(Rgba8::new_params(0, 127, 127, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.generate_alpha_mask(stroke_gb_poly.source_mut(), rbuf);
                self.perform_rendering(&mut trans_arrows, rbuf);
            }
            3 => {
                //------------------------------------
                // Great Britain and a Spiral
                //
                let sp = Spiral::new(self.x, self.y, 10., 150., 30., 0.0);
                let mut stroke: ConvStroke<'_, _> = ConvStroke::new_owned(sp);
                stroke.set_width(15.0);

                let mut gb_poly = PathStorage::new();
                make_gb_poly(&mut gb_poly);

                let mut mtx = TransAffine::new_default();
                mtx *= TransAffine::trans_affine_translation(-1150., -1150.);
                mtx *= TransAffine::trans_affine_scaling_eq(2.0);

                let mut trans_gb_poly = ConvTransform::new_owned(gb_poly, mtx);

                self.ras.add_path(&mut trans_gb_poly, 0);
                ren.set_color(Rgba8::new_params(127, 127, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                let mut stroke_gb_poly: ConvStroke<'_, _> = ConvStroke::new_owned(trans_gb_poly);
                stroke_gb_poly.set_width(0.1);
                self.ras.add_path(&mut stroke_gb_poly, 0);
                ren.set_color(Rgba8::new_params(0, 0, 0, 255));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.ras.add_path(&mut stroke, 0);
                ren.set_color(Rgba8::new_params(0, 127, 127, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.generate_alpha_mask(stroke_gb_poly.source_mut(), rbuf);
                self.perform_rendering(&mut stroke, rbuf);
            }
            4 => {
                // Spiral and glyph
                let sp = Spiral::new(self.x, self.y, 10., 150., 30., 0.0);
                let mut stroke: ConvStroke<'_, _> = ConvStroke::new_owned(sp);
                stroke.set_width(15.0);

                let mut glyph = PathStorage::new();
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

                let mut mtx = TransAffine::new_default();
                mtx *= TransAffine::trans_affine_scaling_eq(4.0);
                mtx *= TransAffine::trans_affine_translation(220., 200.);

                let trans = ConvTransform::new_owned(glyph, mtx);
                let mut curve: ConvCurve<ConvTransform<PathBase<VertexStlStorage>, TransAffine>> =
                    ConvCurve::new_owned(trans);

                self.ras.reset();
                self.ras.add_path(&mut stroke, 0);
                ren.set_color(Rgba8::new_params(0, 0, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.ras.reset();
                self.ras.add_path(&mut curve, 0);
                ren.set_color(Rgba8::new_params(0, 150, 0, 25));
                render_scanlines(&mut self.ras, &mut self.sl, &mut ren);

                self.generate_alpha_mask(&mut stroke, rbuf);
                self.perform_rendering(&mut curve, rbuf);
            }
            _ => {}
        }

        0
    }

    fn draw_text(&mut self, x: f64, y: f64, str: &str, rbuf: &mut RenderBuf) {
        let mut pf = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pf);
        //ren_base.clear(&agg::Rgba8::new_params(255, 255, 240, 255));
        let mut ren_sl = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        let txt = GsvText::new();
        let mut txt_stroke: ConvStroke<'_, _> = ConvStroke::new_owned(txt);
        txt_stroke.set_width(1.5);
        txt_stroke.set_line_cap(LineCap::Round);
        txt_stroke.source_mut().set_size(10.0, 0.);
        txt_stroke.source_mut().set_start_point(x, y);
        txt_stroke.source_mut().set_text(str);
        self.ras.add_path(&mut txt_stroke, 0);
        ren_sl.set_color(Rgba8::new_params(0, 0, 0, 255));
        render_scanlines(&mut self.ras, &mut self.sl, &mut ren_sl);
    }

    fn generate_alpha_mask<VS: VertexSource>(&mut self, vs: &mut VS, rbuf: &mut RenderBuf) {
        let cx = self.util.borrow().width() as u32;
        let cy = self.util.borrow().height() as u32;

        self.alpha_buf = unsafe {
            std::alloc::alloc(
                std::alloc::Layout::from_size_align(cx as usize * cy as usize, 1).unwrap(),
            )
        };
        self.alpha_mask
            .rbuf_mut()
            .attach(self.alpha_buf, cx, cy, cx as i32);
        let mut pixf = PixGray8::new_borrowed(self.alpha_mask.rbuf_mut());
        let mut rb = agg::RendererBase::<PixGray8>::new_borrowed(&mut pixf);
        let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        self.util.borrow_mut().start_timer();
        if self.operation.borrow().cur_item() == 0 {
            ren.ren_mut().clear(&agg::Gray8::new_params(0, 255));
            ren.set_color(agg::Gray8::new_params(255, 255));
        } else {
            ren.ren_mut().clear(&agg::Gray8::new_params(255, 255));
            ren.set_color(agg::Gray8::new_params(0, 255));
        }

        self.ras.add_path(vs, 0);
        render_scanlines(&mut self.ras, &mut self.sl, &mut ren);
        let t1 = self.util.borrow().elapsed_time();

        let buf = format!("Generate AlphaMask: {:.3}ms", t1);
        self.draw_text(250., 20., &buf, rbuf);
    }

    fn perform_rendering<VS: VertexSource>(&mut self, vs: &mut VS, rbuf: &mut RenderBuf) {
        let mut pixf = agg::PixBgr24::new_borrowed(rbuf);
        let mut pixfa = PixAmaskAdaptor::new(&mut pixf, &mut self.alpha_mask);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixfa);
        //ren_base.clear(&agg::Rgba8::new_params(255, 255, 240, 255));
        let mut ren_sl = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        ren_sl.set_color(Rgba8::new_params(127, 0, 0, 127));

        self.util.borrow_mut().start_timer();
        self.ras.reset();
        self.ras.add_path(vs, 0);
        render_scanlines(&mut self.ras, &mut self.sl, &mut ren_sl);

        let t1 = self.util.borrow().elapsed_time();

        let buf = format!("Render with AlphaMask: {:.3}ms", t1);
        self.draw_text(250., 5., &buf, rbuf);
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    //plat.app_mut().init();
    plat.set_caption("AGG Example. Alpha-Mask as a Polygon Clipper");

    if plat.init(640, 520, WindowFlag::Resize as u32) {
        plat.run();
    }
}

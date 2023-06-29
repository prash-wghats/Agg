use crate::ctrl::cbox::*;
use crate::ctrl::polygon::*;
use crate::ctrl::rbox::*;
use crate::ctrl::slider::*;
use crate::platform::*;
use agg::basics::{uround, RectD};
use agg::blur::*;
use agg::bounding_rect::*;
use agg::color_gray::*;
use agg::color_rgba::*;
use agg::conv_curve::*;
use agg::conv_stroke::*;
use agg::conv_transform::ConvTransform;
use agg::gsv_text::*;
use agg::path_storage::*;
use agg::pixfmt_gray::*;
use agg::pixfmt_rgb::*;
use agg::rasterizer_scanline_aa::*;
use agg::renderer_scanline::*;
use agg::rendering_buffer::*;
use agg::scanline_p::*;
use agg::trans_affine::TransAffine;
use agg::trans_perspective::TransPerspective;
use ctrl::render_ctrl;

use agg::{Color, PixFmt, PixFmtGray, RasterScanLine};
mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = true;

struct Application {
    method: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    radius: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    shadow_ctrl: Rc<RefCell<Polygon<'static, agg::Rgba8>>>,
    channel_r: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    channel_g: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    channel_b: Rc<RefCell<Cbox<'static, agg::Rgba8>>>,
    //path: PathStorage,
    shape: ConvCurve<'static, PathStorage>,
    //shadow_trans: ConvTransform<'static,ConvCurve<'static,PathStorage>, TransPerspective>,
    ras: RasterizerScanlineAa,
    sl: ScanlineP8,
    rbuf2: RenderBuf,
    stack_blur: StackBlur<agg::Rgba8, StackBlurCalcRgb>,
    recursive_blur: RecursiveBlur<agg::Rgba8, RecursiveBlurCalcRgb>,
    shape_bounds: RectD,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {}
type PixfmtGray8R<'a> = AlphaBlendGray<'a, Gray8, BlenderGray<Gray8>, RenderBuf, 3, 2>;
type PixfmtGray8G<'a> = AlphaBlendGray<'a, Gray8, BlenderGray<Gray8>, RenderBuf, 3, 1>;
type PixfmtGray8B<'a> = AlphaBlendGray<'a, Gray8, BlenderGray<Gray8>, RenderBuf, 3, 0>;

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut method = Rbox::new(10.0, 10.0, 130.0, 70.0, !flip_y);
        let mut radius = Slider::new(
            130. + 10.0,
            10.0 + 4.0,
            130. + 300.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        );
        let mut shadow_ctrl = Polygon::<agg::Rgba8>::new(4, 5.0);
        let channel_r = Cbox::new(10.0, 80.0, "Red", !flip_y);
        let mut channel_g = Cbox::new(10.0, 95.0, "Green", !flip_y);
        let channel_b = Cbox::new(10.0, 110.0, "Blue", !flip_y);
        let mut path = PathStorage::new();

        method.set_text_size(8., 0.);
        method.add_item("Stack Blur");
        method.add_item("Recursive Blur");
        method.add_item("Channels");
        method.set_cur_item(0);

        radius.set_range(0.0, 40.0);
        radius.set_value(15.0);
        radius.set_label("Blur Radius=%1.2f");

        channel_g.set_status(true);

        path.remove_all();
        path.move_to(28.47, 6.45);
        path.curve3_ctrl(21.58, 1.12, 19.82, 0.29);
        path.curve3_ctrl(17.19, -0.93, 14.21, -0.93);
        path.curve3_ctrl(9.57, -0.93, 6.57, 2.25);
        path.curve3_ctrl(3.56, 5.42, 3.56, 10.60);
        path.curve3_ctrl(3.56, 13.87, 5.03, 16.26);
        path.curve3_ctrl(7.03, 19.58, 11.99, 22.51);
        path.curve3_ctrl(16.94, 25.44, 28.47, 29.64);
        path.line_to(28.47, 31.40);
        path.curve3_ctrl(28.47, 38.09, 26.34, 40.58);
        path.curve3_ctrl(24.22, 43.07, 20.17, 43.07);
        path.curve3_ctrl(17.09, 43.07, 15.28, 41.41);
        path.curve3_ctrl(13.43, 39.75, 13.43, 37.60);
        path.line_to(13.53, 34.77);
        path.curve3_ctrl(13.53, 32.52, 12.38, 31.30);
        path.curve3_ctrl(11.23, 30.08, 9.38, 30.08);
        path.curve3_ctrl(7.57, 30.08, 6.42, 31.35);
        path.curve3_ctrl(5.27, 32.62, 5.27, 34.81);
        path.curve3_ctrl(5.27, 39.01, 9.57, 42.53);
        path.curve3_ctrl(13.87, 46.04, 21.63, 46.04);
        path.curve3_ctrl(27.59, 46.04, 31.40, 44.04);
        path.curve3_ctrl(34.28, 42.53, 35.64, 39.31);
        path.curve3_ctrl(36.52, 37.21, 36.52, 30.71);
        path.line_to(36.52, 15.53);
        path.curve3_ctrl(36.52, 9.13, 36.77, 7.69);
        path.curve3_ctrl(37.01, 6.25, 37.57, 5.76);
        path.curve3_ctrl(38.13, 5.27, 38.87, 5.27);
        path.curve3_ctrl(39.65, 5.27, 40.23, 5.62);
        path.curve3_ctrl(41.26, 6.25, 44.19, 9.18);
        path.line_to(44.19, 6.45);
        path.curve3_ctrl(38.72, -0.88, 33.74, -0.88);
        path.curve3_ctrl(31.35, -0.88, 29.93, 0.78);
        path.curve3_ctrl(28.52, 2.44, 28.47, 6.45);
        path.close_polygon(0);

        path.move_to(28.47, 9.62);
        path.line_to(28.47, 26.66);
        path.curve3_ctrl(21.09, 23.73, 18.95, 22.51);
        path.curve3_ctrl(15.09, 20.36, 13.43, 18.02);
        path.curve3_ctrl(11.77, 15.67, 11.77, 12.89);
        path.curve3_ctrl(11.77, 9.38, 13.87, 7.06);
        path.curve3_ctrl(15.97, 4.74, 18.70, 4.74);
        path.curve3_ctrl(22.41, 4.74, 28.47, 9.62);
        path.close_polygon(0);

        let mut shape_mtx = TransAffine::new_default();
        shape_mtx *= TransAffine::trans_affine_scaling_eq(4.0);
        shape_mtx *= TransAffine::trans_affine_translation(150., 100.);
        path.transform(&shape_mtx, 0);
        let mut shape = ConvCurve::new_owned(path);
        let mut shape_bounds = RectD::new(0., 0., 0., 0.);
        bounding_rect_single(
            &mut shape,
            0,
            &mut shape_bounds.x1,
            &mut shape_bounds.y1,
            &mut shape_bounds.x2,
            &mut shape_bounds.y2,
        );

        *shadow_ctrl.xn_mut(0) = shape_bounds.x1;
        *shadow_ctrl.yn_mut(0) = shape_bounds.y1;
        *shadow_ctrl.xn_mut(1) = shape_bounds.x2;
        *shadow_ctrl.yn_mut(1) = shape_bounds.y1;
        *shadow_ctrl.xn_mut(2) = shape_bounds.x2;
        *shadow_ctrl.yn_mut(2) = shape_bounds.y2;
        *shadow_ctrl.xn_mut(3) = shape_bounds.x1;
        *shadow_ctrl.yn_mut(3) = shape_bounds.y2;
        shadow_ctrl.set_line_color(Rgba8::new_from_rgba(&Rgba::new_params(0., 0.3, 0.5, 0.3)));

        let method = Rc::new(RefCell::new(method));
        let radius = Rc::new(RefCell::new(radius));
        let shadow_ctrl = Rc::new(RefCell::new(shadow_ctrl));
        let channel_r = Rc::new(RefCell::new(channel_r));
        let channel_g = Rc::new(RefCell::new(channel_g));
        let channel_b = Rc::new(RefCell::new(channel_b));

        let app = Self {
            method: method.clone(),
            radius: radius.clone(),
            shadow_ctrl: shadow_ctrl.clone(),
            channel_r: channel_r.clone(),
            channel_g: channel_g.clone(),
            channel_b: channel_b.clone(),
            //path: path.clone(),
            shape: shape,
            //shadow_trans,
            //shape: shape,
            ras: RasterizerScanlineAa::new(),
            sl: ScanlineP8::new(),
            rbuf2: RenderBuf::new_default(),
            stack_blur: StackBlur::new(),
            recursive_blur: RecursiveBlur::new(),
            shape_bounds: shape_bounds,
            ctrls: CtrlContainer {
                ctrl: vec![method, radius, shadow_ctrl, channel_r, channel_g, channel_b],
                cur_ctrl: -1,
                num_ctrl: 6,
            },
            util: util,
        };

        app
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }
    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        Draw::No
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rb);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);

        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        self.ras.clip_box(
            0.,
            0.,
            self.util.borrow().width(),
            self.util.borrow().height() as f64,
        );
        let shadow_persp = TransPerspective::new_rect_to_quad(
            self.shape_bounds.x1,
            self.shape_bounds.y1,
            self.shape_bounds.x2,
            self.shape_bounds.y2,
            self.shadow_ctrl.borrow().polygon(),
        );

        /*self.shadow_trans.trans_mut().rect_to_quad(
            self.shape_bounds.x1,
            self.shape_bounds.y1,
            self.shape_bounds.x2,
            self.shape_bounds.y2,
            self.shadow_ctrl.borrow_mut().polygon(),
        );*/

        let mut shadow_trans = ConvTransform::new_borrowed(&mut self.shape, shadow_persp);

        // Render shadow
        self.ras.add_path(&mut shadow_trans, 0);
        render_scanlines_aa_solid(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &Rgba8::new_from_rgba(&Rgba::new_params(0.2, 0.3, 0.0, 1.0)),
        );

        // Calculate the bounding box and extend it by the blur radius
        let mut bbox = RectD::new(0.0, 0.0, 0.0, 0.0);
        bounding_rect_single(
            &mut shadow_trans,
            0,
            &mut bbox.x1,
            &mut bbox.y1,
            &mut bbox.x2,
            &mut bbox.y2,
        );

        bbox.x1 -= self.radius.borrow().value();
        bbox.y1 -= self.radius.borrow().value();
        bbox.x2 += self.radius.borrow().value();
        bbox.y2 += self.radius.borrow().value();

        if self.method.borrow().cur_item() == 1 {
            // The recursive blur method represents the true Gussian Blur,
            // with theoretically infinite kernel. The restricted window size
            // results in extra influence of edge pixels. It's impossible to
            // solve correctly, but extending the right and top areas to another
            // radius value produces fair result.
            //------------------
            bbox.x2 += self.radius.borrow().value();
            bbox.y2 += self.radius.borrow().value();
        }

        self.util.borrow_mut().start_timer();
        if self.method.borrow().cur_item() != 2 {
            // Create a new pixel renderer and attach it to the main one as a child image.
            // It returns true if the attachment suceeded. It fails if the rectangle
            // (bbox) is fully clipped.
            //------------------
            let mut pixf2 = agg::PixBgr24::new_borrowed(&mut self.rbuf2);
            if pixf2.attach_pixfmt(
                ren_base.ren(),
                bbox.x1 as i32,
                bbox.y1 as i32,
                bbox.x2 as i32,
                bbox.y2 as i32,
            ) {
                // Blur it
                if self.method.borrow().cur_item() == 0 {
                    // More general method, but 30-40% slower.
                    //------------------
                    //self.stack_blur.blur(&mut pixf2, uround(self.radius.borrow().value()) as u32);

                    // Faster, but bore specific.
                    // Works only for 8 bits per channel and only with radii <= 254.
                    //------------------
                    stack_blur_rgb24(
                        &mut pixf2,
                        self.radius.borrow().value() as u32,
                        self.radius.borrow().value() as u32,
                    );
                } else {
                    // True Gaussian Blur, 3-5 times slower than Stack Blur,
                    // but still constant time of radius. Very sensitive
                    // to precision, doubles are must here.
                    //------------------
                    self.recursive_blur
                        .blur(&mut pixf2, self.radius.borrow().value());
                }
            }
        } else {
            // Blur separate channels
            //------------------
            if self.channel_r.borrow().status() {
                let mut pixf2r = PixfmtGray8R::new_borrowed(&mut self.rbuf2);
                if pixf2r.attach_pixfmt(
                    ren_base.ren(),
                    bbox.x1 as i32,
                    bbox.y1 as i32,
                    bbox.x2 as i32,
                    bbox.y2 as i32,
                ) {
                    stack_blur_gray8(
                        &mut pixf2r,
                        self.radius.borrow().value() as u32,
                        self.radius.borrow().value() as u32,
                    );
                }
            }

            if self.channel_g.borrow().status() {
                let mut pixf2g = PixfmtGray8G::new_borrowed(&mut self.rbuf2);
                if pixf2g.attach_pixfmt(
                    ren_base.ren(),
                    bbox.x1 as i32,
                    bbox.y1 as i32,
                    bbox.x2 as i32,
                    bbox.y2 as i32,
                ) {
                    stack_blur_gray8(
                        &mut pixf2g,
                        self.radius.borrow().value() as u32,
                        self.radius.borrow().value() as u32,
                    );
                }
            }

            if self.channel_b.borrow().status() {
                let mut pixf2b = PixfmtGray8B::new_borrowed(&mut self.rbuf2);
                if pixf2b.attach_pixfmt(
                    ren_base.ren(),
                    bbox.x1 as i32,
                    bbox.y1 as i32,
                    bbox.x2 as i32,
                    bbox.y2 as i32,
                ) {
                    stack_blur_gray8(
                        &mut pixf2b,
                        self.radius.borrow().value() as u32,
                        self.radius.borrow().value() as u32,
                    );
                }
            }
        }
        let tm = self.util.borrow_mut().elapsed_time();

        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.shadow_ctrl.borrow_mut(),
        );

        // Render the shape itself
        //------------------
        self.ras.add_path(shadow_trans.source_mut(), 0);
        render_scanlines_aa_solid(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &Rgba8::new_from_rgba(&Rgba::new_params(0.6, 0.9, 0.7, 0.8)),
        );

        let mut t = GsvText::new();
        t.set_size(10.0, 0.);
        let buf = format!("{:.2} ms", tm);
        t.set_start_point(140.0, 30.0);
        t.set_text(&buf);

        let mut st: ConvStroke<'_, _> = ConvStroke::new_owned(t);
        st.set_width(1.5);

        self.ras.add_path(&mut st, 0);
        render_scanlines_aa_solid(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &Rgba8::new_from_rgba(&Rgba::new_params(0.0, 0.0, 0.0, 1.0)),
        );

        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.method.borrow_mut(),
        );
        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.radius.borrow_mut(),
        );
        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.channel_r.borrow_mut(),
        );
        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.channel_g.borrow_mut(),
        );
        render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut ren_base,
            &mut *self.channel_b.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    //plat.app_mut().init();
    plat.set_caption("AGG Example. Gaussian and Stack Blur");

    if plat.init(440, 330, WindowFlag::Resize as u32) {
        plat.run();
    }
}

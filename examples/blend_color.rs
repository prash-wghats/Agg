use agg::array::PodArray;
use agg::{Color, PixFmt, RasterScanLine, RenderBuf, RenderBuffer};

use crate::ctrl::polygon::Polygon;
use crate::ctrl::rbox::Rbox;
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

struct Application {
    method: Rc<RefCell<Rbox<'static, agg::Rgba8>>>,
    radius: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    shadow_ctrl: Rc<RefCell<Polygon<'static, agg::Rgba8>>>,
    //path: agg::PathStorage,
    shape: agg::ConvCurve<'static, agg::PathStorage>,
    ras: agg::RasterizerScanlineAa,
    sl: agg::ScanlineP8,
    shape_bounds: agg::RectD,
    gray8_buf: PodArray<u8>,
    gray8_rbuf: RenderBuf,
    gray8_rbuf2: RenderBuf,
    color_lut: PodArray<ColorType>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let method = ctrl_ptr(Rbox::new(10.0, 10.0, 130.0, 55.0, !flip_y));
        let radius = ctrl_ptr(Slider::new(
            130. + 10.0,
            10.0 + 4.0,
            130. + 300.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let shadow_ctrl = ctrl_ptr(Polygon::new(4, 5.0));
        let mut path = agg::PathStorage::new();

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

        let mut shape_mtx = agg::TransAffine::new_default();
        shape_mtx *= agg::TransAffine::trans_affine_scaling_eq(4.0);
        shape_mtx *= agg::TransAffine::trans_affine_translation(150., 100.);
        path.transform(&shape_mtx, 0);
        let mut shape = agg::ConvCurve::new_owned(path);
        let mut shape_bounds = agg::RectD::new(0., 0., 0., 0.);
        agg::bounding_rect_single(
            &mut shape,
            0,
            &mut shape_bounds.x1,
            &mut shape_bounds.y1,
            &mut shape_bounds.x2,
            &mut shape_bounds.y2,
        );
        *shadow_ctrl.borrow_mut().xn_mut(0) = shape_bounds.x1;
        *shadow_ctrl.borrow_mut().yn_mut(0) = shape_bounds.y1;
        *shadow_ctrl.borrow_mut().xn_mut(1) = shape_bounds.x2;
        *shadow_ctrl.borrow_mut().yn_mut(1) = shape_bounds.y1;
        *shadow_ctrl.borrow_mut().xn_mut(2) = shape_bounds.x2;
        *shadow_ctrl.borrow_mut().yn_mut(2) = shape_bounds.y2;
        *shadow_ctrl.borrow_mut().xn_mut(3) = shape_bounds.x1;
        *shadow_ctrl.borrow_mut().yn_mut(3) = shape_bounds.y2;
        shadow_ctrl
            .borrow_mut()
            .set_line_color(agg::Rgba8::new_from_rgba(&agg::Rgba::new_params(
                0., 0.3, 0.5, 0.3,
            )));

        let mut color_lut = PodArray::new();
        let p = &GRADIENT_COLOR;
        for i in 0..256 {
            color_lut.push(agg::Rgba8::new_params(
                p[i * 4 + 0] as u32,
                p[i * 4 + 1] as u32,
                p[i * 4 + 2] as u32,
                if i as u32 > 63 { 255 } else { i as u32 * 4 },
                //p[i*4+3] as u32,
            ).into());
            //m_color_lut[i].premultiply();
        }

        let app = Application {
            method: method.clone(),
            radius: radius.clone(),
            shadow_ctrl: shadow_ctrl.clone(),
            //path: agg::PathStorage::new(),
            shape: shape,
            ras: agg::RasterizerScanlineAa::new(),
            sl: agg::ScanlineP8::new(),
            shape_bounds: shape_bounds,
            gray8_buf: PodArray::new(),
            gray8_rbuf: agg::RenderBuf::new_default(),
            gray8_rbuf2: agg::RenderBuf::new_default(),
            color_lut: color_lut,
            ctrls: CtrlContainer {
                ctrl: vec![method, radius, shadow_ctrl],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        };
        app.method.borrow_mut().set_text_size(8., 0.);
        app.method.borrow_mut().add_item("Single Color");
        app.method.borrow_mut().add_item("Color LUT");
        app.method.borrow_mut().set_cur_item(1);

        app.radius.borrow_mut().set_range(0.0, 40.0);
        app.radius.borrow_mut().set_value(15.0);
        app.radius.borrow_mut().set_label("Blur Radius=%1.2f");

        app
    }

    fn on_resize(&mut self, sx: i32, sy: i32) {
        self.gray8_buf.resize((sx * sy) as usize, 0);
        self.gray8_rbuf
            .attach(self.gray8_buf.as_mut_ptr(), sx as u32, sy as u32, sx);
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf_gray8 = agg::PixGray8::new_borrowed(&mut self.gray8_rbuf);
        let mut renb_gray8 = agg::RendererBase::new_borrowed(&mut pixf_gray8);
        renb_gray8.clear(&agg::Gray8::new_params(0, 255));
        let w = self.util.borrow_mut().width();
        self.ras
            .clip_box(0., 0., w, self.util.borrow_mut().height());

        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut renb = agg::RendererBase::new_borrowed(&mut pixf);
        renb.clear(&agg::Rgba8::new_params(255, 242, 242, 255).into());

        let shadow_persp = agg::TransPerspective::new_rect_to_quad(
            self.shape_bounds.x1,
            self.shape_bounds.y1,
            self.shape_bounds.x2,
            self.shape_bounds.y2,
            self.shadow_ctrl.borrow().polygon(),
        );

        let mut shadow_trans = agg::ConvTransform::new_borrowed(&mut self.shape, shadow_persp);

        self.util.borrow_mut().start_timer();

        self.ras.add_path(&mut shadow_trans, 0);
        agg::render_scanlines_aa_solid(
            &mut self.ras,
            &mut self.sl,
            &mut renb_gray8,
            &agg::Gray8::new_params(255, 255),
        );

        let mut bbox = agg::RectD::new(0.0, 0.0, 0.0, 0.0);
        agg::bounding_rect_single(
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

        if bbox.clip(&agg::RectD::new(
            0.0,
            0.0,
            self.util.borrow().width(),
            self.util.borrow().height(),
        )) {
            let mut pixf2 = agg::PixGray8::new_borrowed(&mut self.gray8_rbuf2);
            if pixf2.attach_pixfmt(
                &mut pixf_gray8,
                bbox.x1 as i32,
                bbox.y1 as i32,
                bbox.x2 as i32,
                bbox.y2 as i32,
            ) {
                agg::stack_blur_gray8(
                    &mut pixf2,
                    self.radius.borrow().value() as u32,
                    self.radius.borrow().value() as u32,
                );
            }
            if self.method.borrow_mut().cur_item() == 0 {
                renb.blend_from_color(
                    &mut pixf2,
                    &agg::Rgba8::new_params(0, 100, 0, 255).into(),
                    None,
                    bbox.x1 as i32,
                    bbox.y1 as i32,
                    255,
                );
            } else {
                renb.blend_from_lut(
                    &mut pixf2,
                    &self.color_lut,
                    None,
                    bbox.x1 as i32,
                    bbox.y1 as i32,
                    255,
                );
            }
        }

        let tm = self.util.borrow_mut().elapsed_time();

        let mut t = agg::GsvText::new();
        t.set_size(10.0, 0.);
        let buf = format!("{:.2} ms", tm);
        t.set_start_point(140.0, 30.0);
        t.set_text(&buf);
        let mut st: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(t);
        st.set_width(1.5);

        self.ras.add_path(&mut st, 0);
        agg::render_scanlines_aa_solid(
            &mut self.ras,
            &mut self.sl,
            &mut renb,
            &ColorType::new_from_rgba(&agg::Rgba::new_params(0., 0., 0., 1.)),
        );

        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut renb,
            &mut *self.method.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut renb,
            &mut *self.radius.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.ras,
            &mut self.sl,
            &mut renb,
            &mut *self.shadow_ctrl.borrow_mut(),
        );
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

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Gaussian and Stack Blur");

    if plat.init(440, 330, WindowFlag::Resize as u32) {
        plat.run();
    }
}

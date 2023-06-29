use crate::platform::*;

use agg::{
    RasterScanLine, RenderBuf, RenderBuffer, RendererScanlineColor,
};

mod ctrl;
mod platform;
use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;


struct Application {
    polygon_angle: Ptr<Slider<'static, agg::Rgba8>>,
    polygon_scale: Ptr<Slider<'static, agg::Rgba8>>,
    image_angle: Ptr<Slider<'static, agg::Rgba8>>,
    image_scale: Ptr<Slider<'static, agg::Rgba8>>,
    rotate_polygon: Ptr<Cbox<'static, agg::Rgba8>>,
    rotate_image: Ptr<Cbox<'static, agg::Rgba8>>,
    example: Ptr<Rbox<'static, agg::Rgba8>>,
    image_center_x: f64,
    image_center_y: f64,
    polygon_cx: f64,
    polygon_cy: f64,
    image_cx: f64,
    image_cy: f64,
    dx: f64,
    dy: f64,
    flag: i32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}
impl Application {
    fn create_star(&mut self, ps: &mut agg::PathStorage) {
        let mut r = self.util.borrow().initial_width();
        if self.util.borrow().initial_height() < r {
            r = self.util.borrow().initial_height();
        }

        let r1 = r / 3.0 - 8.0;
        let r2 = r1 / 1.45;
        let nr = 14;

        for i in 0..nr {
            let a = PI * 2.0 * (i as f64) / (nr as f64) - PI / 2.0;
            let dx = a.cos();
            let dy = a.sin();

            if i & 1 == 1 {
                ps.line_to(self.polygon_cx + dx * r1, self.polygon_cy + dy * r1);
            } else {
                if i != 0 {
                    ps.line_to(self.polygon_cx + dx * r2, self.polygon_cy + dy * r2);
                } else {
                    ps.move_to(self.polygon_cx + dx * r2, self.polygon_cy + dy * r2);
                }
            }
        }
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let polygon_angle = ctrl_ptr(Slider::new(5., 5., 145., 11., !flip_y));
        let polygon_scale = ctrl_ptr(Slider::new(5., 5. + 14., 145., 12. + 14., !flip_y));
        let image_angle = ctrl_ptr(Slider::new(155., 5., 300., 12., !flip_y));
        let image_scale = ctrl_ptr(Slider::new(155., 5. + 14., 300., 12. + 14., !flip_y));
        let rotate_polygon = ctrl_ptr(Cbox::new(5., 5. + 14. + 14., "Rotate Polygon", !flip_y));
        let rotate_image = ctrl_ptr(Cbox::new(5., 5. + 14. + 14. + 14., "Rotate Image", !flip_y));
        let example = ctrl_ptr(Rbox::new(
            -3.0,
            14. + 14. + 14. + 14.,
            -3.0,
            14. + 14. + 14. + 14.,
            !flip_y,
        ));
        polygon_angle.borrow_mut().set_label("Polygon Angle=%3.2f");
        polygon_scale.borrow_mut().set_label("Polygon Scale=%3.2f");
        polygon_angle.borrow_mut().set_range(-180.0, 180.0);
        polygon_scale.borrow_mut().set_range(0.1, 5.0);
        polygon_scale.borrow_mut().set_value(1.0);

        image_angle.borrow_mut().set_label("Image Angle=%3.2f");
        image_scale.borrow_mut().set_label("Image Scale=%3.2f");
        image_angle.borrow_mut().set_range(-180.0, 180.0);
        image_scale.borrow_mut().set_range(0.1, 5.0);
        image_scale.borrow_mut().set_value(1.0);

        example.borrow_mut().add_item("0");
        example.borrow_mut().add_item("1");
        example.borrow_mut().add_item("2");
        example.borrow_mut().add_item("3");
        example.borrow_mut().add_item("4");
        example.borrow_mut().add_item("5");
        example.borrow_mut().add_item("6");
        example.borrow_mut().set_cur_item(0);
        Application {
            polygon_angle: polygon_angle.clone(),
            polygon_scale: polygon_scale.clone(),
            image_angle: image_angle.clone(),
            image_scale: image_scale.clone(),
            rotate_polygon: rotate_polygon.clone(),
            rotate_image: rotate_image.clone(),
            example: example.clone(),

            image_center_x: 0.0,
            image_center_y: 0.0,
            polygon_cx: 0.0,
            polygon_cy: 0.0,
            image_cx: 0.0,
            image_cy: 0.0,
            dx: 0.0,
            dy: 0.0,
            flag: 0,
            ctrls: CtrlContainer {
                ctrl: vec![
                    polygon_angle,
                    polygon_scale,
                    image_angle,
                    image_scale,
                    rotate_polygon,
                    rotate_image,
                    example,
                ],
                cur_ctrl: -1,
                num_ctrl: 7,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        self.image_center_x = self.util.borrow().initial_width() / 2.0;
        self.image_center_y = self.util.borrow().initial_height() / 2.0;

        self.image_cx = self.util.borrow().initial_width() / 2.0;
        self.polygon_cx = self.image_cx;
        self.image_cy = self.util.borrow().initial_height() / 2.0;
        self.polygon_cy = self.image_cy;
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.flag = 0;
        Draw::No
    }

    fn on_ctrl_change(&mut self, _rb: &mut agg::RenderBuf) {
        if self.rotate_polygon.borrow().status() || self.rotate_image.borrow().status() {
            self.util.borrow_mut().set_wait_mode(false);
        } else {
            self.util.borrow_mut().set_wait_mode(true);
        }
        //return Draw::Yes;
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgra32::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut pixf_img =
            agg::PixBgra32::new_owned(self.util.borrow_mut().rbuf_img_mut(0).clone());

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut rs = agg::RendererScanlineAASolid::new_borrowed(&mut rb);

        let mut image_mtx = agg::TransAffine::new_default();
        let mut polygon_mtx = agg::TransAffine::new_default();

        polygon_mtx *=
            agg::TransAffine::trans_affine_translation(-self.polygon_cx, -self.polygon_cy);
        polygon_mtx *= agg::TransAffine::trans_affine_rotation(
            self.polygon_angle.borrow().value() * PI / 180.0,
        );
        polygon_mtx *=
            agg::TransAffine::trans_affine_scaling_eq(self.polygon_scale.borrow().value());
        polygon_mtx *= agg::TransAffine::trans_affine_translation(self.polygon_cx, self.polygon_cy);

        match self.example.borrow().cur_item() {
            0 => {} // Example 0, Identity Matrix
            1 => {
                image_mtx *= agg::TransAffine::trans_affine_translation(
                    -self.image_center_x,
                    -self.image_center_y,
                );
                image_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.polygon_angle.borrow().value() * PI / 180.0,
                );
                image_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.polygon_scale.borrow().value());
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(self.polygon_cx, self.polygon_cy);
                image_mtx.invert();
            }
            2 => {
                image_mtx *= agg::TransAffine::trans_affine_translation(
                    -self.image_center_x,
                    -self.image_center_y,
                );
                image_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.image_angle.borrow().value() * PI / 180.0,
                );
                image_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.image_scale.borrow().value());
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(self.image_cx, self.image_cy);
                image_mtx.invert();
            }
            3 => {
                image_mtx *= agg::TransAffine::trans_affine_translation(
                    -self.image_center_x,
                    -self.image_center_y,
                );
                image_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.image_angle.borrow().value() * PI / 180.0,
                );
                image_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.image_scale.borrow().value());
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(self.polygon_cx, self.polygon_cy);
                image_mtx.invert();
            }
            4 => {
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(-self.image_cx, -self.image_cy);
                image_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.polygon_angle.borrow().value() * PI / 180.0,
                );
                image_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.polygon_scale.borrow().value());
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(self.polygon_cx, self.polygon_cy);
                image_mtx.invert();
            }
            5 => {
                image_mtx *= agg::TransAffine::trans_affine_translation(
                    -self.image_center_x,
                    -self.image_center_y,
                );
                image_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.image_angle.borrow().value() * PI / 180.0,
                );
                image_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.polygon_angle.borrow().value() * PI / 180.0,
                );
                image_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.image_scale.borrow().value());
                image_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.polygon_scale.borrow().value());
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(self.image_cx, self.image_cy);
                image_mtx.invert();
            }
            6 => {
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(-self.image_cx, -self.image_cy);
                image_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.image_angle.borrow().value() * PI / 180.0,
                );
                image_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.image_scale.borrow().value());
                image_mtx *=
                    agg::TransAffine::trans_affine_translation(self.image_cx, self.image_cy);
                image_mtx.invert();
            }
            _ => panic!("Invalid example type!"),
        }

        let mut interpolator: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(image_mtx);
        let mut sg = agg::SpanImageFilterRgbaBilinearClip::new(
            &mut pixf_img,
            &mut interpolator,
            agg::Rgba8::new_params(255, 255, 255, 255),
        );
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineU8::new();
        let mut ps = agg::PathStorage::new();
        self.create_star(&mut ps);

        let mut tr = agg::ConvTransform::new_owned(ps, polygon_mtx);
        ras.add_path(&mut tr, 0);
        let mut sa = agg::VecSpan::new();
        agg::render_scanlines_aa(&mut ras, &mut sl, rs.ren_mut(), &mut sa, &mut sg);

        let e1 = agg::Ellipse::new_ellipse(self.image_cx, self.image_cy, 5., 5., 20, false);
        let mut e2 = agg::Ellipse::new_ellipse(self.image_cx, self.image_cy, 2., 2., 20, false);
        let mut c1: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(e1);
        rs.set_color(agg::Rgba8::new_params(175, 200, 0, 255));
        ras.add_path(c1.source_mut(), 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);

        rs.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        ras.add_path(&mut c1, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);

        ras.add_path(&mut e2, 0);
        agg::render_scanlines(&mut ras, &mut sl, &mut rs);

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.polygon_angle.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.polygon_scale.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.image_angle.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.image_scale.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.rotate_polygon.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rb,
            &mut *self.rotate_image.borrow_mut(),
        );
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.example.borrow_mut());
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let distance = (x as f64 - self.image_cx).powi(2) + (y as f64 - self.image_cy).powi(2);
            if distance < 5.0 {
                self.dx = x as f64 - self.image_cx;
                self.dy = y as f64 - self.image_cy;
                self.flag = 1;
            } else {
                let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
                let mut polygon_mtx = agg::TransAffine::new_default();

                polygon_mtx *=
                    agg::TransAffine::trans_affine_translation(-self.polygon_cx, -self.polygon_cy);
                polygon_mtx *= agg::TransAffine::trans_affine_rotation(
                    self.polygon_angle.borrow().value() * PI / 180.0,
                );
                polygon_mtx *=
                    agg::TransAffine::trans_affine_scaling_eq(self.polygon_scale.borrow().value());
                polygon_mtx *=
                    agg::TransAffine::trans_affine_translation(self.polygon_cx, self.polygon_cy);

                let mut ps = agg::PathStorage::new();
                self.create_star(&mut ps);

                let mut tr = agg::ConvTransform::new_owned(ps, polygon_mtx);
                ras.add_path(&mut tr, 0);
                if ras.hit_test(x, y) {
                    self.dx = x as f64 - self.polygon_cx;
                    self.dy = y as f64 - self.polygon_cy;
                    self.flag = 2;
                }
            }
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.flag == 1 {
                self.image_cx = x as f64 - self.dx;
                self.image_cy = y as f64 - self.dy;
                return Draw::Yes;
            }

            if self.flag == 2 {
                self.polygon_cx = x as f64 - self.dx;
                self.polygon_cy = y as f64 - self.dy;
                return Draw::Yes;
            }
        } else {
            return self.on_mouse_button_up(rb, x, y, flags);
        }
        Draw::No
    }

    fn on_idle(&mut self) -> Draw {
        let mut redraw = false;
        if self.rotate_polygon.borrow().status() {
            let v = self.polygon_angle.borrow().value();
            self.polygon_angle.borrow_mut().set_value(v + 0.5);
            let v = self.polygon_angle.borrow().value();
            if v >= 180.0 {
                self.polygon_angle.borrow_mut().set_value(v - 360.);
            }
            redraw = true;
        }

        if self.rotate_image.borrow().status() {
            let v = self.image_angle.borrow().value();
            self.image_angle.borrow_mut().set_value(v + 0.5);
            let v = self.image_angle.borrow().value();
            if v >= 180.0 {
                self.image_angle.borrow_mut().set_value(v - 360.);
            }
            redraw = true;
        }

        if redraw {
            return Draw::Yes;
        }
        Draw::No
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut img_name = "spheres";
    if args.len() > 1 {
        img_name = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);

    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("spheres") {
            buf = format!(
                "File not found: {}. Download http://www.antigrain.com/{}
				or copy it from another directory if available.",
                img_name, img_name
            );
        } else {
            buf = format!("File not found: {}", img_name);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
    }

    plat.set_caption("Image Affine Transformations with filtering");
	let w = plat.app().util.borrow().rbuf_img(0).width();
	let h = plat.app().util.borrow().rbuf_img(0).height(); 
    if plat.init(w, h, 0) {
        plat.run();
    }
}

use crate::platform::*;
//use agg::basics::{RectD, uround};
use agg::rendering_buffer::RenderBuf;
use agg::{Color, ColorFn, GradientFunc, RasterScanLine};

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::ops::Index;
use std::ops::IndexMut;

mod ctrl;
mod platform;
use crate::ctrl::gamma::Gamma;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::spline::Spline;

use core::f64::consts::PI;
use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const CENTER_X: f64 = 350.0;
const CENTER_Y: f64 = 280.0;

const FLIP_Y: bool = true;

struct GradientPolyMorphWrapper<'a> {
    gradient: &'a dyn GradientFunc,
}

impl<'a> GradientPolyMorphWrapper<'a> {
    fn new(f: &'a dyn GradientFunc) -> Self {
        GradientPolyMorphWrapper { gradient: f }
    }
}

impl<'a> GradientFunc for GradientPolyMorphWrapper<'a> {
    fn calculate(&self, x: i32, y: i32, d: i32) -> i32 {
        self.gradient.calculate(x, y, d)
    }
}

struct ColoFnProfile<C: Color>([C; 256], [u8; 256]);
impl<C: Color> ColoFnProfile<C> {
    pub fn new(c: [C; 256], v: [u8; 256]) -> Self {
        Self(c, v)
    }
}

impl<C: Color> ColorFn<C> for ColoFnProfile<C> {
    fn size(&self) -> u32 {
        self.0.len() as u32
    }
    fn get(&mut self, i: u32) -> C {
        self.0[self.1[i as usize] as usize]
    }
}

impl<C: Color> Index<usize> for ColoFnProfile<C> {
    type Output = C;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[self.1[i] as usize]
    }
}

impl<C: Color> IndexMut<usize> for ColoFnProfile<C> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[self.1[i] as usize]
    }
}

struct Application {
    profile: Ptr<Gamma<'static, agg::Rgba8>>,
    spline_r: Ptr<Spline<'static, agg::Rgba8>>,
    spline_g: Ptr<Spline<'static, agg::Rgba8>>,
    spline_b: Ptr<Spline<'static, agg::Rgba8>>,
    spline_a: Ptr<Spline<'static, agg::Rgba8>>,
    rbox: Ptr<Rbox<'static, agg::Rgba8>>,
    pdx: f64,
    pdy: f64,
    center_x: f64,
    center_y: f64,
    scale: f64,
    prev_scale: f64,
    angle: f64,
    prev_angle: f64,
    scale_x: f64,
    prev_scale_x: f64,
    scale_y: f64,
    prev_scale_y: f64,
    mouse_move: bool,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Drop for Application {
    fn drop(&mut self) {
        let fd = File::create("settings.dat").unwrap();
        let mut writer = std::io::BufWriter::new(fd);
        write!(writer, "{}\n", self.center_x).unwrap();
        write!(writer, "{}\n", self.center_y).unwrap();
        write!(writer, "{}\n", self.scale).unwrap();
        write!(writer, "{}\n", self.angle).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().x(0)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().y(0)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().x(1)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().y(1)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().x(2)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().y(2)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().x(3)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().y(3)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().x(4)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().y(4)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().x(5)).unwrap();
        write!(writer, "{}\n", self.spline_r.borrow().y(5)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().x(0)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().y(0)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().x(1)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().y(1)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().x(2)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().y(2)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().x(3)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().y(3)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().x(4)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().y(4)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().x(5)).unwrap();
        write!(writer, "{}\n", self.spline_g.borrow().y(5)).unwrap();

        write!(writer, "{}\n", self.spline_b.borrow().x(0)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().y(0)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().x(1)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().y(1)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().x(2)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().y(2)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().x(3)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().y(3)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().x(4)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().y(4)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().x(5)).unwrap();
        write!(writer, "{}\n", self.spline_b.borrow().y(5)).unwrap();

        write!(writer, "{}\n", self.spline_a.borrow().x(0)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().y(0)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().x(1)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().y(1)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().x(2)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().y(2)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().x(3)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().y(3)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().x(4)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().y(4)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().x(5)).unwrap();
        write!(writer, "{}\n", self.spline_a.borrow().y(5)).unwrap();

        let (mut x1, mut y1, mut x2, mut y2) = (0., 0., 0., 0.);
        self.profile
            .borrow()
            .values(&mut x1, &mut y1, &mut x2, &mut y2);
        write!(writer, "{}\n", x1).unwrap();
        write!(writer, "{}\n", y1).unwrap();
        write!(writer, "{}\n", x2).unwrap();
        write!(writer, "{}\n", y2).unwrap();
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut profile = Gamma::new(10.0, 10.0, 200.0, 170.0 - 5.0, !flip_y);
        let mut spline_r = Spline::new(210., 10., 210. + 250., 5. + 40., 6, !flip_y);
        let mut spline_g = Spline::new(210., 10. + 40., 210. + 250., 5. + 80., 6, !flip_y);
        let mut spline_b = Spline::new(210., 10. + 80., 210. + 250., 5. + 120., 6, !flip_y);
        let mut spline_a = Spline::new(210., 10. + 120., 210. + 250., 5. + 160., 6, !flip_y);
        let mut rbox = Rbox::new(10.0, 180.0, 200.0, 300.0, !flip_y);

        profile.set_border_width(2.0, 2.0);

        spline_r.set_background_color(&agg::Rgba8::new_params(255, 200, 200, 255));
        spline_g.set_background_color(&agg::Rgba8::new_params(200, 255, 200, 255));
        spline_b.set_background_color(&agg::Rgba8::new_params(200, 200, 255, 255));
        spline_a.set_background_color(&agg::Rgba8::new_params(255, 255, 255, 255));

        spline_r.set_border_width(1.0, 2.0);
        spline_g.set_border_width(1.0, 2.0);
        spline_b.set_border_width(1.0, 2.0);
        spline_a.set_border_width(1.0, 2.0);
        rbox.set_border_width(2.0, 2.0);

        spline_r.set_point(0, 0.0, 1.0);
        spline_r.set_point(1, 1.0 / 5.0, 1.0 - 1.0 / 5.0);
        spline_r.set_point(2, 2.0 / 5.0, 1.0 - 2.0 / 5.0);
        spline_r.set_point(3, 3.0 / 5.0, 1.0 - 3.0 / 5.0);
        spline_r.set_point(4, 4.0 / 5.0, 1.0 - 4.0 / 5.0);
        spline_r.set_point(5, 1.0, 0.0);
        spline_r.update_spline();

        spline_g.set_point(0, 0.0, 1.0);
        spline_g.set_point(1, 1.0 / 5.0, 1.0 - 1.0 / 5.0);
        spline_g.set_point(2, 2.0 / 5.0, 1.0 - 2.0 / 5.0);
        spline_g.set_point(3, 3.0 / 5.0, 1.0 - 3.0 / 5.0);
        spline_g.set_point(4, 4.0 / 5.0, 1.0 - 4.0 / 5.0);
        spline_g.set_point(5, 1.0, 0.0);
        spline_g.update_spline();

        spline_b.set_point(0, 0.0, 1.0);
        spline_b.set_point(1, 1.0 / 5.0, 1.0 - 1.0 / 5.0);
        spline_b.set_point(2, 2.0 / 5.0, 1.0 - 2.0 / 5.0);
        spline_b.set_point(3, 3.0 / 5.0, 1.0 - 3.0 / 5.0);
        spline_b.set_point(4, 4.0 / 5.0, 1.0 - 4.0 / 5.0);
        spline_b.set_point(5, 1.0, 0.0);
        spline_b.update_spline();

        spline_a.set_point(0, 0.0, 1.0);
        spline_a.set_point(1, 1.0 / 5.0, 1.0);
        spline_a.set_point(2, 2.0 / 5.0, 1.0);
        spline_a.set_point(3, 3.0 / 5.0, 1.0);
        spline_a.set_point(4, 4.0 / 5.0, 1.0);
        spline_a.set_point(5, 1.0, 1.0);
        spline_a.update_spline();

        rbox.add_item("Circular");
        rbox.add_item("Diamond");
        rbox.add_item("Linear");
        rbox.add_item("XY");
        rbox.add_item("sqrt(XY)");
        rbox.add_item("Conic");
        rbox.set_cur_item(0);

        let profile = ctrl_ptr(profile);
        let spline_r = ctrl_ptr(spline_r);
        let spline_g = ctrl_ptr(spline_g);
        let spline_b = ctrl_ptr(spline_b);
        let spline_a = ctrl_ptr(spline_a);
        let rbox = ctrl_ptr(rbox);

        let mut app = Application {
            profile: profile.clone(),
            spline_r: spline_r.clone(),
            spline_g: spline_g.clone(),
            spline_b: spline_b.clone(),
            spline_a: spline_a.clone(),
            rbox: rbox.clone(),

            pdx: 0.0,
            pdy: 0.0,
            center_x: 0.,
            center_y: 0.,
            scale: 1.0,
            prev_scale: 1.0,
            angle: 0.0,
            prev_angle: 0.0,
            scale_x: 1.0,
            prev_scale_x: 1.0,
            scale_y: 1.0,
            prev_scale_y: 1.0,
            mouse_move: false,
            ctrls: CtrlContainer {
                ctrl: vec![profile, spline_r, spline_g, spline_b, spline_a, rbox],
                cur_ctrl: -1,
                num_ctrl: 6,
            },
            util: util,
        };

        let fd = File::open("settings.dat").unwrap();
        let reader = BufReader::new(fd);
        let mut lines = reader.lines().map(|line| {
            line.as_ref()
                .unwrap()
                .parse::<f64>()
                .expect(&format!("unable to parse {}", line.unwrap()))
        });

        let mut x: f64;
        let mut y: f64;
        let x2: f64;
        let y2: f64;
        let mut t: f64;

        t = lines.next().unwrap();
        app.center_x = t;
        t = lines.next().unwrap();
        app.center_y = t;
        t = lines.next().unwrap();
        app.scale = t;
        t = lines.next().unwrap();
        app.angle = t;
        x = lines.next().unwrap();
        y = lines.next().unwrap();
        app.spline_r.borrow_mut().set_point(0, x, y);
        x = lines.next().unwrap();
        y = lines.next().unwrap();
        app.spline_r.borrow_mut().set_point(1, x, y);
        app.spline_r
            .borrow_mut()
            .set_point(2, lines.next().unwrap(), lines.next().unwrap());
        app.spline_r
            .borrow_mut()
            .set_point(3, lines.next().unwrap(), lines.next().unwrap());
        app.spline_r
            .borrow_mut()
            .set_point(4, lines.next().unwrap(), lines.next().unwrap());
        app.spline_r
            .borrow_mut()
            .set_point(5, lines.next().unwrap(), lines.next().unwrap());
        x = lines.next().unwrap();
        y = lines.next().unwrap();
        app.spline_g.borrow_mut().set_point(0, x, y);
        app.spline_g
            .borrow_mut()
            .set_point(1, lines.next().unwrap(), lines.next().unwrap());
        app.spline_g
            .borrow_mut()
            .set_point(2, lines.next().unwrap(), lines.next().unwrap());
        app.spline_g
            .borrow_mut()
            .set_point(3, lines.next().unwrap(), lines.next().unwrap());
        app.spline_g
            .borrow_mut()
            .set_point(4, lines.next().unwrap(), lines.next().unwrap());
        app.spline_g
            .borrow_mut()
            .set_point(5, lines.next().unwrap(), lines.next().unwrap());
        x = lines.next().unwrap();
        y = lines.next().unwrap();
        app.spline_b.borrow_mut().set_point(0, x, y);
        app.spline_b
            .borrow_mut()
            .set_point(1, lines.next().unwrap(), lines.next().unwrap());
        app.spline_b
            .borrow_mut()
            .set_point(2, lines.next().unwrap(), lines.next().unwrap());
        app.spline_b
            .borrow_mut()
            .set_point(3, lines.next().unwrap(), lines.next().unwrap());
        app.spline_b
            .borrow_mut()
            .set_point(4, lines.next().unwrap(), lines.next().unwrap());
        app.spline_b
            .borrow_mut()
            .set_point(5, lines.next().unwrap(), lines.next().unwrap());
        x = lines.next().unwrap();
        y = lines.next().unwrap();
        app.spline_a.borrow_mut().set_point(0, x, y);
        app.spline_a
            .borrow_mut()
            .set_point(1, lines.next().unwrap(), lines.next().unwrap());
        app.spline_a
            .borrow_mut()
            .set_point(2, lines.next().unwrap(), lines.next().unwrap());
        app.spline_a
            .borrow_mut()
            .set_point(3, lines.next().unwrap(), lines.next().unwrap());
        app.spline_a
            .borrow_mut()
            .set_point(4, lines.next().unwrap(), lines.next().unwrap());
        app.spline_a
            .borrow_mut()
            .set_point(5, lines.next().unwrap(), lines.next().unwrap());

        app.spline_r.borrow_mut().update_spline();
        app.spline_g.borrow_mut().update_spline();
        app.spline_b.borrow_mut().update_spline();
        app.spline_a.borrow_mut().update_spline();

        x = lines.next().unwrap();
        y = lines.next().unwrap();
        x2 = lines.next().unwrap();
        y2 = lines.next().unwrap();
        app.profile.borrow_mut().set_values(x, y, x2, y2);

        app
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        if key == KeyCode::F1 as u32 {
            let fd = File::create("colors.dat");
            if fd.is_ok() {
                let mut fd = fd.unwrap();
                for i in 0..256 {
                    let c = agg::Rgba::new_params(
                        self.spline_r.borrow().spline()[i],
                        self.spline_g.borrow().spline()[i],
                        self.spline_b.borrow().spline()[i],
                        self.spline_a.borrow().spline()[i],
                    );
                    write!(fd, "    {}, {}, {}, {},\n", c.r, c.g, c.b, c.a).unwrap();
                }
            }

            let fd = File::create("profile.dat");
            if fd.is_ok() {
                let mut fd = fd.unwrap();
                for i in 0..256 {
                    write!(fd, "{}, ", self.profile.borrow().gamma()[i]).unwrap();
                    if (i & 0xF) == 0xF {
                        write!(fd, "\n").unwrap();
                    }
                }
            }
        }
        Draw::No
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let mut pf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pf);
        rb.clear(&ColorType::new_params(0, 0, 0, 255));

        self.profile.borrow_mut().set_text_size(8.0, 0.);

        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.profile.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.spline_r.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.spline_g.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.spline_b.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.spline_a.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.rbox.borrow_mut());

        let ini_scale = 1.0;

        let mut mtx1 = agg::TransAffine::new_default();
        mtx1 *= agg::TransAffine::trans_affine_scaling(ini_scale, ini_scale);
        mtx1 *= agg::TransAffine::trans_affine_rotation(agg::deg2rad(0.0));
        mtx1 *= agg::TransAffine::trans_affine_translation(CENTER_X, CENTER_Y);
        mtx1 *= *self.util.borrow_mut().trans_affine_resizing();

        let mut e1 = agg::Ellipse::new();
        e1.init(0.0, 0.0, 110.0, 110.0, 64, false);

        let mut mtx_g1 = agg::TransAffine::new_default();
        mtx_g1 *= agg::TransAffine::trans_affine_scaling(ini_scale, ini_scale);
        mtx_g1 *= agg::TransAffine::trans_affine_scaling(self.scale, self.scale);
        mtx_g1 *= agg::TransAffine::trans_affine_scaling(self.scale_x, self.scale_y);
        mtx_g1 *= agg::TransAffine::trans_affine_rotation(self.angle);
        mtx_g1 *= agg::TransAffine::trans_affine_translation(self.center_x, self.center_y);
        mtx_g1 *= *self.util.borrow_mut().trans_affine_resizing();
        mtx_g1.invert();

        let mut color_profile: [ColorType; 256] = [ColorType::new(); 256];
        for i in 0..256 {
            color_profile[i] = ColorType::new_from_rgba(&agg::Rgba::new_params(
                self.spline_r.borrow().spline()[i],
                self.spline_g.borrow().spline()[i],
                self.spline_b.borrow().spline()[i],
                self.spline_a.borrow().spline()[i],
            ));
        }

        let mut t1 = agg::ConvTransform::new_owned(e1, mtx1);

        let gr_circle = agg::GradientReflectAdaptor::new(agg::GradientRadial {});
        let gr_diamond = agg::GradientReflectAdaptor::new(agg::GradientDiamond {});
        let gr_x = agg::GradientReflectAdaptor::new(agg::GradientX {});
        let gr_xy = agg::GradientReflectAdaptor::new(agg::GradientXY {});
        let gr_sqrt_xy = agg::GradientReflectAdaptor::new(agg::GradientSqrtXY {});
        let gr_conic = agg::GradientReflectAdaptor::new(agg::GradientConic {});

        let mut gr_ptr: GradientPolyMorphWrapper;

        match self.rbox.borrow().cur_item() {
            1 => gr_ptr = GradientPolyMorphWrapper::new(&gr_diamond),
            2 => gr_ptr = GradientPolyMorphWrapper::new(&gr_x),
            3 => gr_ptr = GradientPolyMorphWrapper::new(&gr_xy),
            4 => gr_ptr = GradientPolyMorphWrapper::new(&gr_sqrt_xy),
            5 => gr_ptr = GradientPolyMorphWrapper::new(&gr_conic),
            _ => gr_ptr = GradientPolyMorphWrapper::new(&gr_circle),
        }

        let mut span_alloc = agg::VecSpan::new();
        let mut colors = ColoFnProfile::new(
            color_profile,
            self.profile.borrow().gamma().try_into().unwrap(),
        );
        let mut inter: agg::SpanIpLinear<_> = agg::SpanIpLinear::new(mtx_g1);
        let mut span_gen = agg::SpanGradient::new(&mut inter, &mut gr_ptr, &mut colors, 0., 150.);

        ras.add_path(&mut t1, 0);
        agg::render_scanlines_aa(&mut ras, &mut sl, &mut rb, &mut span_alloc, &mut span_gen);
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if self.mouse_move {
            let mut x2 = x as f64;
            let mut y2 = y as f64;
            self.util
                .borrow_mut()
                .trans_affine_resizing()
                .inverse_transform(&mut x2, &mut y2);

            if flags & InputFlag::KbdCtrl as u32 != 0 {
                let dx = x2 - self.center_x;
                let dy = y2 - self.center_y;
                self.scale_x = self.prev_scale_x * dx / self.pdx;
                self.scale_y = self.prev_scale_y * dy / self.pdy;
                return Draw::Yes;
            } else {
                if flags & InputFlag::MouseLeft as u32 != 0 {
                    self.center_x = x2 + self.pdx;
                    self.center_y = y2 + self.pdy;
                    return Draw::Yes;
                }

                if flags & InputFlag::MouseRight as u32 != 0 {
                    let dx = x2 - self.center_x;
                    let dy = y2 - self.center_y;
                    self.scale = self.prev_scale
                        * ((dx * dx + dy * dy).sqrt()
                            / (self.pdx * self.pdx + self.pdy * self.pdy).sqrt());

                    self.angle = self.prev_angle + (dy.atan2(dx) - self.pdy.atan2(self.pdx));
                    return Draw::Yes;
                }
            }
        }
        return Draw::No;
    }

    fn on_mouse_button_down(
        &mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, _flags: u32,
    ) -> Draw {
        self.mouse_move = true;
        let mut x2 = x as f64;
        let mut y2 = y as f64;
        self.util
            .borrow_mut()
            .trans_affine_resizing()
            .inverse_transform(&mut x2, &mut y2);

        self.pdx = self.center_x - x2;
        self.pdy = self.center_y - y2;
        self.prev_scale = self.scale;
        self.prev_angle = self.angle + PI;
        self.prev_scale_x = self.scale_x;
        self.prev_scale_y = self.scale_y;
        return Draw::Yes;
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.mouse_move = false;
        Draw::No
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG gradients with Mach bands compensation");

    if plat.init(512, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

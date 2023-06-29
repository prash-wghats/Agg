use crate::platform::*;

use agg::{ImagePattern, PatternFilter, PixFmt, Pixel, RasterScanLine, RenderBuf, VertexSource};
mod ctrl;
mod platform;

use crate::ctrl::bezier::Bezier;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

static BRIGHTNESS_TO_ALPHA: [u8; 256 * 3] = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 254, 254, 254, 254, 254, 254, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254,
    254, 254, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253, 253,
    252, 252, 252, 252, 252, 252, 252, 252, 252, 252, 252, 252, 251, 251, 251, 251, 251, 251, 251,
    251, 251, 250, 250, 250, 250, 250, 250, 250, 250, 249, 249, 249, 249, 249, 249, 249, 248, 248,
    248, 248, 248, 248, 248, 247, 247, 247, 247, 247, 246, 246, 246, 246, 246, 246, 245, 245, 245,
    245, 245, 244, 244, 244, 244, 243, 243, 243, 243, 243, 242, 242, 242, 242, 241, 241, 241, 241,
    240, 240, 240, 239, 239, 239, 239, 238, 238, 238, 238, 237, 237, 237, 236, 236, 236, 235, 235,
    235, 234, 234, 234, 233, 233, 233, 232, 232, 232, 231, 231, 230, 230, 230, 229, 229, 229, 228,
    228, 227, 227, 227, 226, 226, 225, 225, 224, 224, 224, 223, 223, 222, 222, 221, 221, 220, 220,
    219, 219, 219, 218, 218, 217, 217, 216, 216, 215, 214, 214, 213, 213, 212, 212, 211, 211, 210,
    210, 209, 209, 208, 207, 207, 206, 206, 205, 204, 204, 203, 203, 202, 201, 201, 200, 200, 199,
    198, 198, 197, 196, 196, 195, 194, 194, 193, 192, 192, 191, 190, 190, 189, 188, 188, 187, 186,
    186, 185, 184, 183, 183, 182, 181, 180, 180, 179, 178, 177, 177, 176, 175, 174, 174, 173, 172,
    171, 171, 170, 169, 168, 167, 166, 166, 165, 164, 163, 162, 162, 161, 160, 159, 158, 157, 156,
    156, 155, 154, 153, 152, 151, 150, 149, 148, 148, 147, 146, 145, 144, 143, 142, 141, 140, 139,
    138, 137, 136, 135, 134, 133, 132, 131, 130, 129, 128, 128, 127, 125, 124, 123, 122, 121, 120,
    119, 118, 117, 116, 115, 114, 113, 112, 111, 110, 109, 108, 107, 106, 105, 104, 102, 101, 100,
    99, 98, 97, 96, 95, 94, 93, 91, 90, 89, 88, 87, 86, 85, 84, 82, 81, 80, 79, 78, 77, 75, 74, 73,
    72, 71, 70, 69, 67, 66, 65, 64, 63, 61, 60, 59, 58, 57, 56, 54, 53, 52, 51, 50, 48, 47, 46, 45,
    44, 42, 41, 40, 39, 37, 36, 35, 34, 33, 31, 30, 29, 28, 27, 25, 24, 23, 22, 20, 19, 18, 17, 15,
    14, 13, 12, 11, 9, 8, 7, 6, 4, 3, 2, 1,
];

struct PatternSrcBrightnessToAlphaRgba8<'a> {
    //rb: &RenderBuf,
    pf: Pixfmt<'a>,
}

impl<'a> PatternSrcBrightnessToAlphaRgba8<'a> {
    fn new(rb: &mut RenderBuf) -> Self {
        PatternSrcBrightnessToAlphaRgba8 {
            pf: Pixfmt::new_owned(rb.clone()),
        }
    }
}

impl<'a> Pixel for PatternSrcBrightnessToAlphaRgba8<'a> {
    type ColorType = ColorType;
    fn width(&self) -> f64 {
        self.pf.width() as f64
    }

    fn height(&self) -> f64 {
        self.pf.height() as f64
    }

    fn pixel(&self, x: i32, y: i32) -> ColorType {
        let mut c = self.pf.pixel(x, y);
        c.a = BRIGHTNESS_TO_ALPHA[c.r as usize + c.g as usize + c.b as usize];
        c
    }
}

struct Application {
    ctrl_color: agg::Rgba8,
    curve1: Ptr<Bezier<'static, agg::Rgba8>>,
    curve2: Ptr<Bezier<'static, agg::Rgba8>>,
    curve3: Ptr<Bezier<'static, agg::Rgba8>>,
    curve4: Ptr<Bezier<'static, agg::Rgba8>>,
    curve5: Ptr<Bezier<'static, agg::Rgba8>>,
    curve6: Ptr<Bezier<'static, agg::Rgba8>>,
    curve7: Ptr<Bezier<'static, agg::Rgba8>>,
    curve8: Ptr<Bezier<'static, agg::Rgba8>>,
    curve9: Ptr<Bezier<'static, agg::Rgba8>>,
    scale_x: Ptr<Slider<'static, agg::Rgba8>>,
    start_x: Ptr<Slider<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}


impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let curve1 = ctrl_ptr(Bezier::new());
        let curve2 = ctrl_ptr(Bezier::new());
        let curve3 = ctrl_ptr(Bezier::new());
        let curve4 = ctrl_ptr(Bezier::new());
        let curve5 = ctrl_ptr(Bezier::new());
        let curve6 = ctrl_ptr(Bezier::new());
        let curve7 = ctrl_ptr(Bezier::new());
        let curve8 = ctrl_ptr(Bezier::new());
        let curve9 = ctrl_ptr(Bezier::new());
        let scale_x = ctrl_ptr(Slider::new(5.0, 5.0, 240.0, 12.0, !flip_y));
        let start_x = ctrl_ptr(Slider::new(250.0, 5.0, 495.0, 12.0, !flip_y));

        let app = Application {
            curve1: curve1.clone(),
            curve2: curve2.clone(),
            curve3: curve3.clone(),
            curve4: curve4.clone(),
            curve5: curve5.clone(),
            curve6: curve6.clone(),
            curve7: curve7.clone(),
            curve8: curve8.clone(),
            curve9: curve9.clone(),
            scale_x: scale_x.clone(),
            start_x: start_x.clone(),
            ctrl_color: agg::Rgba8::new_params(0, 75, 125, 75),
            ctrls: CtrlContainer {
                ctrl: vec![
                    curve1, curve2, curve3, curve4, curve5, curve6, curve7, curve8, curve9,
                    scale_x, start_x,
                ],
                cur_ctrl: -1,
                num_ctrl: 11,
            },
            util: util,
        };

        app.curve1.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve2.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve3.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve4.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve5.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve6.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve7.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve8.borrow_mut().set_line_color(&app.ctrl_color);
        app.curve9.borrow_mut().set_line_color(&app.ctrl_color);

        app.curve1
            .borrow_mut()
            .set_curve(64., 19., 14., 126., 118., 266., 19., 265.);
        app.curve2
            .borrow_mut()
            .set_curve(112., 113., 178., 32., 200., 132., 125., 438.);
        app.curve3
            .borrow_mut()
            .set_curve(401., 24., 326., 149., 285., 11., 177., 77.);
        app.curve4
            .borrow_mut()
            .set_curve(188., 427., 129., 295., 19., 283., 25., 410.);
        app.curve5
            .borrow_mut()
            .set_curve(451., 346., 302., 218., 265., 441., 459., 400.);
        app.curve6
            .borrow_mut()
            .set_curve(454., 198., 14., 13., 220., 291., 483., 283.);
        app.curve7
            .borrow_mut()
            .set_curve(301., 398., 355., 231., 209., 211., 170., 353.);
        app.curve8
            .borrow_mut()
            .set_curve(484., 101., 222., 33., 486., 435., 487., 138.);
        app.curve9
            .borrow_mut()
            .set_curve(143., 147., 11., 45., 83., 427., 132., 197.);

        app.curve1.borrow_mut().no_transform();
        app.curve2.borrow_mut().no_transform();
        app.curve3.borrow_mut().no_transform();
        app.curve4.borrow_mut().no_transform();
        app.curve5.borrow_mut().no_transform();
        app.curve6.borrow_mut().no_transform();
        app.curve7.borrow_mut().no_transform();
        app.curve8.borrow_mut().no_transform();
        app.curve9.borrow_mut().no_transform();

        app.scale_x.borrow_mut().set_label("Scale X=%0.2f");
        app.scale_x.borrow_mut().set_range(0.2, 3.0);
        app.scale_x.borrow_mut().set_value(1.0);
        app.scale_x.borrow_mut().no_transform();

        app.start_x.borrow_mut().set_label("Start X=%0.2f");
        app.start_x.borrow_mut().set_range(0.0, 10.0);
        app.start_x.borrow_mut().set_value(0.0);
        app.start_x.borrow_mut().no_transform();

        app
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pf = agg::PixBgr24::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pf);
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 243, 255));

        //let ren = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineP8::new();

        let p1 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(0));
        let p2 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(1));
        let p3 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(2));
        let p4 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(3));
        let p5 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(4));
        let p6 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(5));
        let p7 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(6));
        let p8 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(7));
        let p9 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(8));

        let fltr = agg::PatternFilterBilinearRgba8::new();

        let patt = agg::LineImagePattern::new(fltr);
        let mut ren_img = agg::RendererOutlineImage::new(&mut ren_base, patt);
        ren_img.set_scale_x(self.scale_x.borrow_mut().value());
        ren_img.set_start_x(self.start_x.borrow_mut().value());

        let mut ras_img: agg::RasterizerOutlineAa<'_, _> =
            agg::RasterizerOutlineAa::new(&mut ren_img);

        let mut drawcurve = |p: &_, vs: &mut _| {
            (&mut ras_img).ren_mut().pattern_mut().create(p);
            (&mut ras_img)
                .ren_mut()
                .set_scale_x(self.scale_x.borrow().value());
            (&mut ras_img)
                .ren_mut()
                .set_start_x(self.start_x.borrow().value());
            (&mut ras_img).add_path(vs, 0);
        };
		
        drawcurve(&p1, self.curve1.borrow_mut().curve());
        drawcurve(&p2, self.curve2.borrow_mut().curve());
        drawcurve(&p3, self.curve3.borrow_mut().curve());
        drawcurve(&p4, self.curve4.borrow_mut().curve());
        drawcurve(&p5, self.curve5.borrow_mut().curve());
        drawcurve(&p6, self.curve6.borrow_mut().curve());
        drawcurve(&p7, self.curve7.borrow_mut().curve());
        drawcurve(&p8, self.curve8.borrow_mut().curve());
        drawcurve(&p9, self.curve9.borrow_mut().curve());

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve1.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve2.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve3.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve4.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve5.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve6.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve7.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve8.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.curve9.borrow_mut(),
        );

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.scale_x.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.start_x.borrow_mut(),
        );
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        if key == ' ' as u32 {
            if let Ok(mut file) = std::fs::File::create("coord") {
                writeln!(
                    file,
                    "{}, {}, {}, {}, {}, {}, {}, {}",
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
        }
        Draw::No
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);

    let buf;
    let ext = plat.app().util.borrow().img_ext().to_string();
    let r1 = plat.app_mut().util.borrow_mut().load_img(0, "1");
    let r2 = plat.app_mut().util.borrow_mut().load_img(1, "2");
    let r3 = plat.app_mut().util.borrow_mut().load_img(2, "3");
    let r4 = plat.app_mut().util.borrow_mut().load_img(3, "4");
    let r5 = plat.app_mut().util.borrow_mut().load_img(4, "5");
    let r6 = plat.app_mut().util.borrow_mut().load_img(5, "6");
    let r7 = plat.app_mut().util.borrow_mut().load_img(6, "7");
    let r8 = plat.app_mut().util.borrow_mut().load_img(7, "8");
    let r9 = plat.app_mut().util.borrow_mut().load_img(8, "9");
    if !r1 || !r2 || !r3 || !r4 || !r5 || !r6 || !r7 || !r8 || !r9 {
        buf = format!(
            "There must be files 1{}...9{}\n:. Download http://www.antigrain.com/
				or copy it from another directory if available.",
            ext, ext
        );

        plat.app_mut().util.borrow_mut().message(&buf);
        return;
    }

    plat.set_caption("Image Affine Transformations with filtering");

    if plat.init(500, 450, WindowFlag::Resize as u32) {
        plat.run();
    }
}

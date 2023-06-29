use crate::platform::*;

use agg::{
    ImagePattern, PatternFilter, PixFmt, Pixel, RasterScanLine, RenderBuf, Renderer,
    RendererOutline, RendererScanlineColor,
};

mod ctrl;
mod platform;

use crate::ctrl::polygon::Polygon;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
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
        let c = self.pf.pixel(x, y);
        let a = BRIGHTNESS_TO_ALPHA[c.r as usize + c.g as usize + c.b as usize];
        ColorType {
            r: c.r,
            g: c.g,
            b: c.b,
            a,
        }
    }
}

struct Application {
    ctrl_color: agg::Rgba8,
    line1: Ptr<Polygon<'static, agg::Rgba8>>,
    scale_x: Ptr<Slider<'static, agg::Rgba8>>,
    start_x: Ptr<Slider<'static, agg::Rgba8>>,
    scale: agg::TransAffine,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn draw_polyline<Ras: RasterScanLine>(&self, ras: &mut Ras, polyline: &[f64], num_points: u32) {
        let vs = agg::PolyPlainAdaptor::new_init(
            polyline,
            num_points as usize,
            self.line1.borrow().close(),
        );
        let mut trans = agg::ConvTransform::new_owned(vs, self.scale);
        ras.add_path(&mut trans, 0);
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let ctrl_color = agg::Rgba8::new_params(0, 75, 125, 75);
        let scale = agg::TransAffine::new_default();
        let line1 = ctrl_ptr(Polygon::new(5, 5.));
        let scale_x = ctrl_ptr(Slider::new(5.0, 5.0, 240.0, 12.0, !flip_y));
        let start_x = ctrl_ptr(Slider::new(250.0, 5.0, 495.0, 12.0, !flip_y));
        line1.borrow_mut().set_line_color(ctrl_color);
        *line1.borrow_mut().xn_mut(0) = 20.;
        *line1.borrow_mut().yn_mut(0) = 20.;
        *line1.borrow_mut().xn_mut(1) = 500. - 20.;
        *line1.borrow_mut().yn_mut(1) = 500. - 20.;
        *line1.borrow_mut().xn_mut(2) = 500. - 60.;
        *line1.borrow_mut().yn_mut(2) = 20.;
        *line1.borrow_mut().xn_mut(3) = 40.;
        *line1.borrow_mut().yn_mut(3) = 500. - 40.;
        *line1.borrow_mut().xn_mut(4) = 100.;
        *line1.borrow_mut().yn_mut(4) = 300.;
        line1.borrow_mut().set_close(false);

        scale_x.borrow_mut().set_label("Scale X=%0.2f");
        scale_x.borrow_mut().set_range(0.2, 3.0);
        scale_x.borrow_mut().set_value(1.0);
        scale_x.borrow_mut().no_transform();

        start_x.borrow_mut().set_label("Start X=%0.2f");
        start_x.borrow_mut().set_range(0.0, 10.0);
        start_x.borrow_mut().set_value(0.0);
        start_x.borrow_mut().no_transform();

        Application {
            ctrl_color,
            line1: line1.clone(),
            scale_x: scale_x.clone(),
            start_x: start_x.clone(),
            scale,
            ctrls: CtrlContainer {
                ctrl: vec![line1, scale_x, start_x],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        }
        //app.line1.borrow_mut().set_transform(&app.scale); Doesnt work!
    }

    fn on_init(&mut self) {
        self.line1.borrow_mut().set_transform(&self.scale);
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let pixf0 = Pixfmt::new_owned(rbuf.clone());
        let mut ren_base0 = agg::RendererBase::new_owned(pixf0);

        let pixf1 = Pixfmt::new_owned(rbuf.clone());
        let mut ren_base1 = agg::RendererBase::new_owned(pixf1);

        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pixf);
        ren_base.clear(&agg::Rgba8::new_params(125, 190, 220, 255));

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineU8::new();
        let width = self.util.borrow().width();
        let height = self.util.borrow().height();
        ras.clip_box(0., 0., width, height);
        // Pattern source. Must have an interface:
        // width() const
        // height() const
        // pixel(int x, int y) const
        // Any agg::renderer_base<> or derived
        // is good for the use as a source.
        let p1 = PatternSrcBrightnessToAlphaRgba8::new(self.util.borrow_mut().rbuf_img_mut(0));
        let fltr = agg::PatternFilterBilinearRgba8::new();
        // agg::line_image_pattern is the main container for the patterns. It creates
        // a copy of the patterns extended according to the needs of the filter.
        // agg::line_image_pattern can operate with arbitrary image width, but if the
        // width of the pattern is power of 2, it's better to use the modified
        // version agg::line_image_pattern_pow2 because it works about 15-25 percent
        // faster than agg::line_image_pattern (because of using simple masking instead
        // of expensive '%' operation).

        //-- Create with specifying the source
        //pattern_type patt(fltr, src);

        //-- Create uninitialized and set the source
        let mut patt = agg::LineImagePattern::new(fltr);
        patt.create(&p1);
        let mut ren_img = agg::RendererOutlineImage::new(&mut ren_base0, patt);
        // Set the clip box a bit bigger than you expect. You need it
        // to draw the clipped line caps correctly. The correct result
        // is achieved with raster clipping.
        ren_img.set_scale_x(self.scale_x.borrow().value());
        ren_img.set_start_x(self.start_x.borrow().value());
        // Calculate the dilation value so that, the line caps were
        // drawn correctly.
        let w2 = 9.0;
        ren_img.clip_box(
            50. - w2,
            50. - w2,
            width as f64 - 50. + w2,
            height as f64 - 50. + w2,
        );

        let mut ras_img: agg::RasterizerOutlineAa<'_, _> =
            agg::RasterizerOutlineAa::new(&mut ren_img);

        //-- create uninitialized and set parameters
        let mut profile = agg::LineProfileAA::new();
        profile.set_smoother_width(10.0);
        profile.set_width(8.0);

        let mut ren_line = agg::RendererOutlineAa::new(&mut ren_base1, profile);
        ren_line.set_color(ColorType::new_params(0, 0, 127, 255));

        ren_line.clip_box(
            50. - w2,
            50. - w2,
            width as f64 - 50. + w2,
            height as f64 - 50. + w2,
        );

        let mut ras_line: agg::RasterizerOutlineAa<'_, _> =
            agg::RasterizerOutlineAa::new(&mut ren_line);
        ras_line.set_round_cap(true);
        //ras_line.line_join(agg::outline_no_join);     //optional

        // First, draw polyline without raster clipping just to show the idea
        self.draw_polyline(
            &mut ras_line,
            self.line1.borrow().polygon(),
            self.line1.borrow().num_points(),
        );
        self.draw_polyline(
            &mut ras_img,
            self.line1.borrow().polygon(),
            self.line1.borrow().num_points(),
        );

        // Clear the area, almost opaque, but not completely
        ren_base.blend_bar(
            0,
            0,
            width as i32,
            height as i32,
            &ColorType::new_params(255, 255, 255, 255),
            200,
        );

        // Set the raster clip box and then, draw again.
        // In reality there shouldn't be two calls above.
        // It's done only for demonstration
        ren_base.set_clip_box(50, 50, width as i32 - 50, height as i32 - 50);
        ras_img
            .ren_mut()
            .ren_mut()
            .set_clip_box(50, 50, width as i32 - 50, height as i32 - 50);
        ras_line
            .ren_mut()
            .ren_mut()
            .set_clip_box(50, 50, width as i32 - 50, height as i32 - 50);
        // This "copy_bar" is also for demonstration only
        ren_base.copy_bar(
            0,
            0,
            width as i32,
            height as i32,
            &ColorType::new_params(255, 255, 255, 255),
        );

        // Finally draw polyline correctly clipped: We use double clipping,
        // first is vector clipping, with extended clip box, second is raster
        // clipping with normal clip box.
        ras_img.ren_mut().set_scale_x(self.scale_x.borrow().value());
        ras_img.ren_mut().set_start_x(self.start_x.borrow().value());
        self.draw_polyline(
            &mut ras_line,
            self.line1.borrow().polygon(),
            self.line1.borrow().num_points(),
        );
        self.draw_polyline(
            &mut ras_img,
            self.line1.borrow().polygon(),
            self.line1.borrow().num_points(),
        );

        // Reset clipping and draw the controls and stuff
        ren_base.reset_clipping(true);

        self.line1
            .borrow_mut()
            .set_line_width(1.0 / self.scale.scale());
        self.line1
            .borrow_mut()
            .set_point_radius(5.0 / self.scale.scale());

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.line1.borrow_mut(),
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

        let buf = format!(
            "Len={:.2}",
            agg::calc_distance(
                self.line1.borrow().polygon()[0],
                self.line1.borrow().polygon()[1],
                self.line1.borrow().polygon()[2],
                self.line1.borrow().polygon()[3]
            ) * self.scale.scale()
        );

        let mut t = agg::GsvText::new();
        t.set_size(10.0, 0.);
        t.set_start_point(10.0, 30.0);
        t.set_text(&buf);

        let mut pt: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(t);
        pt.set_width(1.5);
        pt.set_line_cap(agg::LineCap::Round);

        ras.add_path(&mut pt, 0);

        let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);
        ren.set_color(ColorType::new_params(0, 0, 0, 255));
        agg::render_scanlines(&mut ras, &mut sl, &mut ren);
    }

    fn on_key(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, key: u32, _flags: u32) -> Draw {
        if key == '+' as u32 || key == KeyCode::KpPlus as u32 {
            self.scale *= agg::TransAffine::trans_affine_translation(-x as f64, -y as f64);
            self.scale *= agg::TransAffine::trans_affine_scaling_eq(1.1);
            self.scale *= agg::TransAffine::trans_affine_translation(x as f64, y as f64);
            return Draw::Yes;
        } else if key == '-' as u32 || key == KeyCode::KpMinus as u32 {
            self.scale *= agg::TransAffine::trans_affine_translation(-x as f64, -y as f64);
            self.scale *= agg::TransAffine::trans_affine_scaling_eq(1.0 / 1.1);
            self.scale *= agg::TransAffine::trans_affine_translation(x as f64, y as f64);
            return Draw::Yes;
        }
        Draw::No
    }
}

fn main() {
    let img_name = "1";

    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);

    let buf;
    if !plat.app_mut().util.borrow_mut().load_img(0, img_name) {
        if img_name.eq("1") {
            buf = format!(
                "File not found: {}. Download http://www.antigrain.com/{}
				or copy it from another directory if available.",
                img_name, img_name
            );
        } else {
            buf = format!("File not found: {}", img_name);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
        return;
    }

    plat.set_caption("AGG Example. Clipping Lines with Image Patterns");

    if plat.init(500, 500, WindowFlag::Resize as u32) {
        plat.run();
    }
}

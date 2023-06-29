use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;

use agg::basics::is_stop;
use agg::font_cache_manager::*;
use agg::font_freetype::FreetypeBase;
use agg::{FontEngine, RasterScanLine, RendererScanlineColor, VertexSource};
mod ctrl;
mod platform;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}
mod misc;
use misc::pixel_formats::*;

type FontEngineType = FreetypeBase<'static, i32>;

const TEXT: &str = "Anti-Grain Geometry is designed as a set of loosely coupled  \
algorithms and class templates united with a common idea,  \
so that all the components can be easily combined. Also,  \
the template based design allows you to replace any part of  \
the library without the necessity to modify a single byte in  \
the existing code.  \
AGG is designed keeping in mind extensibility and flexibility.  \
Basically I just wanted to create a toolkit that would allow me  \
(and anyone else) to add new fancy algorithms very easily.  \
AGG does not dictate you any style of its use, you are free to  \
use any part of it. However, AGG is often associated with a tool  \
for rendering images in memory. That is not quite true, but it can  \
be a good starting point in studying. The tutorials describe the  \
use of AGG starting from the low level functionality that deals with  \
frame buffers and pixels. Then you will gradually understand how to  \
abstract different parts of the library and how to use them separately.  \
Remember, the raster picture is often not the only thing you want to  \
obtain, you will probably want to print your graphics with highest  \
possible quality and in this case you can easily combine the \"vectorial\"  \
part of the library with some API like Windows GDI, having a common  \
external interface. If that API can render multi-polygons with non-zero  \
and even-odd filling rules it's all you need to incorporate AGG into  \
your application. For example, Windows API PolyPolygon perfectly fits  \
these needs, except certain advanced things like gradient filling,  \
Gouraud shading, image transformations, and so on. Or, as an alternative,  \
you can use all AGG algorithms producing high resolution pixel images and  \
then to send the result to the printer as a pixel map. \
Below is a typical brief scheme of the AGG rendering pipeline.  \
Please note that any component between the Vertex Source  \
and Screen Output is not mandatory. It all depends on your  \
particular needs. For example, you can use your own rasterizer,  \
based on Windows API. In this case you won't need the AGG rasterizer  \
and renderers. Or, if you need to draw only lines, you can use the  \
AGG outline rasterizer that has certain restrictions but works faster.  \
The number of possibilities is endless.  \
Vertex Source is some object that produces polygons or polylines as  \
a set of consecutive 2D vertices with commands like MoveTo, LineTo.  \
It can be a container or some other object that generates vertices  \
on demand.  \
Coordinate conversion pipeline consists of a number of coordinate  \
converters. It always works with vectorial data (X,Y) represented  \
as floating point numbers (double). For example, it can contain an  \
affine transformer, outline (stroke) generator, some marker  \
generator (like arrowheads/arrowtails), dashed lines generator,  \
and so on. The pipeline can have branches and you also can have  \
any number of different pipelines. You also can write your own  \
converter and include it into the pipeline.  \
Scanline Rasterizer converts vectorial data into a number of  \
horizontal scanlines. The scanlines usually (but not obligatory)  \
carry information about Anti-Aliasing as coverage values.  \
Renderers render scanlines, sorry for the tautology. The simplest  \
example is solid filling. The renderer just adds a color to the  \
scanline and writes the result into the rendering buffer.  \
More complex renderers can produce multi-color result,  \
like gradients, Gouraud shading, image transformations,  \
patterns, and so on. Rendering Buffer is a buffer in memory  \
that will be displayed afterwards. Usually but not obligatory  \
it contains pixels in format that fits your video system.  \
For example, 24 bits B-G-R, 32 bits B-G-R-A, or 15  \
bits R-G-B-555 for Windows. But in general, there're no  \
restrictions on pixel formats or color space if you write  \
your own low level class that supports that format.  \
Colors in AGG appear only in renderers, that is, when you  \
actually put some data to the rendering buffer. In general,  \
there's no general purpose structure or class like color,  \
instead, AGG always operates with concrete color space.  \
There are plenty of color spaces in the world, like RGB,  \
HSV, CMYK, etc., and all of them have certain restrictions.  \
For example, the RGB color space is just a poor subset of  \
colors that a human eye can recognize. If you look at the full  \
CIE Chromaticity Diagram, you will see that the RGB triangle  \
is just a little part of it.  \
In other words there are plenty of colors in the real world  \
that cannot be reproduced with RGB, CMYK, HSV, etc. Any color  \
space except the one existing in Nature is restrictive. Thus,  \
it was decided not to introduce such an object like color in  \
order not to restrict the possibilities in advance. Instead,  \
there are objects that operate with concrete color spaces.  \
Currently there are agg::rgba and agg::rgba8 that operate  \
with the most popular RGB color space (strictly speaking there's  \
RGB plus Alpha). The RGB color space is used with different  \
pixel formats, like 24-bit RGB or 32-bit RGBA with different  \
order of color components. But the common property of all of  \
them is that they are essentially RGB. Although, AGG doesn't  \
explicitly support any other color spaces, there is at least  \
a potential possibility of adding them. It means that all  \
class and function templates that depend on the color type  \
are parameterized with the ColorT argument.  \
Basically, AGG operates with coordinates of the output device.  \
On your screen there are pixels. But unlike many other libraries  \
and APIs AGG initially supports Subpixel Accuracy. It means  \
that the coordinates are represented as doubles, where fractional  \
values actually take effect. AGG doesn't have an embedded  \
conversion mechanism from world to screen coordinates in order  \
not to restrict your freedom. It's very important where and when  \
you do that conversion, so, different applications can require  \
different approaches. AGG just provides you a transformer of  \
that kind, namely, that can convert your own view port to the  \
device one. And it's your responsibility to include it into  \
the proper place of the pipeline. You can also write your  \
own very simple class that will allow you to operate with  \
millimeters, inches, or any other physical units.  \
Internally, the rasterizers use integer coordinates of the  \
format 24.8 bits, that is, 24 bits for the integer part and 8  \
bits for the fractional one. In other words, all the internal  \
coordinates are multiplied by 256. If you intend to use AGG in  \
some embedded system that has inefficient floating point  \
processing, you still can use the rasterizers with their  \
integer interfaces. Although, you won't be able to use the  \
floating point coordinate pipelines in this case. ";

struct Application {
    ren_type: Ptr<Rbox<'static, agg::Rgba8>>,
    height: Ptr<Slider<'static, agg::Rgba8>>,
    width: Ptr<Slider<'static, agg::Rgba8>>,
    weight: Ptr<Slider<'static, agg::Rgba8>>,
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    hinting: Ptr<Cbox<'static, agg::Rgba8>>,
    kerning: Ptr<Cbox<'static, agg::Rgba8>>,
    performance: Ptr<Cbox<'static, agg::Rgba8>>,
    //feng: font_engine_type,
    fman: FontCacheManager<FontEngineType>,
    old_height: f64,
    font_flip_y: bool,
    //curves: agg::ConvCurve<agg::SerializedIntegerPathAdaptor<i16>>,
    contour:
        agg::ConvContour<'static, agg::ConvCurve<'static, agg::SerializedIntegerPathAdaptor<i32>>>,
    pub ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

type RB<'a> = agg::RendererBase<'a, Pixfmt<'a>>;
type RSlAaS<'a, 'b> = agg::RendererScanlineAASolid<'a, RB<'b>>;
type RSlBS<'a, 'b> = agg::RendererScanlineBinSolid<'a, RB<'b>>;

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let mut ren_type = Rbox::new(5.0, 5.0, 5.0 + 150.0, 110.0, !flip_y);
        let mut height = Slider::new(160., 10.0, 640. - 5.0, 18.0, !flip_y);
        let mut width = Slider::new(160., 30.0, 640. - 5.0, 38.0, !flip_y);
        let mut weight = Slider::new(160., 50.0, 640. - 5.0, 58.0, !flip_y);
        let mut gamma = Slider::new(260., 70.0, 640. - 5.0, 78.0, !flip_y);
        let mut hinting = Cbox::new(160., 65.0, "Hinting", !flip_y);
        let mut kerning = Cbox::new(160., 80.0, "Kerning", !flip_y);
        let mut performance = Cbox::new(160., 95.0, "Test Performance", !flip_y);
        ren_type.add_item("Native Mono");
        ren_type.add_item("Native Gray 8");
        ren_type.add_item("Outline");
        ren_type.add_item("AGG Mono");
        ren_type.add_item("AGG Gray 8");
        ren_type.set_cur_item(2);
        ren_type.no_transform();

        height.set_label("Font Height=%0.2f");
        height.set_range(8., 32.);
        height.set_num_steps(32 - 8);
        height.set_value(18.);
        height.set_text_thickness(1.5);
        height.no_transform();

        width.set_label("Font Width=%0.2f");
        width.set_range(8., 32.);
        width.set_num_steps(32 - 8);
        width.set_text_thickness(1.5);
        width.set_value(18.);
        width.no_transform();

        weight.set_label("Font Weight=%0.2f");
        weight.set_range(-1., 1.);
        weight.set_text_thickness(1.5);
        weight.no_transform();

        gamma.set_label("Gamma=%0.2f");
        gamma.set_range(0.1, 2.0);
        gamma.set_value(1.0);
        gamma.set_text_thickness(1.5);
        gamma.no_transform();

        hinting.set_status(true);
        hinting.no_transform();

        kerning.set_status(true);
        kerning.no_transform();

        kerning.set_status(false);
        performance.no_transform();

        let ren_type = ctrl_ptr(ren_type);
        let height = ctrl_ptr(height);
        let width = ctrl_ptr(width);
        let weight = ctrl_ptr(weight);
        let gamma = ctrl_ptr(gamma);
        let hinting = ctrl_ptr(hinting);
        let kerning = ctrl_ptr(kerning);
        let performance = ctrl_ptr(performance);
        let eng = FontEngineType::new(32);
        let fman = FontCacheManager::new(eng, 32);
        let mut curves = agg::ConvCurve::new_owned(fman.path_adaptor());
        curves.set_approximation_scale(2.0);
        let mut contour = agg::ConvContour::new_owned(curves);
        contour.set_auto_detect_orientation(false);

        Application {
            ren_type: ren_type.clone(),
            height: height.clone(),
            width: width.clone(),
            weight: weight.clone(),
            gamma: gamma.clone(),
            hinting: hinting.clone(),
            kerning: kerning.clone(),
            performance: performance.clone(),
            font_flip_y: !flip_y,
            fman: fman,
            old_height: 0.0,
            contour: contour,
            ctrls: CtrlContainer {
                ctrl: vec![
                    ren_type,
                    height,
                    width,
                    weight,
                    gamma,
                    hinting,
                    kerning,
                    performance,
                ],
                cur_ctrl: -1,
                num_ctrl: 8,
            },
            util: util,
        }
    }

    fn on_ctrl_change(&mut self, rb: &mut agg::RenderBuf) {
        if self.performance.borrow().status() {
            let mut pf = Pixfmt::new_owned(rb.clone());
            let mut ren_base = agg::RendererBase::new_borrowed(&mut pf);
            let mut pf1 = Pixfmt::new_borrowed(rb);
            let mut ren_base1 = agg::RendererBase::new_borrowed(&mut pf1);
            ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
            let mut ren_solid = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);
            let mut ren_bin = agg::RendererScanlineBinSolid::new_borrowed(&mut ren_base1);

            let mut sl = agg::ScanlineU8::new();
            let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

            let mut num_glyphs = 0;
            self.util.borrow_mut().start_timer();
            for _ in 0..50 {
                num_glyphs += self.draw_text(&mut ras, &mut sl, &mut ren_solid, &mut ren_bin);
            }
            let t = self.util.borrow_mut().elapsed_time();
            let buf = format!(
                "Glyphs={}, Time={}ms, {:.3} glyps/sec, {:.3} microsecond/glyph",
                num_glyphs,
                t,
                (num_glyphs as f64 / t) * 1000.0,
                (t / num_glyphs as f64) * 1000.0
            );
            self.util.borrow_mut().message(&buf);

            self.performance.borrow_mut().set_status(false);
        }
    }

    fn on_key(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, key: u32, _flags: u32,
    ) -> Draw {
        if key == ' ' as u32 {
            self.font_flip_y = !self.font_flip_y;
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut tmprb = *rbuf;
        let mut pf = Pixfmt::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pf);
        let mut pf1 = Pixfmt::new_borrowed(&mut tmprb);
        let mut ren_base1 = agg::RendererBase::new_borrowed(&mut pf1);
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut ren_solid = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);
        let mut ren_bin = agg::RendererScanlineBinSolid::new_borrowed(&mut ren_base1);

        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        if self.height.borrow().value() != self.old_height {
            self.old_height = self.height.borrow().value();
            self.width.borrow_mut().set_value(self.old_height);
        }

        if self.ren_type.borrow().cur_item() == 3 {
            // When rendering in mono format,
            // Set threshold gamma = 0.5
            //-------------------
            self.fman
                .engine_mut()
                .set_gamma(agg::GammaThreshold::new_with_threshold(
                    self.gamma.borrow().value() / 2.0,
                ));
        } else {
            self.fman
                .engine_mut()
                .set_gamma(agg::GammaPower::new_with_gamma(self.gamma.borrow().value()));
        }

        if self.ren_type.borrow().cur_item() == 2 {
            // For outline cache set gamma for the rasterizer
            //-------------------
            ras.set_gamma(&agg::GammaPower::new_with_gamma(self.gamma.borrow().value()));
        }

        //ren_base.copy_hline(0, int(height() - self.height.value()) - 10, 100, agg::rgba(0,0,0));
        self.draw_text(&mut ras, &mut sl, &mut ren_solid, &mut ren_bin);

        ras.set_gamma(&agg::GammaPower::new_with_gamma(1.0));

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.ren_type.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.height.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.width.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.weight.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.gamma.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.hinting.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.kerning.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.performance.borrow_mut(),
        );
    }
}

impl Application {
    fn dump_path<VS: VertexSource>(path: &mut VS) {
        let mut fd = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("dump_path")
            .unwrap();

        path.rewind(0);
        let mut cmd;
        let mut x = 0.;
        let mut y = 0.;

        loop {
            cmd = path.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            write!(fd, "{:02X} {:8.2} {:8.2}\n", cmd, x, y).unwrap();
        }
    }

    fn draw_text(
        &mut self, ras: &mut agg::RasterizerScanlineAa, sl: &mut agg::ScanlineU8,
        ren_solid: &mut RSlAaS, ren_bin: &mut RSlBS,
    ) -> u32 {
        let mut gren = agg::GlyphRender::NativeMono;
        match self.ren_type.borrow().cur_item() {
            0 => gren = agg::GlyphRender::NativeMono,
            1 => gren = agg::GlyphRender::NativeGray8,
            2 => gren = agg::GlyphRender::Outline,
            3 => gren = agg::GlyphRender::AggMono,
            4 => gren = agg::GlyphRender::AggGray8,
            _ => (),
        }

        let mut num_glyphs = 0;

        self.contour
            .set_width(-self.weight.borrow().value() * self.height.borrow().value() * 0.05);

        if self
            .fman
            .engine_mut()
            .load_font("timesi.ttf\u{0}", 0, gren, &[], 0)
        {
            self.fman
                .engine_mut()
                .set_hinting(self.hinting.borrow().status());
            self.fman
                .engine_mut()
                .set_height(self.height.borrow().value());
            self.fman
                .engine_mut()
                .set_width(self.width.borrow().value());
            self.fman.engine_mut().set_flip_y(self.font_flip_y);

            let mut mtx = agg::TransAffine::new_default();
            mtx *= agg::TransAffine::trans_affine_rotation(agg::basics::deg2rad(-4.0));
            //mtx *= agg::TransAffine::trans_affine_skewing(-0.4, 0);
            //mtx *= agg::TransAffine::trans_affine_translation(1, 0);
            self.fman.engine_mut().transform(&mtx);

            let mut x = 10.0;
            let mut y0 = self.util.borrow().height() - self.height.borrow().value() - 10.0;
            let mut y = y0;
            let p = TEXT.as_bytes();
            let mut i = 0;
            let mut bsl = agg::ScanlineU8::new();
            while TEXT.len() > i && p[i] != 0 {
                let glyph = self.fman.glyph(p[i] as u32);
                if !glyph.is_null() {
                    if self.kerning.borrow().status() {
                        self.fman.add_kerning(&mut x, &mut y);
                    }

                    if x >= self.util.borrow().width() - self.height.borrow().value() {
                        x = 10.0;
                        y0 -= self.height.borrow().value();
                        if y0 <= 120. {
                            break;
                        }

                        y = y0;
                    }

                    //self.fman.init_embedded_adaptors(glyph, x, y);
                    let gl = unsafe { &*glyph };
                    match unsafe { (*glyph).data_type } {
                        agg::GlyphDataType::Mono => {
                            let mut adp = self.fman.mono_adaptor();
                            adp.init(gl.data.as_ptr(), gl.data_size as usize, x, y);
                            ren_bin.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
                            agg::render_scanlines(
                                &mut adp,
                                &mut self.fman.mono_scanline(),
                                ren_bin,
                            );
                        }
                        agg::GlyphDataType::Gray8 => {
                            let mut adp = self.fman.gray8_adaptor();
                            adp.init(gl.data.as_ptr(), gl.data_size as usize, x, y);
                            ren_solid.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
                            agg::render_scanlines(&mut adp, &mut bsl, ren_solid);
                        }
                        agg::GlyphDataType::Outline => {
                            self.contour.source_mut().source_mut().init(
                                gl.data.as_ptr(),
                                gl.data_size as usize,
                                x,
                                y,
                                1.0,
                            );
                            ras.reset();
                            if (self.weight.borrow().value()).abs() <= 0.01 {
                                // For the sake of efficiency skip the
                                // contour converter if the weight is about zero.
                                //-----------------------
                                ras.add_path(self.contour.source_mut(), 0);
                            } else {
                                ras.add_path(&mut self.contour, 0);
                            }
                            ren_solid.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
                            agg::render_scanlines(ras, sl, ren_solid);
                            //Self::dump_path(self.contour.source_mut().source_mut());
                        }
                        _ => (),
                    }

                    // increment pen position
                    unsafe {
                        // increment pen position
                        x += (*glyph).advance_x;
                        y += (*glyph).advance_y;
                    }
                    num_glyphs += 1;
                }

                i += 1;
            }
        }

        num_glyphs
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption(r#"AGG Example. Rendering Fonts with FreeType"#);

    if plat.init(640, 520, WindowFlag::Resize as u32) {
        plat.run();
    }
}

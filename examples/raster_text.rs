use crate::platform::*;
use agg::embedded_raster_fonts::*;
use agg::glyph_raster_bin::*;
use agg::rendering_buffer::RenderBuf;
use agg::GradientFunc;
mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = true;
struct GradientSineRepeatAdaptor<GradientF: GradientFunc> {
    periods: f64,
    gradient: GradientF,
}

impl<GradientF: GradientFunc> GradientSineRepeatAdaptor<GradientF> {
    fn new(g: GradientF) -> Self {
        Self {
            periods: std::f64::consts::PI * 2.0,
            gradient: g,
        }
    }

    fn periods(&mut self, p: f64) {
        self.periods = p * std::f64::consts::PI * 2.0;
    }
}

impl<GradientF: GradientFunc> GradientFunc for GradientSineRepeatAdaptor<GradientF> {
    fn calculate(&self, x: i32, y: i32, d: i32) -> i32 {
        ((1.0 + (self.gradient.calculate(x, y, d) as f64 * self.periods / d as f64).sin())
            * (d as f64 / 2.0)) as i32
    }
}

struct Application {
    flip_y: bool,
    ctrls: CtrlContainer,
    _util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        Self {
            flip_y,
            ctrls: CtrlContainer {
                ctrl: vec![],
                cur_ctrl: -1,
                num_ctrl: 0,
            },
            _util: util,
        }
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        struct FontType {
            font: &'static [u8],
            name: &'static str,
        }

        let fonts = [
            FontType {
                font: &GSE4X6,
                name: "gse4x6",
            },
            FontType {
                font: &GSE4X8,
                name: "gse4x8",
            },
            FontType {
                font: &GSE5X7,
                name: "gse5x7",
            },
            FontType {
                font: &GSE5X9,
                name: "gse5x9",
            },
            FontType {
                font: &GSE6X9,
                name: "gse6x9",
            },
            FontType {
                font: &GSE6X12,
                name: "gse6x12",
            },
            FontType {
                font: &GSE7X11,
                name: "gse7x11",
            },
            FontType {
                font: &GSE7X11_BOLD,
                name: "gse7x11_bold",
            },
            FontType {
                font: &GSE7X15,
                name: "gse7x15",
            },
            FontType {
                font: &GSE7X15_BOLD,
                name: "gse7x15_bold",
            },
            FontType {
                font: &GSE8X16,
                name: "gse8x16",
            },
            FontType {
                font: &GSE8X16_BOLD,
                name: "gse8x16_bold",
            },
            FontType {
                font: &MCS11_PROP,
                name: "mcs11_prop",
            },
            FontType {
                font: &MCS11_PROP_CONDENSED,
                name: "mcs11_prop_condensed",
            },
            FontType {
                font: &MCS12_PROP,
                name: "mcs12_prop",
            },
            FontType {
                font: &MCS13_PROP,
                name: "mcs13_prop",
            },
            FontType {
                font: &MCS5X10_MONO,
                name: "mcs5x10_mono",
            },
            FontType {
                font: &MCS5X11_MONO,
                name: "mcs5x11_mono",
            },
            FontType {
                font: &MCS6X10_MONO,
                name: "mcs6x10_mono",
            },
            FontType {
                font: &MCS6X11_MONO,
                name: "mcs6x11_mono",
            },
            FontType {
                font: &MCS7X12_MONO_HIGH,
                name: "mcs7x12_mono_high",
            },
            FontType {
                font: &MCS7X12_MONO_LOW,
                name: "mcs7x12_mono_low",
            },
            FontType {
                font: &VERDANA12,
                name: "verdana12",
            },
            FontType {
                font: &VERDANA12_BOLD,
                name: "verdana12_bold",
            },
            FontType {
                font: &VERDANA13,
                name: "verdana13",
            },
            FontType {
                font: &VERDANA13_BOLD,
                name: "verdana13_bold",
            },
            FontType {
                font: &VERDANA14,
                name: "verdana14",
            },
            FontType {
                font: &VERDANA14_BOLD,
                name: "verdana14_bold",
            },
            FontType {
                font: &VERDANA16,
                name: "verdana16",
            },
            FontType {
                font: &VERDANA16_BOLD,
                name: "verdana16_bold",
            },
            FontType {
                font: &VERDANA17,
                name: "verdana17",
            },
            FontType {
                font: &VERDANA17_BOLD,
                name: "verdana17_bold",
            },
            FontType {
                font: &VERDANA18,
                name: "verdana18",
            },
            FontType {
                font: &VERDANA18_BOLD,
                name: "verdana18_bold",
            },
        ];

        let mut glyph = GlyphRasterBin::new(core::ptr::null());
        let mut pixf = agg::PixBgr24::new_borrowed(rb);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut rt = agg::RendererRasterHtextSolid::new(&mut rb, &mut glyph);
        rt.set_color(agg::Rgba8::new_params(0, 0, 0, 255));

        let mut y = 5;
        for font in fonts {
            let buf = format!(
                "A quick brown fox jumps over the lazy dog 0123456789: {}",
                font.name
            );
            rt.glyph_gen_mut().set_font(font.font.as_ptr());
            rt.render_text(5., y as f64, &buf, !self.flip_y);
            y += rt.glyph_gen().height() as i32 + 1;
        }

        let mtx = agg::TransAffine::new_default();
        let mut grad_func =
            GradientSineRepeatAdaptor::<agg::GradientCircle>::new(agg::GradientCircle {});
        grad_func.periods(5.0);
        let mut color_func = agg::GradientLinearColor::new(
            agg::Rgba8::new_params(255, 0, 0, 255),
            agg::Rgba8::new_params(0, 125, 0, 255),
            256,
        );
        let mut inter = agg::SpanIpLinear::new(mtx);
        let sa = agg::VecSpan::<agg::Rgba8>::new();
        let mut sg = agg::SpanGradient::<
            agg::Rgba8,
            agg::SpanIpLinear<agg::TransAffine>,
            GradientSineRepeatAdaptor<agg::GradientCircle>,
            agg::GradientLinearColor<agg::Rgba8>,
        >::new(&mut inter, &mut grad_func, &mut color_func, 0., 150.0);
        let mut ren = agg::RendererScanlineAA::new_borrowed(&mut rb, sa, &mut sg);
        let mut rt2 = agg::RendererRasterHtext::new(&mut ren, &mut glyph);

        let text = "RADIAL REPEATING GRADIENT: A quick brown fox jumps over the lazy dog";
        rt2.render_text(5., 465., &text, !self.flip_y);
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Raster Text");

    if plat.init(640, 480, WindowFlag::Resize as u32) {
        plat.run();
    }
}

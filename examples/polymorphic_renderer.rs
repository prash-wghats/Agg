use crate::platform::*;

use agg::pixfmt_rgb_packed::*;
use agg::pixfmt_rgb::*;
use agg::pixfmt_rgba::*;
use agg::{
    Color, PixFmt, RasterScanLine, RenderBuf, RendererScanline, RendererScanlineColor, Scanline,
};

mod ctrl;
mod platform;
type Ptr<T> = Rc<RefCell<T>>;

use std::cell::RefCell;
use std::rc::Rc;

const FLIP_Y: bool = true;

const PIXFMT: PixFormat = 
	//PixFormat::Rgb555;
	//PixFormat::Rgb565;
	//PixFormat::Rgb24;
	//PixFormat::Bgr24;
	PixFormat::Rgba32;
	//PixFormat::Argb32;
	//PixFormat::Abgr32;
	//PixFormat::Bgra32;

pub struct PolymorphicRendererSolidRgba8Adaptor<'a, Pix: PixFmt> {
    pub ren: agg::RendererScanlineAASolid<'a, agg::RendererBase<'a, Pix>>,
}

impl<'a, Pix: PixFmt> PolymorphicRendererSolidRgba8Adaptor<'a, Pix> {
    pub fn new(pix: Pix) -> Self {
        let rb = agg::RendererBase::new_owned(pix);
        Self {
            ren: agg::RendererScanlineAASolid::new_owned(rb),
        }
    }
    pub fn clear(&mut self, c: &Pix::C) {
        self.ren.ren_mut().clear(c);
    }
}

impl<'a, Pix: PixFmt> RendererScanlineColor for PolymorphicRendererSolidRgba8Adaptor<'a, Pix> {
    type C = Pix::C;
    fn set_color(&mut self, c: Pix::C) {
        self.ren.set_color(c);
    }

    fn color(&self) -> Pix::C {
        self.ren.color()
    }
}

impl<'a, Pix: PixFmt> RendererScanline for PolymorphicRendererSolidRgba8Adaptor<'a, Pix> {
    fn prepare(&mut self) {
        self.ren.prepare();
    }

    fn render<Sl: Scanline>(&mut self, sl: &Sl) {
        self.ren.render(sl);
    }
}

pub struct Application {
    x: [f64; 3],
    y: [f64; 3],
    format: PixFormat,
    ctrls: CtrlContainer,
    _util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(format: PixFormat, _flip_y: bool, util: Ptr<PlatUtil>) -> Application {
        let x = [100.0, 369.0, 143.0];
        let y = [60.0, 170.0, 310.0];
        Self {
            x,
            y,
            format,
            ctrls: CtrlContainer {
                ctrl: vec![],
                cur_ctrl: -1,
                num_ctrl: 0,
            },
            _util: util,
        }
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut path = agg::PathStorage::new();
        path.move_to(self.x[0], self.y[0]);
        path.line_to(self.x[1], self.y[1]);
        path.line_to(self.x[2], self.y[2]);
        path.close_polygon(0);

        fn gen<Pix: PixFmt<C = agg::Rgba8>>(
            mut ren: PolymorphicRendererSolidRgba8Adaptor<Pix>, path: &mut agg::PathStorage,
        ) {
            let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
            let mut sl = agg::ScanlineP8::new();

            ren.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
            ren.set_color(agg::Rgba8::new_params(80, 30, 20, 255));
            ras.add_path(path, 0);
            agg::render_scanlines(&mut ras, &mut sl, &mut ren);
        }
        // Polymorphic renderer class factory
        match self.format {
            PixFormat::Rgb555 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixRgb555::new_borrowed(rb)),
                    &mut path,
                );
            }
            PixFormat::Rgb565 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixRgb565::new_borrowed(rb)),
                    &mut path,
                );
            }
			PixFormat::Rgb24 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixRgb24::new_borrowed(rb)),
                    &mut path,
                );
            },
			PixFormat::Bgr24 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixBgr24::new_borrowed(rb)),
                    &mut path,
                );
            },
			PixFormat::Rgba32 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixRgba32::new_borrowed(rb)),
                    &mut path,
                );
            },
			PixFormat::Argb32 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixArgb32::new_borrowed(rb)),
                    &mut path,
                );
            },
			PixFormat::Abgr32 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixAbgr32::new_borrowed(rb)),
                    &mut path,
                );
            },
			PixFormat::Bgra32 => {
                gen(
                    PolymorphicRendererSolidRgba8Adaptor::new(PixBgra32::new_borrowed(rb)),
                    &mut path,
                );
            },
            _ => panic!(),
        };
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXFMT, FLIP_Y);
    plat.set_caption(r#"AGG Example. Polymorphic Renderers"#);

    if plat.init(400, 330, WindowFlag::Resize as u32) {
        plat.run();
    }
}

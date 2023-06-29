use crate::ctrl::slider::Slider;
use crate::platform::{InputFlag, *};
use agg::math::*;
use agg::rendering_buffer::*;
use agg::{
    AggPrimitive, Args, Color, RasterScanLine, Renderer, RendererScanline, RendererScanlineColor,
    Scanline,
};
mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;
const FLIP_Y: bool = true;

pub struct Square {
    size: f64,
}

impl Square {
    pub fn new(size: f64) -> Square {
        Square { size: size }
    }

    pub fn draw<S: Scanline, Rn: Renderer>(
        &self, ras: &mut agg::RasterizerScanlineAa, sl: &mut S, ren: &mut Rn, color: Rn::C, x: f64,
        y: f64,
    ) {
        ras.reset();
        ras.move_to_d(x * self.size, y * self.size);
        ras.line_to_d(x * self.size + self.size, y * self.size);
        ras.line_to_d(x * self.size + self.size, y * self.size + self.size);
        ras.line_to_d(x * self.size, y * self.size + self.size);
        agg::render_scanlines_aa_solid(ras, sl, ren, &color);
    }
}

struct RendererEnlarged<'a, Ren: Renderer> {
    ras: agg::RasterizerScanlineAa,
    sl: agg::ScanlineU8,
    ren: &'a mut Ren,
    square: Square,
    color: Ren::C,
    _size: f64,
}

impl<'a, Ren: Renderer> RendererEnlarged<'a, Ren> {
    fn new(size: f64, ren: &'a mut Ren) -> RendererEnlarged<Ren> {
        RendererEnlarged {
            ras: agg::RasterizerScanlineAa::new(),
            sl: agg::ScanlineU8::new(),
            square: self::Square::new(size),
            ren,
            color: Ren::C::new(),
            _size: size,
        }
    }
}

impl<'a, Ren: Renderer> RendererScanlineColor for RendererEnlarged<'a, Ren> {
    type C = Ren::C;
    fn set_color(&mut self, c: Ren::C) {
        self.color = c;
    }
}

#[allow(arithmetic_overflow)]
impl<'a, Ren: Renderer> RendererScanline for RendererEnlarged<'a, Ren> {
    fn prepare(&mut self) {}

    fn render<Sl: Scanline>(&mut self, sl: &Sl) {
        let y = sl.y();

        for span in sl.begin() {
            let mut x = span.x;
            let mut covers = span.covers;
            let num_pix = span.len;

            for _i in 0..num_pix as usize {
                let a = unsafe { ((*covers) as u32 * self.color.a().into_u32()) >> 8 };
                unsafe {
                    covers = covers.offset(1);
                }
                let mut cc = self.color;
                *cc.a_mut() = AggPrimitive::from_u32(a);
                self.square.draw(
                    &mut self.ras,
                    &mut self.sl,
                    self.ren,
                    cc,
                    x as f64,
                    y as f64,
                );
                x += 1;
            }
        }
    }
}

pub struct Application {
    x: [f64; 3],
    y: [f64; 3],
    dx: f64,
    dy: f64,
    idx: i32,
    slider1: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    slider2: Rc<RefCell<Slider<'static, agg::Rgba8>>>,
    ctrls: CtrlContainer,
    _util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    pub fn init(&mut self) {
        //
        //self.add_ctrl(self.slider1);
        //self.add_ctrl(self.slider2);

        self.slider1.borrow_mut().set_range(8.0, 100.0);
        self.slider1.borrow_mut().set_num_steps(23);
        self.slider1.borrow_mut().set_value(32.0);

        self.slider2.borrow_mut().set_range(0.1, 3.0);
        self.slider2.borrow_mut().set_value(1.0);

        self.slider1.borrow_mut().set_label("Pixel size=%.1f");
        self.slider2.borrow_mut().set_label("Gamma=%4.3f");

        self.slider1.borrow_mut().no_transform();
        self.slider2.borrow_mut().no_transform();
    }
}

impl Interface for Application {
    fn new(_format: PixFormat, _flip_y_: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let s0 = Rc::new(RefCell::new(Slider::new(
            80.,
            10.,
            600. - 10.,
            19.,
            !FLIP_Y,
        )));
        let s1 = Rc::new(RefCell::new(Slider::new(
            80.,
            10. + 20.,
            600. - 10.,
            19. + 20.,
            !FLIP_Y,
        )));
        let app = Self {
            ctrls: CtrlContainer {
                ctrl: vec![s0.clone(), s1.clone()],
                cur_ctrl: -1,
                num_ctrl: 2,
            },
            slider1: s0,
            slider2: s1,
            idx: -1,
            x: [57., 369., 143.],
            y: [100., 170., 310.],
            dx: 0.0,
            dy: 0.0,
            _util: util,
        };

        app
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rb);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let size_mul = (self.slider1.borrow().value()) as i32 as f64;
        let g = agg::GammaPower::new_with_gamma(self.slider2.borrow().value());
        ras.set_gamma(&g);

        let mut ren_en =
            RendererEnlarged::<agg::RendererBase<agg::PixBgr24>>::new(size_mul, &mut ren_base);

        ras.reset();
        ras.move_to_d(self.x[0] / size_mul, self.y[0] / size_mul);
        ras.line_to_d(self.x[1] / size_mul, self.y[1] / size_mul);
        ras.line_to_d(self.x[2] / size_mul, self.y[2] / size_mul);
        ren_en.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        agg::render_scanlines(&mut ras, &mut sl, &mut ren_en);

        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &agg::Rgba8::new_params(0, 0, 0, 255),
        );

        ras.set_gamma(&agg::GammaNone::new());

        let ps = agg::PathStorage::new();
        let mut pg: agg::ConvStroke<_> = agg::ConvStroke::new_owned(ps);
        pg.set_width(2.0);

        pg.source_mut().remove_all();
        pg.source_mut().move_to(self.x[0], self.y[0]);
        pg.source_mut().line_to(self.x[1], self.y[1]);
        ras.add_path(&mut pg, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &agg::Rgba8::new_params(0, 150, 160, 200),
        );

        pg.source_mut().remove_all();
        pg.source_mut().move_to(self.x[1], self.y[1]);
        pg.source_mut().line_to(self.x[2], self.y[2]);
        ras.add_path(&mut pg, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &agg::Rgba8::new_params(0, 150, 160, 200),
        );

        pg.source_mut().remove_all();
        pg.source_mut().move_to(self.x[2], self.y[2]);
        pg.source_mut().line_to(self.x[0], self.y[0]);
        ras.add_path(&mut pg, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &agg::Rgba8::new_params(0, 150, 160, 200),
        );

        // Render the controls
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.slider1.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.slider2.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            let mut i = 0;
            while i < 3 {
                if ((x as f64 - self.x[i]) * (x as f64 - self.x[i])
                    + (y as f64 - self.y[i]) * (y as f64 - self.y[i]))
                    .sqrt()
                    < 10.0
                {
                    self.dx = x as f64 - self.x[i];
                    self.dy = y as f64 - self.y[i];
                    self.idx = i as i32;
                    break;
                }
                i += 1;
            }
            if i == 3 {
                if point_in_triangle(
                    self.x[0], self.y[0], self.x[1], self.y[1], self.x[2], self.y[2], x as f64,
                    y as f64,
                ) {
                    self.dx = x as f64 - self.x[0];
                    self.dy = y as f64 - self.y[0];
                    self.idx = 3;
                }
            }
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            if self.idx == 3 {
                let dx = x as f64 - self.dx;
                let dy = y as f64 - self.dy;
                self.x[1] -= self.x[0] - dx;
                self.y[1] -= self.y[0] - dy;
                self.x[2] -= self.x[0] - dx;
                self.y[2] -= self.y[0] - dy;
                self.x[0] = dx;
                self.y[0] = dy;
                //self.return true
                return Draw::Yes;
            }

            if self.idx >= 0 {
                self.x[self.idx as usize] = x as f64 - self.dx;
                self.y[self.idx as usize] = y as f64 - self.dy;
                //self.return true
                return Draw::Yes;
            }
        } else {
            return self.on_mouse_button_up(_rb, x, y, flags);
        }
        Draw::No
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.idx = -1;
        Draw::No
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.app_mut().init();
    plat.set_caption("AGG Example. Anti-Aliasing Demo");

    if plat.init(600, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

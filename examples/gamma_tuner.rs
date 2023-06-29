use crate::platform::*;

use agg::{Args, Color, Renderer};

mod ctrl;
mod platform;

use crate::ctrl::rbox::Rbox;
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

struct Application {
    gamma: Ptr<Slider<'static, agg::Rgba8>>,
    r: Ptr<Slider<'static, agg::Rgba8>>,
    g: Ptr<Slider<'static, agg::Rgba8>>,
    b: Ptr<Slider<'static, agg::Rgba8>>,
    pattern: Ptr<Rbox<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let gamma = ctrl_ptr(Slider::new(5., 5., 350. - 5., 11., !flip_y));
        let r = ctrl_ptr(Slider::new(5., 5. + 15., 350. - 5., 11. + 15., !flip_y));
        let g = ctrl_ptr(Slider::new(5., 5. + 30., 350. - 5., 11. + 30., !flip_y));
        let b = ctrl_ptr(Slider::new(5., 5. + 45., 350. - 5., 11. + 45., !flip_y));
        let pattern = ctrl_ptr(Rbox::new(355., 1., 495., 60., !flip_y));

        pattern.borrow_mut().set_text_size(8., 0.);
        pattern.borrow_mut().add_item("Horizontal");
        pattern.borrow_mut().add_item("Vertical");
        pattern.borrow_mut().add_item("Checkered");
        pattern.borrow_mut().set_cur_item(2);

        gamma.borrow_mut().set_range(0.5, 4.0);
        gamma.borrow_mut().set_value(2.2);
        gamma.borrow_mut().set_label("Gamma=%.2f");

        r.borrow_mut().set_value(1.0);
        g.borrow_mut().set_value(1.0);
        b.borrow_mut().set_value(1.0);

        r.borrow_mut().set_label("R=%.2f");
        g.borrow_mut().set_label("G=%.2f");
        b.borrow_mut().set_label("B=%.2f");

        Application {
            gamma: gamma.clone(),
            r: r.clone(),
            g: g.clone(),
            b: b.clone(),
            pattern: pattern.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![gamma, r, g, b, pattern],
                cur_ctrl: -1,
                num_ctrl: 5,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let g = self.gamma.borrow().value();
        let gamma = agg::GammaLut::<
            <ColorType as Args>::ValueType,
            <ColorType as Args>::ValueType,
            { ColorType::BASE_SHIFT },
            { ColorType::BASE_SHIFT },
        >::new_with_gamma(g);

        let mut pixf = PixfmtGamma::new_borrowed(rbuf);
		pixf.blender_mut().set_gamma_owned(gamma);
        let mut renb = agg::RendererBase::new_borrowed(&mut pixf);

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineU8::new();

        const SQUARE_SIZE: usize = 400;
        const VER_STRIPS: usize = 5;

        let mut span1: [ColorType; SQUARE_SIZE] = [ColorType {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; SQUARE_SIZE];
        let mut span2: [ColorType; SQUARE_SIZE] = [ColorType {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; SQUARE_SIZE];

        let color = ColorType {
            r: (self.r.borrow().value() * 255.) as u8,
            g: (self.g.borrow().value() * 255.) as u8,
            b: (self.b.borrow().value() * 255.) as u8,
            a: 255,
        };

        // Draw vertical gradient
        //-----------------------
        let w = self.util.borrow().width() as i32;
        let h = self.util.borrow().height() as i32;
        for i in 0..h {
            let mut k = (i - 80) as f64 / (SQUARE_SIZE - 1) as f64;
            if i < 80 {
                k = 0.0;
            }
            if i >= 80 + SQUARE_SIZE as i32 {
                k = 1.0;
            }

            k = 1.0 - (k / 2.0).powf(1.0 / self.gamma.borrow().value());
            let c = color.gradient(
                &ColorType {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                k,
            );
            renb.copy_hline(0, i, w - 1, &c);
        }

        // Calculate spans
        //-----------------------
        match self.pattern.borrow().cur_item() {
            0 => {
                for i in 0..SQUARE_SIZE {
                    span1[i] = color;
                    span2[i] = color;
                    span1[i].a = ((i * ColorType::BASE_MASK as usize) / SQUARE_SIZE) as u8;
                    span2[i].a = ColorType::BASE_MASK as u8 - span1[i].a;
                }
            }
            1 => {
                for i in 0..SQUARE_SIZE {
                    span1[i] = color;
                    span2[i] = color;
                    if i & 1 == 1 {
                        span1[i].a = ((i * ColorType::BASE_MASK as usize) / SQUARE_SIZE) as u8;
                        span2[i].a = span1[i].a;
                    } else {
                        span1[i].a = (ColorType::BASE_MASK as usize
                            - (i * ColorType::BASE_MASK as usize) / SQUARE_SIZE)
                            as u8;
                        span2[i].a = span1[i].a;
                    }
                }
            }
            2 => {
                for i in 0..SQUARE_SIZE {
                    span1[i] = color;
                    span2[i] = color;
                    if i & 1 == 1 {
                        span1[i].a = ((i * ColorType::BASE_MASK as usize) / SQUARE_SIZE) as u8;
                        span2[i].a = ColorType::BASE_MASK as u8 - span1[i].a;
                    } else {
                        span2[i].a = ((i * ColorType::BASE_MASK as usize) / SQUARE_SIZE) as u8;
                        span1[i].a = ColorType::BASE_MASK as u8 - span2[i].a;
                    }
                }
            }
            _ => (),
        }

        // Clear the area
        //---------------------
        renb.copy_bar(
            50,
            80,
            50 + SQUARE_SIZE as i32 - 1,
            80 + SQUARE_SIZE as i32 - 1,
            &ColorType {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
        );
        // Draw the patern
        //---------------------
        for i in (0..SQUARE_SIZE as i32).step_by(2) {
            let k = i as f64 / (SQUARE_SIZE - 1) as f64;
            let k = 1.0 - f64::powf(k, 1.0 / g);
            let c = color.gradient(&ColorType::new_params(0, 0, 0, 255), k);
            for j in 0..SQUARE_SIZE {
                span1[j].r = c.r;
                span2[j].r = c.r;
                span1[j].g = c.g;
                span2[j].g = c.g;
                span1[j].b = c.b;
                span2[j].b = c.b;
            }
            renb.blend_color_hspan(50, i + 80 + 0, SQUARE_SIZE as i32, &span1, &[], 255);
            renb.blend_color_hspan(50, i + 80 + 1, SQUARE_SIZE as i32, &span2, &[], 255);
        }

        // Draw vertical strips
        //---------------------
        for i in 0..SQUARE_SIZE as i32 {
            let k = i as f64 / (SQUARE_SIZE - 1) as f64;
            let k = 1.0 - f64::powf(k / 2.0, 1.0 / g);
            let c = color.gradient(&ColorType::new_params(0, 0, 0, 255), k);
            for j in 0..VER_STRIPS as i32 {
                let xc = (SQUARE_SIZE as i32 * (j + 1)) / (VER_STRIPS as i32 + 1);
                renb.copy_hline(50 + xc - 10, i + 80, 50 + xc + 10, &c);
            }
        }

        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.gamma.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.r.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.g.borrow_mut());
        ctrl::render_ctrl(&mut ras, &mut sl, &mut renb, &mut *self.b.borrow_mut());
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut renb,
            &mut *self.pattern.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Gamma Tuner");

    if plat.init(500, 500, WindowFlag::Resize as u32) {
        plat.run();
    }
}

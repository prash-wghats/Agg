use crate::platform::*;

use agg::rendering_buffer::RenderBuf;
use agg::RasterScanLine;

mod ctrl;
mod platform;
use crate::ctrl::gamma::Gamma;

use core::f64::consts::PI;

use std::cell::RefCell;
use std::fs::*;
use std::io::*;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

struct Application {
    g_ctrl: Ptr<Gamma<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}
impl Application {
    fn read_gamma(&mut self, fname: &str) {
        let fd = File::open(fname);
        if fd.is_ok() {
            let mut buf = String::new();
            fd.unwrap().read_to_string(&mut buf);
            let split_buf: Vec<&str> = buf.split("\n").collect();
            let kx1 = split_buf[0].parse::<f64>().unwrap();
            let ky1 = split_buf[1].parse::<f64>().unwrap();
            let kx2 = split_buf[2].parse::<f64>().unwrap();
            let ky2 = split_buf[3].parse::<f64>().unwrap();
            /*fd.read_line(&mut buf).unwrap();
            kx1 = buf.trim().parse().unwrap();
            buf.clear();
            fd.read_line(&mut buf).unwrap();
            ky1 = buf.trim().parse().unwrap();
            buf.clear();
            fd.read_line(&mut buf).unwrap();
            kx2 = buf.trim().parse().unwrap();
            buf.clear();
            fd.read_line(&mut buf).unwrap();
            ky2 = buf.trim().parse().unwrap();
            buf.clear(); */
            self.g_ctrl.borrow_mut().set_values(kx1, ky1, kx2, ky2);
        }
    }

    fn write_gamma_bin(&mut self, fname: &str) {
        //let gamma = self.g_ctrl.borrow().gamma();
        if let Ok(mut fd) = File::create(fname) {
            fd.write_all(self.g_ctrl.borrow().gamma()).unwrap();
        }
    }

    fn write_gamma_txt(&mut self, fname: &str) {
        let fd = File::create(fname);
        if let Ok(mut fd) = fd {
            //let gamma = self.g_ctrl.borrow().gamma();
            let mut kx1 = 0.;
            let mut ky1 = 0.;
            let mut kx2 = 0.;
            let mut ky2 = 0.;
            self.g_ctrl
                .borrow_mut()
                .values(&mut kx1, &mut ky1, &mut kx2, &mut ky2);
            fd.write_all(format!("{:.3}\n", kx1).as_bytes()).unwrap();
            fd.write_all(format!("{:.3}\n", ky1).as_bytes()).unwrap();
            fd.write_all(format!("{:.3}\n", kx2).as_bytes()).unwrap();
            fd.write_all(format!("{:.3}\n", ky2).as_bytes()).unwrap();
            let mut temp = |gamma: &[u8]| 
                for i in 0..16 {
                    for j in 0..16 {
                        fd.write_all(format!("{:3},", gamma[i * 16 + j]).as_bytes())
                            .unwrap();
                    }
                    fd.write_all(b"\n").unwrap();
                }
            ;
            temp(self.g_ctrl.borrow().gamma());
        }
    }
}

impl Drop for Application {
    fn drop(&mut self) {
        self.write_gamma_txt("gamma.txt");
        self.write_gamma_bin("gamma.bin");
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let g_ctrl = ctrl_ptr(Gamma::new(10.0, 10.0, 300.0, 200.0, !flip_y));
        Application {
            g_ctrl: g_ctrl.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![g_ctrl],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        self.read_gamma("gamma.txt");
    }

    fn on_draw(&mut self, rb: &mut RenderBuf) {
        let ewidth = self.util.borrow().initial_width() / 2.0 - 10.0;
        let ecenter = self.util.borrow().initial_width() / 2.0;
        let mut color;
        let mut pixf = Pixfmt::new_borrowed(rb);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);

        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        self.g_ctrl.borrow_mut().set_text_size(10.0, 12.0);

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineP8::new();

        ctrl::render_ctrl(&mut ras, &mut sl, &mut rb, &mut *self.g_ctrl.borrow_mut());

        ras.set_gamma(&*self.g_ctrl.borrow()); //XXX

        let ellipse = agg::ellipse::Ellipse::new();
        let poly: agg::ConvStroke<'_, _> = agg::conv_stroke::ConvStroke::new_owned(ellipse);
        let mut tpoly = agg::conv_transform::ConvTransform::new_owned(
            poly,
            *self.util.borrow_mut().trans_affine_resizing(),
        );
        color = agg::Rgba8::new_params(0, 0, 0, 255);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 220.0, ewidth, 15.0, 100, false);
        tpoly.source_mut().set_width(2.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 220.0, 11.0, 11.0, 100, false);
        tpoly.source_mut().set_width(2.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        color = agg::Rgba8::new_params(127, 127, 127, 255);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 260.0, ewidth, 15.0, 100, false);
        tpoly.source_mut().set_width(2.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 260.0, 11.0, 11.0, 100, false);
        tpoly.source_mut().set_width(2.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        color = agg::Rgba8::new_params(192, 192, 192, 155);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 300.0, ewidth, 15.0, 100, false);
        tpoly.source_mut().set_width(2.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 300.0, 11.0, 11.0, 100, false);
        tpoly.source_mut().set_width(2.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        color = agg::Rgba8::new_params(0, 0, 100, 255);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 340.0, ewidth, 15.5, 100, false);
        tpoly.source_mut().set_width(1.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 340.0, 10.5, 10.5, 100, false);
        tpoly.source_mut().set_width(1.0);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 380.0, ewidth, 15.5, 100, false);
        tpoly.source_mut().set_width(0.4);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 380.0, 10.5, 10.5, 100, false);
        tpoly.source_mut().set_width(0.4);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 420.0, ewidth, 15.5, 100, false);
        tpoly.source_mut().set_width(0.1);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        tpoly
            .source_mut()
            .source_mut()
            .init(ecenter, 420.0, 10.5, 10.5, 100, false);
        tpoly.source_mut().set_width(0.1);
        ras.add_path(&mut tpoly, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        let mut mtx = agg::TransAffine::new_default();
        mtx *= agg::TransAffine::trans_affine_skewing(0.15, 0.0);
        mtx *= *self.util.borrow_mut().trans_affine_resizing();
        let text = agg::GsvText::new();
        let mut text1 = agg::GsvTextOutline::new(text, mtx.clone());
        text1.source_mut().set_text("Text 2345");
        text1.source_mut().set_size(50.0, 20.0);
        text1.set_width(2.0);
        text1.source_mut().set_start_point(320.0, 10.0);

        color = agg::Rgba8::new_params(0, 125, 0, 255);
        ras.add_path(&mut text1, 0);
        agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);

        color = agg::Rgba8::new_params(125, 0, 0, 255);
        let mut path = agg::path_storage::PathStorage::new();
        path.move_to(30.0, -1.0);
        path.line_to(60.0, 0.0);
        path.line_to(30.0, 1.0);

        path.move_to(27.0, -1.0);
        path.line_to(10.0, 0.0);
        path.line_to(27.0, 1.0);

        let mut trans = agg::ConvTransform::new_owned(path, mtx);

        for i in 0..35 {
            trans.trans_mut().reset();
            *trans.trans_mut() *= agg::TransAffine::trans_affine_rotation(i as f64 / 35.0 * PI * 2.0);
            *trans.trans_mut() *= agg::TransAffine::trans_affine_translation(400.0, 130.0);
            *trans.trans_mut() *= *self.util.borrow_mut().trans_affine_resizing();
            ras.add_path(&mut trans, 0);
            agg::render_scanlines_aa_solid(&mut ras, &mut sl, &mut rb, &color);
        }
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("Anti-Aliasing Gamma Correction");

    if plat.init(500, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

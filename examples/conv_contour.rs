use crate::platform::*;
use agg::basics::PathFlag;
use agg::rendering_buffer::RenderBuf;
use agg::RasterScanLine;

mod ctrl;
mod platform;
use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;

use std::cell::RefCell;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

struct Application {
    close: Ptr<Rbox<'static, agg::Rgba8>>,
    width: Ptr<Slider<'static, agg::Rgba8>>,
    auto_detect: Ptr<Cbox<'static, agg::Rgba8>>,
    path: agg::PathStorage,
    pub ctrls: CtrlContainer,
    _util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    pub fn compose_path(&mut self) {
        let flag = match self.close.borrow().cur_item() {
            1 => PathFlag::Cw,
            2 => PathFlag::Ccw,
            _ => PathFlag::None,
        };
        self.path.remove_all();
        self.path.move_to(28.47, 6.45);
        self.path.curve3_ctrl(21.58, 1.12, 19.82, 0.29);
        self.path.curve3_ctrl(17.19, -0.93, 14.21, -0.93);
        self.path.curve3_ctrl(9.57, -0.93, 6.57, 2.25);
        self.path.curve3_ctrl(3.56, 5.42, 3.56, 10.60);
        self.path.curve3_ctrl(3.56, 13.87, 5.03, 16.26);
        self.path.curve3_ctrl(7.03, 19.58, 11.99, 22.51);
        self.path.curve3_ctrl(16.94, 25.44, 28.47, 29.64);
        self.path.line_to(28.47, 31.40);
        self.path.curve3_ctrl(28.47, 38.09, 26.34, 40.58);
        self.path.curve3_ctrl(24.22, 43.07, 20.17, 43.07);
        self.path.curve3_ctrl(17.09, 43.07, 15.28, 41.41);
        self.path.curve3_ctrl(13.43, 39.75, 13.43, 37.60);
        self.path.line_to(13.53, 34.77);
        self.path.curve3_ctrl(13.53, 32.52, 12.38, 31.30);
        self.path.curve3_ctrl(11.23, 30.08, 9.38, 30.08);
        self.path.curve3_ctrl(7.57, 30.08, 6.42, 31.35);
        self.path.curve3_ctrl(5.27, 32.62, 5.27, 34.81);
        self.path.curve3_ctrl(5.27, 39.01, 9.57, 42.53);
        self.path.curve3_ctrl(13.87, 46.04, 21.63, 46.04);
        self.path.curve3_ctrl(27.59, 46.04, 31.40, 44.04);
        self.path.curve3_ctrl(34.28, 42.53, 35.64, 39.31);
        self.path.curve3_ctrl(36.52, 37.21, 36.52, 30.71);
        self.path.line_to(36.52, 15.53);
        self.path.curve3_ctrl(36.52, 9.13, 36.77, 7.69);
        self.path.curve3_ctrl(37.01, 6.25, 37.57, 5.76);
        self.path.curve3_ctrl(38.13, 5.27, 38.87, 5.27);
        self.path.curve3_ctrl(39.65, 5.27, 40.23, 5.62);
        self.path.curve3_ctrl(41.26, 6.25, 44.19, 9.18);
        self.path.line_to(44.19, 6.45);
        self.path.curve3_ctrl(38.72, -0.88, 33.74, -0.88);
        self.path.curve3_ctrl(31.35, -0.88, 29.93, 0.78);
        self.path.curve3_ctrl(28.52, 2.44, 28.47, 6.45);
        self.path.close_polygon(flag as u32);

        self.path.move_to(28.47, 9.62);
        self.path.line_to(28.47, 26.66);
        self.path.curve3_ctrl(21.09, 23.73, 18.95, 22.51);
        self.path.curve3_ctrl(15.09, 20.36, 13.43, 18.02);
        self.path.curve3_ctrl(11.77, 15.67, 11.77, 12.89);
        self.path.curve3_ctrl(11.77, 9.38, 13.87, 7.06);
        self.path.curve3_ctrl(15.97, 4.74, 18.70, 4.74);
        self.path.curve3_ctrl(22.41, 4.74, 28.47, 9.62);
        self.path.close_polygon(flag as u32);
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let close = ctrl_ptr(Rbox::new(10.0, 10.0, 130.0, 80.0, !flip_y));
        let width = ctrl_ptr(Slider::new(
            130.0 + 10.0,
            10.0 + 4.0,
            130.0 + 300.0,
            10.0 + 8.0 + 4.0,
            !flip_y,
        ));
        let auto_detect = ctrl_ptr(Cbox::new(
            130.0 + 10.0,
            10.0 + 4.0 + 16.0,
            "Autodetect orientation if not defined",
            !flip_y,
        ));
        close.borrow_mut().add_item("Close");
        close.borrow_mut().add_item("Close CW");
        close.borrow_mut().add_item("Close CCW");
        close.borrow_mut().set_cur_item(0);

        width.borrow_mut().set_range(-100.0, 100.0);
        width.borrow_mut().set_value(0.0);
        width.borrow_mut().set_label("Width=%1.2f");
        Self {
            path: agg::path_storage::PathStorage::new(),
            ctrls: CtrlContainer {
                ctrl: vec![close.clone(), width.clone(), auto_detect.clone()],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            close,
            width,
            auto_detect,
            _util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        let mut sl = agg::ScanlineP8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut mtx = agg::TransAffine::new_default();
        mtx *= agg::TransAffine::trans_affine_scaling_eq(4.0);
        mtx *= agg::TransAffine::trans_affine_translation(150., 100.);

        self.compose_path();

        let trans = agg::ConvTransform::new_borrowed(&mut self.path, mtx);
        let curve: agg::ConvCurve<'_, _> = agg::ConvCurve::new_owned(trans);
        let mut contour = agg::ConvContour::new_owned(curve);
        contour.set_width(self.width.borrow().value());
        contour.set_auto_detect_orientation(self.auto_detect.borrow().status());

        ras.add_path(&mut contour, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &agg::Rgba8::new_params(0, 0, 0, 255),
        );

        // Render the controls
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.close.borrow_mut(),
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
            &mut *self.auto_detect.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Contour Tool & Polygon Orientation");

    if plat.init(440, 330, WindowFlag::Resize as u32) {
        plat.run();
    }
}

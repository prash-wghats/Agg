use crate::ctrl::slider::*;
use crate::platform::*;

use agg::bounding_rect::*;
use agg::path_storage::*;
use agg::rasterizer_scanline_aa::*;
use agg::renderer_scanline::render_all_paths;
use agg::scanline_p::*;
use agg::trans_affine::TransAffine;
use agg::RenderBuf;

mod ctrl;
mod platform;

use core::f64::consts::PI;
use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use crate::misc::parse_lion::*;

const FLIP_Y: bool = true;

struct Application {
    magn_slider: Ptr<Slider<'static, agg::Rgba8>>,
    radius_slider: Ptr<Slider<'static, agg::Rgba8>>,
    rasterizer: RasterizerScanlineAa,
    scanline: ScanlineP8,
    path: PathStorage,
    colors: [agg::Rgba8; 100],
    path_idx: [u32; 100],
    npaths: u32,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    base_dx: f64,
    base_dy: f64,
    angle: f64,
    scale: f64,
    skew_x: f64,
    skew_y: f64,
    nclick: i32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

type Ptr<T> = Rc<RefCell<T>>;
fn crt_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

impl Application {
    fn parse_lion(&mut self) -> u32 {
        self.npaths = parse_lion(&mut self.path, &mut self.colors, &mut self.path_idx);
        bounding_rect(
            &mut self.path,
            self.path_idx,
            0,
            self.npaths,
            &mut self.x1,
            &mut self.y1,
            &mut self.x2,
            &mut self.y2,
        );
        self.base_dx = (self.x2 - self.x1) / 2.0;
        self.base_dy = (self.y2 - self.y1) / 2.0;
        self.npaths
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn on_init(&mut self) {
        self.x1 = 200.;
        self.y1 = 150.;
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let width = self.util.borrow().width();
        let height = self.util.borrow().height();

        let mut pix = agg::PixBgr24::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::<agg::PixBgr24>::new_borrowed(&mut pix);
        rb.clear(&agg::Rgba8::new_params(255, 255, 255, 255));
        let mut r = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
        let mut lens = agg::TransWarpMagnifier::new();
        lens.center(self.x1, self.y1);
        lens.magnification(self.magn_slider.borrow().value());
        lens.radius(self.radius_slider.borrow().value() / self.magn_slider.borrow().value());
        let mut mtx = TransAffine::new_default();
        mtx.multiply(&TransAffine::trans_affine_translation(
            -self.base_dx,
            -self.base_dy,
        ));
        mtx.multiply(&TransAffine::trans_affine_rotation(self.angle + PI));
        mtx.multiply(&TransAffine::trans_affine_translation(
            width / 2.,
            height / 2.,
        ));

        let segm = agg::ConvSegmentator::new_borrowed(&mut self.path);
        let trans_mtx = agg::ConvTransform::new_owned(segm, mtx);
        let mut trans_lens = agg::ConvTransform::new_owned(trans_mtx, lens);
        render_all_paths(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut r,
            &mut trans_lens,
            &self.colors,
            &self.path_idx,
            self.npaths,
        );

        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.magn_slider.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut self.rasterizer,
            &mut self.scanline,
            &mut rb,
            &mut *self.radius_slider.borrow_mut(),
        );
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & InputFlag::MouseLeft as u32 != 0 {
            self.x1 = x as f64;
            self.y1 = y as f64;
            return Draw::Yes;
        }

        if flags & InputFlag::MouseRight as u32 != 0 {
            self.x2 = x as f64;
            self.y2 = y as f64;
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32) -> Draw {
        Draw::No //self.on_mouse_button_down(x, y, flags)
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Application {
        let magn_slider = crt_ptr(Slider::new(5., 5., 495., 12., !flip_y));
        let radius_slider = crt_ptr(Slider::new(5., 20., 495., 27., !flip_y));

        magn_slider.borrow_mut().no_transform();
        magn_slider.borrow_mut().set_range(0.01, 4.0);
        magn_slider.borrow_mut().set_value(3.0);
        magn_slider.borrow_mut().set_label("Scale %3.2f");

        radius_slider.borrow_mut().no_transform();
        radius_slider.borrow_mut().set_range(0.0, 100.0);
        radius_slider.borrow_mut().set_value(70.0);
        radius_slider.borrow_mut().set_label("Radius %3.2f");

        let mut app = Application {
            magn_slider: magn_slider.clone(),
            radius_slider: radius_slider.clone(),
            rasterizer: RasterizerScanlineAa::new(),
            scanline: ScanlineP8::new(),
            path: PathStorage::new(),
            colors: [agg::Rgba8::default(); 100],
            path_idx: [0; 100],
            npaths: 0,
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            base_dx: 0.0,
            base_dy: 0.0,
            angle: 0.0,
            scale: 1.0,
            skew_x: 0.0,
            skew_y: 0.0,
            nclick: 0,
            ctrls: CtrlContainer {
                ctrl: vec![magn_slider, radius_slider],
                cur_ctrl: -1,
                num_ctrl: 2,
            },
            util: util,
        };
        app.parse_lion();
        app
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption(r#"AGG Example. Lion"#);

    if plat.init(500, 600, WindowFlag::Resize as u32) {
        plat.run();
    }
}

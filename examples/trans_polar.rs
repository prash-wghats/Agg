use agg::{Color, RenderBuf, Transformer, VertexSource};

mod ctrl;
mod platform;
use crate::ctrl::slider::Slider;
use crate::platform::*;

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

const FLIP_Y: bool = true;

struct TransPolar {
    base_angle: f64,
    base_scale: f64,
    base_x: f64,
    base_y: f64,
    translation_x: f64,
    translation_y: f64,
    spiral: f64,
}

impl TransPolar {
    fn new() -> TransPolar {
        TransPolar {
            base_angle: 1.0,
            base_scale: 1.0,
            base_x: 0.0,
            base_y: 0.0,
            translation_x: 0.0,
            translation_y: 0.0,
            spiral: 0.0,
        }
    }

    fn base_scale(&mut self, v: f64) {
        self.base_scale = v;
    }

    fn full_circle(&mut self, v: f64) {
        self.base_angle = 2.0 * std::f64::consts::PI / v;
    }

    fn base_offset(&mut self, dx: f64, dy: f64) {
        self.base_x = dx;
        self.base_y = dy;
    }

    fn translation(&mut self, dx: f64, dy: f64) {
        self.translation_x = dx;
        self.translation_y = dy;
    }

    fn spiral(&mut self, v: f64) {
        self.spiral = v;
    }
}

impl Transformer for TransPolar {
    fn transform(&self, x: &mut f64, y: &mut f64) {
        let x1 = (*x + self.base_x) * self.base_angle;
        let y1 = (*y + self.base_y) * self.base_scale + (*x * self.spiral);
        *x = f64::cos(x1) * y1 + self.translation_x;
        *y = f64::sin(x1) * y1 + self.translation_y;
    }

    fn scaling_abs(&self, x: &mut f64, y: &mut f64) {
        *x = 0.;
        *y = 0.;
    }
}

struct TransformedControl<'a, ColorT: Color, Ctr: ctrl::CtrlColor, Pipeline: VertexSource> {
    ctrl: &'a mut Ctr,
    pipeline: Pipeline,
    dum: PhantomData<ColorT>,
}

impl<'a, ColorT: Color, Ctr: ctrl::CtrlColor, Pipeline: VertexSource>
    TransformedControl<'a, ColorT, Ctr, Pipeline>
{
    fn new(ctrl: &'a mut Ctr, pipeline: Pipeline) -> Self {
        Self {
            ctrl: ctrl,
            pipeline: pipeline,
            dum: PhantomData,
        }
    }
}

impl<'a, ColorT: Color, Ctr: ctrl::CtrlColor, Pipeline: VertexSource> ctrl::Ctrl
    for TransformedControl<'a, ColorT, Ctr, Pipeline>
{
    fn num_paths(&self) -> u32 {
        self.ctrl.num_paths()
    }
    fn set_transform(&mut self, _mtx: &agg::TransAffine) {}
    fn in_rect(&self, _x: f64, _y: f64) -> bool {
        false
    }
    fn on_mouse_button_down(&mut self, _x: f64, _y: f64) -> bool {
        false
    }
    fn on_mouse_button_up(&mut self, _x: f64, _y: f64) -> bool {
        false
    }
    fn on_mouse_move(&mut self, _x: f64, _y: f64, _button_flag: bool) -> bool {
        false
    }
    fn on_arrow_keys(&mut self, _left: bool, _right: bool, _down: bool, _up: bool) -> bool {
        false
    }
}

impl<'a, ColorT: Color, Ctr: ctrl::CtrlColor<Col = ColorT>, Pipeline: VertexSource> ctrl::CtrlColor
    for TransformedControl<'a, ColorT, Ctr, Pipeline>
{
    type Col = ColorT;
    fn color(&self, i: u32) -> ColorT {
        self.ctrl.color(i)
    }
}

impl<'a, ColorT: Color, Ctr: ctrl::CtrlColor, Pipeline: VertexSource> VertexSource
    for TransformedControl<'a, ColorT, Ctr, Pipeline>
{
    fn rewind(&mut self, path_id: u32) {
        self.pipeline.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.pipeline.vertex(x, y)
    }
}

struct Application {
    slider1: Ptr<Slider<'static, agg::Rgba8>>,
    slider_spiral: Ptr<Slider<'static, agg::Rgba8>>,
    slider_base_y: Ptr<Slider<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let slider1 = ctrl_ptr(Slider::new(10.0, 10.0, 600.0 - 10.0, 17.0, !flip_y));
        let slider_spiral = ctrl_ptr(Slider::new(
            10.0,
            10.0 + 20.0,
            600.0 - 10.0,
            17.0 + 20.0,
            !flip_y,
        ));
        let slider_base_y = ctrl_ptr(Slider::new(
            10.0,
            10.0 + 40.0,
            600.0 - 10.0,
            17.0 + 40.0,
            !flip_y,
        ));
        slider1.borrow_mut().set_range(0.0, 100.0);
        slider1.borrow_mut().set_num_steps(5);
        slider1.borrow_mut().set_value(32.0);
        slider1.borrow_mut().set_label("Some Value=%1.f");

        slider_spiral.borrow_mut().set_label("Spiral=%0.3f");
        slider_spiral.borrow_mut().set_range(-0.1, 0.1);
        slider_spiral.borrow_mut().set_value(0.0);

        slider_base_y.borrow_mut().set_label("Base Y=%0.3f");
        slider_base_y.borrow_mut().set_range(50.0, 200.0);
        slider_base_y.borrow_mut().set_value(120.0);

        Application {
            slider1: slider1.clone(),
            slider_spiral: slider_spiral.clone(),
            slider_base_y: slider_base_y.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![slider1, slider_spiral, slider_base_y],
                cur_ctrl: -1,
                num_ctrl: 3,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = agg::PixBgr24::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::new_borrowed(&mut pixf);
        //let ren = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);
        let mut sl = agg::ScanlineU8::new();

        ren_base.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

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
            &mut *self.slider_spiral.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &mut *self.slider_base_y.borrow_mut(),
        );
        let tm = &mut *self.slider1.borrow_mut();
        let segm = agg::ConvSegmentator::new_borrowed(tm);
        let mut trans = TransPolar::new();
        trans.full_circle(-600.0);
        trans.base_scale(-1.0);
        trans.base_offset(0.0, self.slider_base_y.borrow().value());
        trans.translation(
            self.util.borrow().width() / 2.0,
            self.util.borrow().height() / 2.0 + 30.0,
        );
        trans.spiral(-self.slider_spiral.borrow().value());
        let pipeline = agg::ConvTransform::new_owned(segm, trans);

        let mut tm1 = Slider::new(10.0, 10.0, 600.0 - 10.0, 17.0, true);
        let mut ctr: TransformedControl<'_, agg::Rgba8, _, _> =
            TransformedControl::new(&mut tm1, pipeline);
        ctrl::render_ctrl(&mut ras, &mut sl, &mut ren_base, &mut ctr);
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);
    plat.set_caption("AGG Example. Polar Transformer");

    if plat.init(600, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

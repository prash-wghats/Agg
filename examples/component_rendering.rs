use agg::RasterScanLine;

use crate::ctrl::slider::Slider;
use crate::platform::*;

mod ctrl;
mod platform;

use std::cell::RefCell;
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

const FLIP_Y: bool = true;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

pub struct Application {
    m_alpha: Ptr<Slider<'static, agg::Rgba8>>,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let m_alpha = ctrl_ptr(Slider::new(5., 5., 320. - 5., 10. + 5., !flip_y));
        m_alpha.borrow_mut().set_label("Alpha=%1.f");
        m_alpha.borrow_mut().set_range(0., 255.);
        m_alpha.borrow_mut().set_value(255.);

        Application {
            m_alpha: m_alpha.clone(),
            ctrls: CtrlContainer {
                ctrl: vec![m_alpha],
                cur_ctrl: -1,
                num_ctrl: 1,
            },
            util: util,
        }
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pf = agg::PixBgr24::new_owned(rbuf.clone());

        let mut pfr =
            agg::AlphaBlendGray::<agg::Gray8, agg::BlenderGray8, agg::RenderBuf, 3, 2>::new_owned(
                rbuf.clone(),
                
            );
        let mut pfg =
            agg::AlphaBlendGray::<agg::Gray8, agg::BlenderGray8, agg::RenderBuf, 3, 1>::new_owned(
                rbuf.clone(),
                
            );
        let mut pfb = agg::AlphaBlendGray::<agg::Gray8, agg::BlenderGray8, agg::RenderBuf, 3, 0>::new_borrowed(
            rbuf,
            
        );

        let mut rbase = agg::RendererBase::new_borrowed(&mut pf);
        let mut rbr = agg::RendererBase::new_borrowed(&mut pfr);
        let mut rbg = agg::RendererBase::new_borrowed(&mut pfg);
        let mut rbb = agg::RendererBase::new_borrowed(&mut pfb);

        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        let mut sl = agg::ScanlineP8::new();

        rbase.clear(&agg::Rgba8::new_params(255, 255, 255, 255));

        let mut er = agg::Ellipse::new_ellipse(
            self.util.borrow().width() / 2. - 0.87 * 50.,
            self.util.borrow().height() / 2. - 0.5 * 50.,
            100.,
            100.,
            100,
false,
        );
        ras.add_path(&mut er, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut rbr,
            &agg::Gray8::new_params(0, self.m_alpha.borrow().value() as u32),
        );

        let mut eg = agg::Ellipse::new_ellipse(
            self.util.borrow().width() / 2. + 0.87 * 50.,
            self.util.borrow().height() / 2. - 0.5 * 50.,
            100.,
            100.,
            100,
false,
        );
        ras.add_path(&mut eg, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut rbg,
            &agg::Gray8::new_params(0, self.m_alpha.borrow().value() as u32),
        );

        let mut eb = agg::Ellipse::new_ellipse(
            self.util.borrow().width() / 2.,
            self.util.borrow().height() / 2. + 50.,
            100.,
            100.,
            100,
false,
        );
        ras.add_path(&mut eb, 0);
        agg::render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut rbb,
            &agg::Gray8::new_params(0, self.m_alpha.borrow().value() as u32),
        );

        ctrl::render_ctrl(
            &mut ras,
            &mut sl,
            &mut rbase,
            &mut *self.m_alpha.borrow_mut(),
        );
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Component Rendering");

    if plat.init(320, 320, WindowFlag::Resize as u32) {
        plat.run();
    }
}

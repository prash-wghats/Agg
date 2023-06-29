use crate::platform::*;
use agg::rendering_buffer::RenderBuf;
use agg::{
    Color, ConvCurve, Gamma, PathStorage, RasterScanLine, RasterStyle, RendererScanlineColor,
    VertexSource,
};

mod ctrl;
mod platform;

use libc::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::Index;
use std::rc::Rc;

fn frand() -> i32 {
    unsafe { rand() }
}

trait ReadLine {
    fn readline(&mut self, buf: &mut String) -> bool;
}

impl ReadLine for BufReader<File> {
    fn readline(&mut self, buf: &mut String) -> bool {
        if let Ok(i) = self.read_line(buf) {
            if i == 0 {
                return false;
            }
            return true;
        }
        false
    }
}
const FLIP_Y: bool = false;

#[derive(Debug)]
struct PathStyle {
    path_id: u32,
    left_fill: i32,
    right_fill: i32,
    line: i32,
}
impl PathStyle {
    fn new() -> Self {
        Self {
            path_id: 0,
            left_fill: 0,
            right_fill: 0,
            line: 0,
        }
    }
}
struct CompoundShape<'a> {
    trans: agg::ConvTransform<'a, ConvCurve<'a, PathStorage>, agg::TransAffine>,
    styles: Vec<PathStyle>,
    _x1: f64,
    _y1: f64,
    _x2: f64,
    _y2: f64,
    buf_rdr: Option<BufReader<File>>,
}

impl<'a> VertexSource for CompoundShape<'a> {
    fn rewind(&mut self, path_id: u32) {
        self.trans.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.trans.vertex(x, y)
    }
}

impl<'a> Index<usize> for CompoundShape<'a> {
    type Output = u32;
    fn index(&self, i: usize) -> &Self::Output {
        &self.styles[i].path_id
    }
}
impl<'a> CompoundShape<'a> {
    fn new() -> Self {
        let p = agg::PathStorage::new();
        let mtx = agg::TransAffine::new_default();
        let cc = agg::ConvCurve::new_owned(p);
        let tr = agg::ConvTransform::new_owned(cc, mtx);
        CompoundShape {
            trans: tr,
            styles: Vec::new(),
            _x1: 0.0,
            _y1: 0.0,
            _x2: 0.0,
            _y2: 0.0,
            buf_rdr: None,
        }
    }

    fn open(&mut self, fname: &str) -> bool {
        if let Ok(fd) = std::fs::File::open(fname) {
            self.buf_rdr = Some(BufReader::new(fd));
            return true;
        }
        false
    }

    fn paths(&self) -> usize {
        self.styles.len()
    }

    fn style(&self, i: usize) -> &PathStyle {
        &self.styles[i]
    }

    #[allow(dead_code)]
    fn scale(&self) -> f64 {
        self.trans.trans().scale()
    }

    fn modify_vertex(&mut self, i: u32, x: f64, y: f64) {
        let (mut x, mut y) = (x, y);
        self.trans.trans_mut().inverse_transform(&mut x, &mut y);
        self.trans.source_mut().source_mut().modify_vertex(i, x, y);
    }

    fn set_scale(&mut self, w: f64, h: f64) {
        self.trans.trans_mut().reset();
        let (mut x1, mut y1, mut x2, mut y2) = (0., 0., 0., 0.);
        let mut tmp = vec![];
        for i in 0..self.styles.len() {
            tmp.push(self.styles[i].path_id);
        }
        agg::bounding_rect(
            self.trans.source_mut().source_mut(),
            tmp,
            0,
            self.styles.len() as u32,
            &mut x1,
            &mut y1,
            &mut x2,
            &mut y2,
        );
        if x1 < x2 && y1 < y2 {
            let mut vp = agg::TransViewport::new();
            vp.set_preserve_aspect_ratio(0.5, 0.5, agg::AspectRatio::Meet);
            vp.set_world_viewport(x1, y1, x2, y2);
            vp.set_device_viewport(0.0, 0.0, w, h);
            *self.trans.trans_mut() = vp.to_affine();
        }
        let s = self.trans.trans_mut().scale();
        self.trans.source_mut().set_approximation_scale(s);
    }

    fn set_approximation_scale(&mut self, s: f64) {
        let sc = self.trans.trans_mut().scale();
        self.trans.source_mut().set_approximation_scale(sc * s);
    }

    fn read_next(&mut self) -> bool {
        self.trans.source_mut().source_mut().remove_all();
        self.styles.clear();
        //const SPACE: &str = " \t\n\r";
        let mut ax: f64;
        let mut ay: f64;
        let mut cx: f64;
        let mut cy: f64;

        if let Some(rdr) = &mut self.buf_rdr {
            let mut buf = String::new();
            loop {
                if !rdr.readline(&mut buf) {
                    return false;
                }
                if buf.as_bytes()[0] == b'=' {
                    break;
                }
            }
            buf.clear();
            while rdr.readline(&mut buf) {
                if buf.as_bytes()[0] == b'!' {
                    break;
                }
                if buf.as_bytes()[0] == b'P' {
                    let mut style = PathStyle::new();
                    let mut iter = buf.split_whitespace();

                    style.path_id = self.trans.source_mut().source_mut().start_new_path();
                    let mut ts = iter.next(); // Path;
                    ts = iter.next(); // left_style
                    style.left_fill = ts.unwrap().parse::<i32>().unwrap();
                    ts = iter.next(); // right_style
                    style.right_fill = ts.unwrap().parse::<i32>().unwrap();
                    ts = iter.next();
                    style.line = ts.unwrap().parse::<i32>().unwrap();
                    ts = iter.next();
                    ax = ts.unwrap().parse::<f64>().unwrap();
                    ts = iter.next();
                    ay = ts.unwrap().parse::<f64>().unwrap();
                    self.trans.source_mut().source_mut().move_to(ax, ay);
                    self.styles.push(style);
                }

                if buf.as_bytes()[0] == b'C' {
                    let mut iter = buf.split_whitespace();
                    let mut ts = iter.next(); // Curve;
                    ts = iter.next();
                    cx = ts.unwrap().parse::<f64>().unwrap(); // cx
                    ts = iter.next();
                    cy = ts.unwrap().parse::<f64>().unwrap(); // cy
                    ts = iter.next();
                    ax = ts.unwrap().parse::<f64>().unwrap();
                    ts = iter.next();
                    ay = ts.unwrap().parse::<f64>().unwrap();
                    self.trans
                        .source_mut()
                        .source_mut()
                        .curve3_ctrl(cx, cy, ax, ay);
                }

                if buf.as_bytes()[0] == b'L' {
                    let mut iter = buf.split_whitespace();
                    let mut ts = iter.next(); // Line;
                    ts = iter.next();
                    ax = ts.unwrap().parse::<f64>().unwrap();
                    ts = iter.next();
                    ay = ts.unwrap().parse::<f64>().unwrap();
                    self.trans.source_mut().source_mut().line_to(ax, ay)
                }

                if buf.as_bytes()[0] == b'<' {
                    // EndPath
                }
                buf.clear();
            }
            return true;
        }
        false
    }

    fn hit_test(&mut self, x: f64, y: f64, r: f64) -> i32 {
        let mut x = x;
        let mut y = y;
        let mut r = r;
        self.trans.trans_mut().inverse_transform(&mut x, &mut y);
        r /= self.trans.trans_mut().scale();
        for i in 0..self.trans.source_mut().source_mut().total_vertices() {
            let (mut vx, mut vy) = (0., 0.);
            let cmd = self
                .trans
                .source_mut()
                .source_mut()
                .vertex(&mut vx, &mut vy);
            if agg::is_vertex(cmd) {
                if agg::calc_distance(x, y, vx, vy) <= r {
                    return i as i32;
                }
            }
        }
        -1
    }
}

struct TestStyles<'a, C: Color> {
    solid_colors: &'a [C],
    gradient: &'a [C],
}

impl<'a, C: Color> TestStyles<'a, C> {
    pub fn new(solid_colors: &'a [C], gradient: &'a [C]) -> Self {
        TestStyles {
            solid_colors,
            gradient,
        }
    }
}

impl<'a, C: Color> RasterStyle<C> for TestStyles<'a, C> {
    // Suppose that style=1 is a gradient
    //---------------------------------------------
    fn is_solid(&self, _style: u32) -> bool {
        true // style != 1
    }

    // Just returns a color
    //---------------------------------------------
    fn color(&self, style: u32) -> &C {
        &self.solid_colors[style as usize]
    }

    // Generate span. In our test case only one style (style=1)
    // can be a span generator, so that, parameter "style"
    // isn't used here.
    //---------------------------------------------
    fn generate_span(&mut self, span: &mut [C], x: i32, _y: i32, len: u32, _style: u32) {
        span.copy_from_slice(&self.gradient[x as usize..(x as u32 + len) as usize]);
    }
}

pub struct Application {
    shape: CompoundShape<'static>,
    colors: [agg::Rgba8; 100],
    scale: agg::TransAffine,
    gamma: agg::GammaLut<u8>,
    gradient: agg::array::PodArray<agg::Rgba8>,
    point_idx: i32,
    hit_x: i32,
    hit_y: i32,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    pub fn open(&mut self, fname: &str) -> bool {
        self.shape.open(fname)
    }

    pub fn read_next(&mut self) {
        self.shape.read_next();
        self.shape
            .set_scale(self.util.borrow().width(), self.util.borrow().height());
    }
}

impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }

    fn new(_format: PixFormat, _flip_y: bool, util: Rc<RefCell<PlatUtil>>) -> Self {
        let mut gamma = agg::GammaLut::new();
        gamma.set_gamma(2.0);

        let mut colors = [agg::Rgba8::default(); 100];
        for i in 0..100 {
            let (b, g, r) = (
                frand() as u32 & 0xFF,
                frand() as u32 & 0xFF,
                frand() as u32 & 0xFF,
            );
            colors[i] = agg::Rgba8::new_params(r, g, b, 230);
            colors[i].apply_gamma_dir(&gamma);
            colors[i].premultiply();
        }

        Self {
            shape: CompoundShape::new(),
            colors,
            scale: agg::TransAffine::new_default(),
            gamma,
            gradient: agg::array::PodArray::new(),
            point_idx: -1,
            hit_x: -1,
            hit_y: -1,
            ctrls: CtrlContainer {
                ctrl: vec![],
                cur_ctrl: -1,
                num_ctrl: 0,
            },
            util: util,
        }
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if (flags & 3) == 0 {
            return self.on_mouse_button_up(rb, x, y, flags);
        } else {
            if self.point_idx >= 0 {
                let mut xd = x as f64;
                let mut yd = y as f64;
                self.scale.inverse_transform(&mut xd, &mut yd);
                self.shape.modify_vertex(self.point_idx as u32, xd, yd);
                return Draw::Yes;
            }
        }
        Draw::No
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if flags & 1 != 0 {
            let mut xd = x as f64;
            let mut yd = y as f64;
            let r = 4.0 / self.scale.scale();
            self.scale.inverse_transform(&mut xd, &mut yd);
            self.point_idx = self.shape.hit_test(xd, yd, r);
            return Draw::Yes;
        }
        if flags & 2 != 0 {
            self.hit_x = x;
            self.hit_y = y;
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.point_idx = -1;
        self.hit_x = -1;
        self.hit_y = -1;
        Draw::Yes
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgr24Pre::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24Pre>::new_borrowed(&mut pixf);
        //let mut sl = agg::ScanlineP8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 240, 255));
        let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);

        let width = self.util.borrow().width();
        self.gradient.resize(width as usize, agg::Rgba8::new());
        let c1 = agg::Rgba8::new_params(255, 0, 0, 180);
        let c2 = agg::Rgba8::new_params(0, 0, 255, 180);
        for i in 0..width as usize {
            self.gradient[i] = c1.gradient(&c2, i as f64 / width as f64);
            self.gradient[i].premultiply();
        }

        let mut ras_clip_dbl: agg::RasterizerScanlineAa<agg::RasterizerSlClipDbl> =
            agg::RasterizerScanlineAa::new();
        let mut rasc_clip_dbl: agg::RasterizerCompoundAa<agg::RasterizerSlClipDbl> =
            agg::RasterizerCompoundAa::new();
        let mut sl = agg::ScanlineU8::new();
        let mut sl_bin = agg::ScanlineBin::new();
        let lscale = self.scale;
        let sc = self.scale.scale();
        self.shape.set_approximation_scale(sc);

        let mut shape = agg::ConvTransform::new_borrowed(&mut self.shape, lscale);

        // Fill shape
        //----------------------
        rasc_clip_dbl.clip_box(0., 0., width, self.util.borrow().height());
        rasc_clip_dbl.reset();
        //rasc.filling_rule(agg::fill_even_odd);
        self.util.borrow_mut().start_timer();
        let plen = shape.source().paths();
        for i in 0..plen {
            if shape.source().style(i).left_fill >= 0 || shape.source().style(i).right_fill >= 0 {
                rasc_clip_dbl.set_styles(
                    shape.source().style(i).left_fill,
                    shape.source().style(i).right_fill,
                );
                let id = shape.source().style(i).path_id;
                rasc_clip_dbl.add_path(&mut shape, id);
            }
        }

        let mut style_handler = TestStyles::new(&self.colors, &self.gradient);
        let mut alloc = agg::VecSpan::new();
        agg::render_scanlines_compound(
            &mut rasc_clip_dbl,
            &mut sl,
            &mut sl_bin,
            ren.ren_mut(),
            &mut alloc,
            &mut style_handler,
        );

        let tfill = self.util.borrow_mut().elapsed_time();

        // Hit-test test
        let mut draw_strokes = true;
        if self.hit_x >= 0 && self.hit_y >= 0 {
            if rasc_clip_dbl.hit_test(self.hit_x, self.hit_y) {
                draw_strokes = false;
            }
        }

        // Draw strokes
        //----------------------
        self.util.borrow_mut().start_timer();
        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(shape);
        if draw_strokes {
            ras_clip_dbl.clip_box(0., 0., width, self.util.borrow().height());
            stroke.set_width(f64::sqrt(self.scale.scale()));
            stroke.set_line_join(agg::LineJoin::Round);
            stroke.set_line_cap(agg::LineCap::Round);
            let plen = stroke.source().source().paths();
            for i in 0..plen {
                ras_clip_dbl.reset();
                let l = stroke.source().source().style(i).line;
                if l >= 0 {
                    let id = stroke.source().source().style(i).path_id;
                    ras_clip_dbl.add_path(&mut stroke, id);
                    ren.set_color(agg::Rgba8::new_params(0, 0, 0, 128));
                    agg::render_scanlines(&mut ras_clip_dbl, &mut sl, &mut ren);
                }
            }
        }
        let tstroke = self.util.borrow_mut().elapsed_time();

        let mut t = agg::GsvText::new();
        t.set_size(8.0, 0.);
        t.set_flip(true);
        let buf = format!(
            "Fill={:.2}ms ({}FPS) Stroke={:.2}ms ({}FPS) Total={:.2}ms ({}FPS)
		
			Space: Next Shape
			
		+/- : ZoomIn/ZoomOut (with respect to the mouse pointer)",
            tfill,
            (1000.0 / tfill) as i32,
            tstroke,
            (1000.0 / tstroke) as i32,
            tfill + tstroke,
            (1000.0 / (tfill + tstroke)) as i32
        );
        t.set_start_point(10.0, 20.0);
        t.set_text(&buf);

        let mut ts: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(t);
        ts.set_width(1.6);
        ts.set_line_cap(agg::LineCap::Round);

        ras.add_path(&mut ts, 0);
        ren.set_color(agg::Rgba8::new_params(0, 0, 0, 255));
        agg::render_scanlines(&mut ras, &mut sl, &mut ren);

        if self.gamma.gamma() != 1.0 {
            pixf.apply_gamma_inv(&self.gamma);
        }
    }

    fn on_key(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, key: u32, _flags: u32) -> Draw {
        let (x, y) = (x as f64, y as f64);
        if key == ' ' as u32 {
            self.shape.read_next();
            self.shape
                .set_scale(self.util.borrow().width(), self.util.borrow().height());
            return Draw::Yes;
        }

        if key == 'p' as u32 || key == KeyCode::KpPlus as u32 {
            self.scale *= agg::TransAffine::trans_affine_translation(-x, -y);
            self.scale *= agg::TransAffine::trans_affine_scaling_eq(1.1);
            self.scale *= agg::TransAffine::trans_affine_translation(x, y);
            return Draw::Yes;
        }

        if key == 'm' as u32 || key == KeyCode::KpMinus as u32 {
            self.scale *= agg::TransAffine::trans_affine_translation(-x, -y);
            self.scale *= agg::TransAffine::trans_affine_scaling_eq(1.0 / 1.1);
            self.scale *= agg::TransAffine::trans_affine_translation(x, y);
            return Draw::Yes;
        }

        if key == 'l' as u32 {
            self.scale *= agg::TransAffine::trans_affine_translation(-x, -y);
            self.scale *= agg::TransAffine::trans_affine_rotation(-PI / 20.0);
            self.scale *= agg::TransAffine::trans_affine_translation(x, y);
            return Draw::Yes;
        }

        if key == 'r' as u32 {
            self.scale *= agg::TransAffine::trans_affine_translation(-x, -y);
            self.scale *= agg::TransAffine::trans_affine_rotation(PI / 20.0);
            self.scale *= agg::TransAffine::trans_affine_translation(x, y);
            return Draw::Yes;
        }
        Draw::No
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut fname = "shapes.txt";
    if args.len() > 1 {
        fname = &args[1];
    }

    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgr24, FLIP_Y);

    let buf;
    if !plat.app_mut().open(fname) {
        if fname.eq("shapes.txt") {
            buf = format!(
                "File not found: {}. Download http://www.antigrain.com/{}
				or copy it from another directory if available.",
                fname, fname
            );
        } else {
            buf = format!("File not found: {}", fname);
        }
        plat.app_mut().util.borrow_mut().message(&buf);
    }

    plat.set_caption("AGG Example - Flash Rasterizer");

    if plat.init(655, 520, WindowFlag::Resize as u32) {
        plat.app_mut().read_next();
        plat.run();
    }
}

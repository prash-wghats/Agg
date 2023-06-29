use crate::platform::*;
use agg::rendering_buffer::RenderBuf;
use agg::{
    Color, ConvCurve, Gamma, PathStorage, RasterScanLine, RendererScanlineColor, VertexSource,
};

mod ctrl;
mod platform;

use libc::*;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
    //path: agg::PathStorage,
    //affine: agg::TransAffine,
    trans: agg::ConvTransform<'a, ConvCurve<'a, PathStorage>, agg::TransAffine>,
    styles: Vec<PathStyle>,
    _x1: f64,
    _y1: f64,
    _x2: f64,
    _y2: f64,
    min_style: i32,
    max_style: i32,
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

impl<'a> CompoundShape<'a> {
    fn new() -> Self {
        let p = agg::PathStorage::new();
        let mtx = agg::TransAffine::new_default();
        let cc = agg::ConvCurve::new_owned(p);
        let tr = agg::ConvTransform::new_owned(cc, mtx);
        CompoundShape {
            //m_path: path_storage::new(),
            //m_affine: trans_affine::identity(),
            //m_curve: conv_curve::new(m_path),
            trans: tr,
            styles: Vec::new(),
            _x1: 0.0,
            _y1: 0.0,
            _x2: 0.0,
            _y2: 0.0,
            min_style: std::i32::MAX,
            max_style: std::i32::MIN,
            buf_rdr: None,
        }
    }

    fn min_style(&self) -> i32 {
        self.min_style
    }

    fn max_style(&self) -> i32 {
        self.max_style
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
                    if style.left_fill >= 0 {
                        if style.left_fill < self.min_style {
                            self.min_style = style.left_fill;
                        }
                        if style.left_fill > self.max_style {
                            self.max_style = style.left_fill;
                        }
                    }
                    if style.right_fill >= 0 {
                        if style.right_fill < self.min_style {
                            self.min_style = style.right_fill;
                        }
                        if style.right_fill > self.max_style {
                            self.max_style = style.right_fill;
                        }
                    }
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

pub struct Application {
    shape: CompoundShape<'static>,
    colors: [agg::Rgba8; 100],
    scale: agg::TransAffine,
    gamma: agg::GammaLut<u8>,

    point_idx: i32,

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

            point_idx: -1,

            ctrls: CtrlContainer {
                ctrl: vec![],
                cur_ctrl: -1,
                num_ctrl: 0,
            },
            util: util,
        }
    }

    fn on_mouse_move(&mut self, rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if (flags & 1) == 0 {
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

        Draw::No
    }

    fn on_mouse_button_up(
        &mut self, _rb: &mut agg::RenderBuf, _x: i32, _y: i32, _flags: u32,
    ) -> Draw {
        self.point_idx = -1;
        Draw::No
    }

    fn on_draw(&mut self, rbuf: &mut agg::RenderBuf) {
        let mut pixf = agg::PixBgr24Pre::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::<agg::PixBgr24Pre>::new_borrowed(&mut pixf);
        //let mut sl = agg::ScanlineP8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
        ren_base.clear(&agg::Rgba8::new_params(255, 255, 240, 255));
        let mut ren = agg::RendererScanlineAASolid::new_borrowed(&mut ren_base);

        let width = self.util.borrow().width();
        let height = self.util.borrow().height();
        let mut ras_clip_dbl: agg::RasterizerScanlineAa<agg::RasterizerSlClipDbl> =
            agg::RasterizerScanlineAa::new();

        let mut sl = agg::ScanlineU8::new();
        let lscale = self.scale;
        let sc = self.scale.scale();
        self.shape.set_approximation_scale(sc);

        let mut shape = agg::ConvTransform::new_borrowed(&mut self.shape, lscale);

        let mut tmp_path = agg::PathStorage::new();

        ras.clip_box(0., 0., width, height);
        // This is an alternative method of Flash rasterization.
        // We decompose the compound shape into separate paths
        // and select the ones that fit the given style (left or right).
        // So that, we form a sub-shape and draw it as a whole.
        //
        // Here the regular scanline rasterizer is used, but it doesn't
        // automatically close the polygons. So that, the rasterizer
        // actually works with a set of polylines instead of polygons.
        // Of course, the data integrity must be preserved, that is,
        // the polylines must eventually form a closed contour
        // (or a set of closed contours). So that, first we set
        // auto_close(false);
        //
        // The second important thing is that one path can be rasterized
        // twice, if it has both, left and right fill. Sometimes the
        // path has equal left and right fill, so that, the same path
        // will be added twice even for a single sub-shape. If the
        // rasterizer can tolerate these degenerates you can add them,
        // but it's also fine just to omit them.
        //
        // The third thing is that for one side (left or right)
        // you should invert the direction of the paths.
        //
        // The main disadvantage of this method is imperfect stitching
        // of the adjacent polygons. The problem can be solved if we use
        // compositing operation "plus" instead of alpha-blend. But
        // in this case we are forced to use an RGBA buffer, clean it with
        // zero, rasterize using "plus" operation, and then alpha-blend
        // the result over the final scene. It can be too expensive.
        ras.set_auto_close(false);
        self.util.borrow_mut().start_timer();
        let min = shape.source().min_style();
        let max = shape.source().max_style();
        for s in min..=max {
            ras.reset();
            let plen = shape.source().paths();
            for i in 0..plen {
                let left_fill = shape.source().style(i).left_fill;
                let right_fill = shape.source().style(i).right_fill;
                let id = shape.source().style(i).path_id;
                if left_fill != right_fill {
                    if left_fill == s {
                        ras.add_path(&mut shape, id);
                    }
                    if right_fill == s {
                        tmp_path.remove_all();
                        tmp_path.concat_path(&mut shape, id);
                        tmp_path.invert_polygon(0);
                        ras.add_path(&mut tmp_path, 0);
                    }
                }
            }
            agg::render_scanlines_aa_solid(
                &mut ras,
                &mut sl,
                ren.ren_mut(),
                &self.colors[s as usize],
            );
        }
        let tfill = self.util.borrow_mut().elapsed_time();
        ras.set_auto_close(true);

        // Draw strokes
        //----------------------
        self.util.borrow_mut().start_timer();
        let mut stroke: agg::ConvStroke<'_, _> = agg::ConvStroke::new_owned(shape);

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

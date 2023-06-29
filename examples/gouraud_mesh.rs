use crate::platform::*;

use agg::color_rgba::*;
use agg::conv_stroke::*;

use agg::gamma_lut::GammaLut;
use agg::gsv_text::*;
use agg::math::calc_distance;
use agg::math_stroke::{LineCap, LineJoin};

use agg::rasterizer_compound_aa::RasterizerCompoundAa;
use agg::renderer_scanline::{render_scanlines_aa_solid, render_scanlines_compound};
use agg::rendering_buffer::RenderBuf;
use agg::scanline_bin::ScanlineBin;
use agg::span_allocator::VecSpan;
use agg::span_gouraud_rgba::SpanGouraudRgba;

use agg::{Gamma, RasterScanLine, RasterStyle, SpanGenerator};

mod ctrl;
mod platform;

use libc::*;
use std::cell::RefCell;
use std::rc::Rc;
type Ptr<T> = Rc<RefCell<T>>;

const FLIP_Y: bool = true;

struct MeshPoint {
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    color: Rgba8,
    dc: Rgba8,
}

impl MeshPoint {
    fn new(x_: f64, y_: f64, dx_: f64, dy_: f64, c: Rgba8, dc_: Rgba8) -> Self {
        Self {
            x: x_,
            y: y_,
            dx: dx_,
            dy: dy_,
            color: c,
            dc: dc_,
        }
    }
}

struct MeshTriangle {
    p1: u32,
    p2: u32,
    p3: u32,
}

impl MeshTriangle {
    fn new(i: u32, j: u32, k: u32) -> Self {
        Self {
            p1: i,
            p2: j,
            p3: k,
        }
    }
}

struct MeshEdge {
    p1: u32,
    p2: u32,
    tl: i32,
    tr: i32,
}

impl MeshEdge {
    fn new(p1_: u32, p2_: u32, tl_: i32, tr_: i32) -> Self {
        Self {
            p1: p1_,
            p2: p2_,
            tl: tl_,
            tr: tr_,
        }
    }
}

fn frand() -> i32 {
    unsafe { rand() }
}

fn random(v1: f64, v2: f64) -> f64 {
    //(v2 - v1) * (frand() % 1000.0) / 999.0 + v1
    (v2 - v1) * (frand() as f64 % 1000.0) / 999.0 + v1
}

struct MeshCtrl {
    cols: u32,
    rows: u32,
    drag_idx: i32,
    drag_dx: f64,
    drag_dy: f64,
    cell_w: f64,
    cell_h: f64,
    start_x: f64,
    start_y: f64,
    vertices: Vec<MeshPoint>,
    triangles: Vec<MeshTriangle>,
    edges: Vec<MeshEdge>,
}

#[allow(dead_code)]
impl MeshCtrl {
    fn new() -> Self {
        Self {
            cols: 0,
            rows: 0,
            drag_idx: -1,
            drag_dx: 0.0,
            drag_dy: 0.0,
            cell_w: 0.0,
            cell_h: 0.0,
            start_x: 0.0,
            start_y: 0.0,
            vertices: Vec::new(),
            triangles: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn vertex(&self, i: usize) -> &MeshPoint {
        &self.vertices[i]
    }

    fn vertex_mut(&mut self, i: usize) -> &mut MeshPoint {
        &mut self.vertices[i]
    }

    fn vertex_xy(&self, x: u32, y: u32) -> &MeshPoint {
        &self.vertices[(y * self.rows + x) as usize]
    }

    fn vertex_xy_mut(&mut self, x: u32, y: u32) -> &mut MeshPoint {
        &mut self.vertices[(y * self.rows + x) as usize]
    }

    fn num_triangles(&self) -> usize {
        self.triangles.len()
    }

    fn triangle(&self, i: usize) -> &MeshTriangle {
        &self.triangles[i]
    }

    fn triangle_mut(&mut self, i: usize) -> &mut MeshTriangle {
        &mut self.triangles[i]
    }

    fn num_edges(&self) -> usize {
        self.edges.len()
    }

    fn edge(&self, i: usize) -> &MeshEdge {
        &self.edges[i]
    }

    fn edge_mut(&mut self, i: usize) -> &mut MeshEdge {
        &mut self.edges[i]
    }

    fn randomize_points(&mut self, _delta: f64) {
        for i in 0..self.rows {
            for j in 0..self.cols {
                let xc = j as f64 * self.cell_w + self.start_x;
                let yc = i as f64 * self.cell_h + self.start_y;
                let x1 = xc - self.cell_w / 4.0;
                let y1 = yc - self.cell_h / 4.0;
                let x2 = xc + self.cell_w / 4.0;
                let y2 = yc + self.cell_h / 4.0;
                let mut p = self.vertex_xy_mut(j, i);
                p.x += p.dx;
                p.y += p.dy;
                if p.x < x1 {
                    p.x = x1;
                    p.dx = -p.dx;
                }
                if p.y < y1 {
                    p.y = y1;
                    p.dy = -p.dy;
                }
                if p.x > x2 {
                    p.x = x2;
                    p.dx = -p.dx;
                }
                if p.y > y2 {
                    p.y = y2;
                    p.dy = -p.dy;
                }
            }
        }
    }

    fn rotate_colors(&mut self) {
        for i in 1..self.vertices.len() {
            let mesh = &mut self.vertices[i];
            let mut c = mesh.color;
            let mut dc = mesh.dc;
            let mut r = c.r as i32 + if dc.r == 0 { 5 as i32 } else { -5 };
            let mut g = c.g as i32 + if dc.g == 0 { 5 as i32 } else { -5 };
            let mut b = c.b as i32 + if dc.b == 0 { 5 as i32 } else { -5 };
            if r < 0 {
                r = 0;
                dc.r ^= 1;
            }
            if r > 255 {
                r = 255;
                dc.r ^= 1;
            }
            if g < 0 {
                g = 0;
                dc.g ^= 1;
            }
            if g > 255 {
                g = 255;
                dc.g ^= 1;
            }
            if b < 0 {
                b = 0;
                dc.b ^= 1;
            }
            if b > 255 {
                b = 255;
                dc.b ^= 1;
            }
            c.r = r as u8;
            c.g = g as u8;
            c.b = b as u8;
        }
    }

    fn on_mouse_button_down(&mut self, x: f64, y: f64, flags: u32) -> bool {
        if flags & 1 != 0 {
            for i in 0..self.vertices.len() {
                if calc_distance(x, y, self.vertices[i].x, self.vertices[i].y) < 5.0 {
                    self.drag_idx = i as i32;
                    self.drag_dx = x - self.vertices[i].x;
                    self.drag_dy = y - self.vertices[i].y;
                    return true;
                }
            }
        }
        false
    }

    fn on_mouse_move(&mut self, x: f64, y: f64, flags: u32) -> bool {
        if flags & 1 != 0 {
            if self.drag_idx >= 0 {
                self.vertices[self.drag_idx as usize].x = x - self.drag_dx;
                self.vertices[self.drag_idx as usize].y = y - self.drag_dy;
                return true;
            }
        } else {
            self.on_mouse_button_up(x, y, flags);
        }
        false
    }

    fn on_mouse_button_up(&mut self, _x: f64, _y: f64, _flags: u32) -> bool {
        let ret = self.drag_idx >= 0;
        self.drag_idx = -1;
        ret
    }

    fn generate(
        &mut self, cols: u32, rows: u32, cell_w: f64, cell_h: f64, start_x: f64, start_y: f64,
    ) {
        self.cols = cols;
        self.rows = rows;
        self.cell_w = cell_w;
        self.cell_h = cell_h;
        self.start_x = start_x;
        self.start_y = start_y;

        let mut start_y = start_y;
        self.vertices.clear();
        for _i in 0..self.rows {
            let mut x = start_x;
            for _j in 0..self.cols {
                let dx = random(-0.5, 0.5);
                let dy = random(-0.5, 0.5);
                let c = Rgba8::new_params(
                    //frand() & 0xFF,
                    frand() as u32 & 0xFF,
                    frand() as u32 & 0xFF,
                    frand() as u32 & 0xFF,
                    255,
                );
                let dc = Rgba8::new_params(
                    frand() as u32 & 1,
                    frand() as u32 & 1,
                    frand() as u32 & 1,
                    255,
                );
                self.vertices
                    .push(MeshPoint::new(x, start_y, dx, dy, c, dc));
                x += cell_w;
            }
            start_y += cell_h;
        }

        //  4---3
        //  |t2/|
        //  | / |
        //  |/t1|
        //  1---2
        self.triangles.clear();
        self.edges.clear();
        for i in 0..self.rows - 1 {
            for j in 0..self.cols - 1 {
                let p1 = i * self.cols + j;
                let p2 = p1 + 1;
                let p3 = p2 + self.cols;
                let p4 = p1 + self.cols;
                self.triangles.push(MeshTriangle::new(p1, p2, p3));
                self.triangles.push(MeshTriangle::new(p3, p4, p1));

                let curr_cell = i * (self.cols - 1) + j;
                let left_cell = if j > 0 { curr_cell as i32 - 1 } else { -1 };
                let bott_cell = if i > 0 {
                    (curr_cell - (self.cols - 1)) as i32
                } else {
                    -1
                };

                let curr_t1 = curr_cell as i32 * 2;
                let curr_t2 = curr_t1 + 1;

                let left_t1 = if left_cell >= 0 { left_cell * 2 } else { -1 };
                let _left_t2 = if left_cell >= 0 { left_t1 + 1 } else { -1 };

                let bott_t1 = if bott_cell >= 0 { bott_cell * 2 } else { -1 };
                let bott_t2 = if bott_cell >= 0 { bott_t1 + 1 } else { -1 };

                self.edges.push(MeshEdge::new(p1, p2, curr_t1, bott_t2));
                self.edges.push(MeshEdge::new(p1, p3, curr_t2, curr_t1));
                self.edges.push(MeshEdge::new(p1, p4, left_t1, curr_t2));

                if j == self.cols - 2 {
                    // Last column
                    self.edges.push(MeshEdge::new(p2, p3, curr_t1, -1));
                }

                if i == self.rows - 2 {
                    // Last row
                    self.edges.push(MeshEdge::new(p3, p4, curr_t2, -1));
                }
            }
        }
    }
}

struct StylesGouraud {
    gouraud_type: Vec<SpanGouraudRgba<Rgba8>>,
    c: Rgba8,
}

impl StylesGouraud {
    fn new<Ga: Gamma<u8, u8>>(mesh: &MeshCtrl, gamma: &Ga) -> Self {
        let mut triangles = Vec::new();
        for i in 0..mesh.num_triangles() {
            let t = mesh.triangle(i);
            let p1 = mesh.vertex(t.p1 as usize);
            let p2 = mesh.vertex(t.p2 as usize);
            let p3 = mesh.vertex(t.p3 as usize);

            let mut c1 = p1.color;
            let mut c2 = p2.color;
            let mut c3 = p3.color;
            c1.apply_gamma_dir(gamma);
            c2.apply_gamma_dir(gamma);
            c3.apply_gamma_dir(gamma);
            let mut gouraud =
                SpanGouraudRgba::new(c1, c2, c3, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, 0.);
            gouraud.prepare();
            triangles.push(gouraud);
        }
        Self {
            gouraud_type: triangles,
            c: Rgba8::new_params(0, 0, 0, 0),
        }
    }
}

impl RasterStyle<Rgba8> for StylesGouraud {
    fn is_solid(&self, _style: u32) -> bool {
        false
    }

    fn color(&self, _style: u32) -> &Rgba8 {
        &self.c
    }

    fn generate_span(&mut self, span: &mut [Rgba8], x: i32, y: i32, len: u32, style: u32) {
        self.gouraud_type[style as usize].generate(span, x, y, len);
    }
}

struct Application {
    mesh_ctrl: MeshCtrl,
    gamma: GammaLut,
    ctrls: CtrlContainer,
    util: Rc<RefCell<PlatUtil>>,
}

impl Interface for Application {
    fn new(_format: PixFormat, _flip_y: bool, util: Ptr<PlatUtil>) -> Application {
        Application {
            mesh_ctrl: MeshCtrl::new(),
            gamma: GammaLut::new(),
            ctrls: CtrlContainer {
                ctrl: vec![],
                cur_ctrl: -1,
                num_ctrl: 0,
            },
            util: util,
        }
    }

    fn on_init(&mut self) {
        self.util.borrow_mut().set_wait_mode(false);
        self.mesh_ctrl.generate(20, 20, 17., 17., 40., 40.);
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut pix = agg::PixBgra32Pre::new_borrowed(rbuf);
        let mut ren_base = agg::RendererBase::<agg::PixBgra32Pre>::new_borrowed(&mut pix);
        ren_base.clear(&agg::Rgba8::new_params(0, 0, 0, 255));

        let mut sl = agg::ScanlineU8::new();
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let mut sl_bin = ScanlineBin::new();

        let mut rasc: RasterizerCompoundAa = RasterizerCompoundAa::new();
        let mut alloc = VecSpan::<agg::Rgba8>::new();

        let mut styles = StylesGouraud::new(&self.mesh_ctrl, &self.gamma);
        self.util.borrow_mut().start_timer();
        rasc.reset();
        //rasc.clip_box(40, 40, width() - 40, height() - 40);
        for i in 0..self.mesh_ctrl.num_edges() {
            let e = self.mesh_ctrl.edge(i);
            let p1 = self.mesh_ctrl.vertex(e.p1 as usize);
            let p2 = self.mesh_ctrl.vertex(e.p2 as usize);
            rasc.set_styles(e.tl, e.tr);
            rasc.move_to_d(p1.x, p1.y);
            rasc.line_to_d(p2.x, p2.y);
        }
        render_scanlines_compound(
            &mut rasc,
            &mut sl,
            &mut sl_bin,
            &mut ren_base,
            &mut alloc,
            &mut styles,
        );
        let tm = self.util.borrow_mut().elapsed_time();

        let mut t = GsvText::new();
        t.set_size(10.0, 0.);

        let mut pt: agg::ConvStroke<'_, _> = ConvStroke::new_owned(t);
        pt.set_width(1.5);
        pt.set_line_cap(LineCap::Round);
        pt.set_line_join(LineJoin::Round);

        let buf = format!(
            "{:3.2} ms, {} triangles, {:.0} tri/sec",
            tm,
            self.mesh_ctrl.num_triangles(),
            self.mesh_ctrl.num_triangles() as f64 / tm * 1000.0
        );
        pt.source_mut().set_start_point(10.0, 10.0);
        pt.source_mut().set_text(&buf);

        ras.add_path(&mut pt, 0);
        render_scanlines_aa_solid(
            &mut ras,
            &mut sl,
            &mut ren_base,
            &Rgba8::new_params(255, 255, 255, 255),
        );

        if self.gamma.gamma() != 1.0 {
            pix.apply_gamma_inv(&self.gamma);
        }
    }

    fn on_mouse_move(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if self.mesh_ctrl.on_mouse_move(x as f64, y as f64, flags) {
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_button_down(&mut self, _rb: &mut RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if self
            .mesh_ctrl
            .on_mouse_button_down(x as f64, y as f64, flags)
        {
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_mouse_button_up(&mut self, _rb: &mut agg::RenderBuf, x: i32, y: i32, flags: u32) -> Draw {
        if self.mesh_ctrl.on_mouse_button_up(x as f64, y as f64, flags) {
            return Draw::Yes;
        }
        Draw::No
    }

    fn on_idle(&mut self) -> Draw {
        self.mesh_ctrl.randomize_points(1.0);
        self.mesh_ctrl.rotate_colors();
        Draw::Yes
    }

    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.ctrls
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PixFormat::Bgra32, FLIP_Y);
    //plat.app_mut().init();
    plat.set_caption("AGG Example. Gouraud Mesh");

    if plat.init(400, 400, WindowFlag::Resize as u32) {
        plat.run();
    }
}

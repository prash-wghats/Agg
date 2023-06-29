use crate::ctrl::cbox::Cbox;
use crate::ctrl::rbox::Rbox;
use crate::ctrl::slider::Slider;
use crate::platform::*;

use agg::basics::PathCmd;
use agg::rendering_buffer::RenderBuf;
use agg::{
    Color, ColorFn, CurveBase, CurveType4, RasterScanLine, RenderPrim, RendererScanlineColor,
    VertexSource,
};
mod ctrl;
mod platform;

use libc::*;
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

mod misc;
use misc::pixel_formats::*;

type Ptr<T> = Rc<RefCell<T>>;
fn ctrl_ptr<T>(t: T) -> Ptr<T> {
    Rc::new(RefCell::new(t))
}

fn frand() -> i32 {
    unsafe { rand() }
}

const FLIP_Y: bool = true;
const RAND_MAX: f64 = 0x7fff as f64;
#[derive(Clone, Copy, PartialEq)]
struct Node {
    x: f64,
    y: f64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct Edge {
    node1: i32,
    node2: i32,
}

struct Graph {
    num_nodes: i32,
    num_edges: i32,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    fn new(num_nodes: i32, num_edges: i32) -> Graph {
        let mut nodes = Vec::with_capacity(num_nodes as usize);
        let mut edges = Vec::with_capacity(num_edges as usize);

        unsafe {
            srand(100);
        }

        for _ in 0..num_nodes {
            nodes.push(Node {
                x: (frand() as f64 / RAND_MAX) * 0.75 + 0.2,
                y: (frand() as f64 / RAND_MAX) * 0.85 + 0.1,
            });
        }
        let mut i = 0;
        while i < num_edges {
            edges.push(Edge {
                node1: frand() % num_nodes,
                node2: frand() % num_nodes,
            });
            if edges[i as usize].node1 == edges[i as usize].node2 {
                i -= 1;
            }
            i += 1;
        }

        Graph {
            num_nodes,
            num_edges,
            nodes,
            edges,
        }
    }

    fn get_num_nodes(&self) -> i32 {
        self.num_nodes
    }

    fn get_num_edges(&self) -> i32 {
        self.num_edges
    }

    fn get_node(&self, idx: i32, w: f64, h: f64) -> Node {
        let mut node = Node { x: 0., y: 0. };
        if idx < self.num_nodes {
            node = self.nodes[idx as usize];

            node.x = node.x * w;
            node.y = node.y * h;
        }
        node
    }

    fn get_edge(&self, idx: i32) -> Edge {
        let mut edge = Edge { node1: 0, node2: 0 };
        if idx < self.num_edges {
            edge = self.edges[idx as usize]
        }
        edge
    }
}

struct Line {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    f: i32,
}

impl Line {
    fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Line {
        Line {
            x1,
            y1,
            x2,
            y2,
            f: 0,
        }
    }
}
impl VertexSource for Line {
    fn rewind(&mut self, _: u32) {
        self.f = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.f == 0 {
            *x = self.x1;
            *y = self.y1;
            self.f += 1;
            return PathCmd::MoveTo as u32;
        } else if self.f == 1 {
            *x = self.x2;
            *y = self.y2;
            self.f += 1;
            return PathCmd::LineTo as u32;
        }
        PathCmd::Stop as u32
    }
}

struct Curve {
    c: agg::Curve4,
}

impl Curve {
    fn new(x1: f64, y1: f64, x2: f64, y2: f64, k: f64) -> Curve {
        let mut c = agg::Curve4::new();
        c.init(
            x1,
            y1,
            x1 - (y2 - y1) * k,
            y1 + (x2 - x1) * k,
            x2 + (y2 - y1) * k,
            y2 - (x2 - x1) * k,
            x2,
            y2,
        );
        Curve { c }
    }
}
impl VertexSource for Curve {
    fn rewind(&mut self, path_id: u32) {
        self.c.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.c.vertex(x, y)
    }
}

type StrokeDraft<'a, VS> = StrokeDraftArrow<'a, VS>;
type DashStrokeDraft<'a, VS> = DashStrokeDraftArrow<'a, VS>;

struct StrokeDraftSimple<Source: VertexSource> {
    s: Source,
}
#[allow(dead_code)]
impl<Source: VertexSource> StrokeDraftSimple<Source> {
    fn new(s: Source, _w: f64) -> StrokeDraftSimple<Source> {
        StrokeDraftSimple { s }
    }
}

impl<Source: VertexSource> VertexSource for StrokeDraftSimple<Source> {
    fn rewind(&mut self, path_id: u32) {
        self.s.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.s.vertex(x, y)
    }
}

struct StrokeDraftArrow<'a, Source: VertexSource> {
    c: agg::ConvMarkerConcat<
        'a,
        agg::ConvMarkerAdaptor<'a, Source, agg::VcgenMarkersTerm>,
        agg::Arrowhead,
    >,
}

impl<'a, Source: VertexSource> StrokeDraftArrow<'a, Source> {
    fn new(s: Source, _w: f64) -> Self {
        let mut s = agg::ConvMarkerAdaptor::new_owned(s);
        let mut ah = agg::Arrowhead::new();
        ah.head(0., 10., 5., 0.);
        s.set_shorten(10.0);
        let c = agg::ConvMarkerConcat::new_owned(s, ah);
        StrokeDraftArrow { c }
    }
}

impl<'a, Source: VertexSource> VertexSource for StrokeDraftArrow<'a, Source> {
    fn rewind(&mut self, path_id: u32) {
        self.c.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.c.vertex(x, y)
    }
}

struct StrokeFineSimple<'a, Source: VertexSource> {
    s: agg::ConvStroke<'a, Source>,
}

#[allow(dead_code)]
impl<'a, Source: VertexSource> StrokeFineSimple<'a, Source> {
    fn new(s: Source, w: f64) -> Self {
        let mut s = agg::ConvStroke::new_owned(s);
        s.set_width(w);
        StrokeFineSimple { s }
    }
}

impl<'a, Source: VertexSource> VertexSource for StrokeFineSimple<'a, Source> {
    fn rewind(&mut self, path_id: u32) {
        self.s.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.s.vertex(x, y)
    }
}

struct StrokeFineArrow<'a, Source: VertexSource> {
    c: agg::ConvMarkerConcat<
        'a,
        agg::ConvStroke<'a, Source, agg::VcgenMarkersTerm>,
        agg::Arrowhead,
    >,
}

impl<'a, Source: VertexSource> StrokeFineArrow<'a, Source> {
    fn new(src: Source, w: f64) -> Self {
        let mut s = agg::ConvStroke::new_owned(src);
        let mut ah = agg::Arrowhead::new();
        ah.head(0., 10., 5., 0.);
        s.set_width(w);
        s.set_shorten(w * 2.0);
        //let m = agg::ConvMarker::new(s.markers_mut(), ah);
        let c = agg::ConvMarkerConcat::new_owned(s, ah);
        StrokeFineArrow { c }
    }
}

impl<'a, Source: VertexSource> VertexSource for StrokeFineArrow<'a, Source> {
    fn rewind(&mut self, path_id: u32) {
        self.c.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.c.vertex(x, y)
    }
}

struct DashStrokeDraftSimple<'a, Source: VertexSource> {
    dash_type: agg::ConvDash<'a, Source, agg::VcgenMarkersTerm>,
}

#[allow(dead_code)]
impl<'a, Source: VertexSource> DashStrokeDraftSimple<'a, Source> {
    fn new(src: Source, dash_len: f64, gap_len: f64, _w: f64) -> Self {
        let mut d = Self {
            dash_type: agg::ConvDash::new_owned(src),
        };
        d.dash_type.add_dash(dash_len, gap_len);
        d
    }
}

impl<'a, Source: VertexSource> VertexSource for DashStrokeDraftSimple<'a, Source> {
    fn rewind(&mut self, path_id: u32) {
        self.dash_type.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.dash_type.vertex(x, y)
    }
}

struct DashStrokeDraftArrow<'a, Source: VertexSource> {
    c: agg::ConvMarkerConcat<'a, agg::ConvDash<'a, Source, agg::VcgenMarkersTerm>, agg::Arrowhead>,
}
impl<'a, Source: VertexSource> DashStrokeDraftArrow<'a, Source> {
    fn new(src: Source, dash_len: f64, gap_len: f64, _w: f64) -> Self {
        let mut d = agg::ConvDash::new_owned(src);
        let mut ah = agg::Arrowhead::new();
        ah.head(0., 10., 5., 0.);
        d.add_dash(dash_len, gap_len);
        d.set_shorten(10.0);
        let c = agg::ConvMarkerConcat::new_owned(d, ah);
        DashStrokeDraftArrow { c }
    }
}

impl<'a, Source: VertexSource> VertexSource for DashStrokeDraftArrow<'a, Source> {
    fn rewind(&mut self, path_id: u32) {
        self.c.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.c.vertex(x, y)
    }
}

struct DashStrokeFineSimple<'a, Source: VertexSource> {
    stroke_type: agg::ConvStroke<'a, agg::ConvDash<'a, Source>>,
}

#[allow(dead_code)]
impl<'a, Source: VertexSource> DashStrokeFineSimple<'a, Source> {
    fn new(src: Source, dash_len: f64, gap_len: f64, w: f64) -> Self {
        let mut d = agg::ConvDash::new_owned(src);
        d.add_dash(dash_len, gap_len);
        let mut s = Self {
            stroke_type: agg::ConvStroke::new_owned(d),
        };
        s.stroke_type.set_width(w);
        s
    }
}

impl<'a, Source: VertexSource> VertexSource for DashStrokeFineSimple<'a, Source> {
    fn rewind(&mut self, path_id: u32) {
        self.stroke_type.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.stroke_type.vertex(x, y)
    }
}

struct DashStrokeFineArrow<'a, Source: VertexSource> {
    c: agg::ConvMarkerConcat<
        'a,
        agg::ConvStroke<'a, agg::ConvDash<'a, Source, agg::VcgenMarkersTerm>>,
        agg::Arrowhead,
    >,
}

impl<'a, Source: VertexSource> DashStrokeFineArrow<'a, Source> {
    fn new(src: Source, dash_len: f64, gap_len: f64, w: f64) -> Self {
        let mut s = agg::ConvStroke::new_owned(agg::ConvDash::new_owned(src));
        s.set_width(w);
        s.source_mut().add_dash(dash_len, gap_len);
        s.source_mut().set_shorten(w * 2.0);

        let mut ah = agg::Arrowhead::new();
        ah.head(0., 10., 5., 0.);
        let c = agg::ConvMarkerConcat::new_owned(s, ah);
        Self { c }
    }
}

impl<'a, Source: VertexSource> VertexSource for DashStrokeFineArrow<'a, Source> {
    fn rewind(&mut self, path_id: u32) {
        self.c.rewind(path_id);
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.c.vertex(x, y)
    }
}

struct ColorFunc<C: Color>([C; 256]);
impl<C: Color> ColorFunc<C> {
    pub fn new() -> Self {
        Self([C::new(); 256])
    }
}

impl<C: Color> ColorFn<C> for ColorFunc<C> {
    fn size(&self) -> u32 {
        self.0.len() as u32
    }
    fn get(&mut self, i: u32) -> C {
        self.0[i as usize]
    }
}

impl<C: Color> Index<usize> for ColorFunc<C> {
    type Output = C;
    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl<C: Color> IndexMut<usize> for ColorFunc<C> {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

struct Application {
    m_type: Ptr<Rbox<'static, ColorType>>,
    m_width: Ptr<Slider<'static, ColorType>>,
    m_benchmark: Ptr<Cbox<'static, ColorType>>,
    m_draw_nodes: Ptr<Cbox<'static, ColorType>>,
    m_draw_edges: Ptr<Cbox<'static, ColorType>>,
    m_draft: Ptr<Cbox<'static, ColorType>>,
    m_translucent: Ptr<Cbox<'static, ColorType>>,
    m_graph: Graph,
    m_gradient_colors: ColorFunc<ColorType>,
    m_draw: i32,
    m_sl: agg::ScanlineU8,
    m_ctrls: CtrlContainer,
    m_util: Rc<RefCell<PlatUtil>>,
}

impl Application {
    fn draw_nodes_draft(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut prim = agg::RendererPrimitives::new(&mut rb);

        for i in 0..self.m_graph.get_num_nodes() {
            let node = self.m_graph.get_node(
                i,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            prim.set_fill_color(self.m_gradient_colors[147]);
            prim.set_line_color(self.m_gradient_colors[255]);
            prim.outlined_ellipse(node.x as i32, node.y as i32, 10, 10);
            prim.set_fill_color(self.m_gradient_colors[50]);
            prim.solid_ellipse(node.x as i32, node.y as i32, 4, 4);
        }
    }

    fn draw_nodes_fine(&mut self, rbuf: &mut RenderBuf, ras: &mut agg::RasterizerScanlineAa) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);

        for i in 0..self.m_graph.get_num_nodes() {
            let node = self.m_graph.get_node(
                i,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let mut ell = agg::Ellipse::new_ellipse(
                node.x,
                node.y,
                5.0 * self.m_width.borrow().value(),
                5.0 * self.m_width.borrow().value(),
                0,
                false,
            );

            match self.m_draw {
                0 => ell.rewind(0),
                1 => ras.add_path(&mut ell, 0),
                2 => {
                    ras.reset();
                    ras.add_path(&mut ell, 0);
                    ras.sort();
                }
                3 => {
                    let mut gf = agg::GradientRadialD {};
                    let mut mtx = agg::TransAffine::new_default();
                    mtx *= agg::TransAffine::trans_affine_scaling_eq(
                        self.m_width.borrow().value() / 2.0,
                    );
                    mtx *= agg::TransAffine::trans_affine_translation(node.x, node.y);
                    mtx.invert();
                    let mut inter = agg::SpanIpLinear::new(mtx);
                    let sa = agg::VecSpan::<ColorType>::new();

                    let c1 = ColorType::new_params(255, 255, 0, 70);
                    let c2 = ColorType::new_params(0, 0, 255, 255);
                    let mut sc = ColorFunc::new();

                    for i in 0..256 {
                        sc[i] = c1.gradient(&c2, i as f64 / 255.0);
                    }

                    let mut sg = agg::SpanGradient::<
                        ColorType,
                        agg::SpanIpLinear<agg::TransAffine>,
                        agg::GradientRadialD,
                        ColorFunc<ColorType>,
                    >::new(&mut inter, &mut gf, &mut sc, 0., 10.);

                    let mut ren = agg::RendererScanlineAA::new_borrowed(&mut rb, sa, &mut sg);
                    ras.add_path(&mut ell, 0);
                    agg::render_scanlines(ras, &mut self.m_sl, &mut ren);
                }
                _ => {}
            }
        }
    }

    fn render_edge_fine<Source: VertexSource>(
        &mut self, ras: &mut agg::RasterizerScanlineAa, ren_fine: &mut RSlAaS,
        ren_draft: &mut RSlBS, src: &mut Source,
    ) {
        let (mut x, mut y) = (0., 0.);
        match self.m_draw {
            0 => {
                src.rewind(0);
                while !agg::basics::is_stop(src.vertex(&mut x, &mut y)) {}
            }
            1 => {
                ras.reset();
                ras.add_path(src, 0);
            }
            2 => {
                ras.reset();
                ras.add_path(src, 0);
                ras.sort();
            }
            3 => {
                let r = frand() as u32 & 0x7F;
                let g = frand() as u32 & 0x7F;
                let b = frand() as u32 & 0x7F;
                let mut a = 255;
                if self.m_translucent.borrow().status() {
                    a = 80
                };
                ras.add_path(src, 0);

                if self.m_type.borrow().cur_item() < 4 {
                    ren_fine.set_color(agg::Rgba8::new_params(r, g, b, a));
                    agg::render_scanlines(ras, &mut self.m_sl, ren_fine);
                } else {
                    ren_draft.set_color(agg::Rgba8::new_params(r, g, b, a));
                    agg::render_scanlines(ras, &mut self.m_sl, ren_draft);
                }
            }
            _ => {}
        }
    }

    fn draw_lines_draft(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut prim = agg::RendererPrimitives::new(&mut rb);
        let r = frand() as u32 & 0x7F;
        let g = frand() as u32 & 0x7F;
        let b = frand() as u32 & 0x7F;
        let mut a = 255;
        if self.m_translucent.borrow().status() {
            a = 80
        };
        prim.set_line_color(agg::Rgba8::new_params(r, g, b, a));
        let mut ras = agg::RasterizerOutline::new(&mut prim);

        for i in 0..self.m_graph.get_num_edges() {
            let e = self.m_graph.get_edge(i);
            let n1 = self.m_graph.get_node(
                e.node1,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let n2 = self.m_graph.get_node(
                e.node2,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let l = Line::new(n1.x, n1.y, n2.x, n2.y);
            let mut s = StrokeDraft::new(l, self.m_width.borrow().value());

            ras.add_path(&mut s, 0);
        }
    }

    fn draw_curves_draft(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut prim = agg::RendererPrimitives::new(&mut rb);
        let r = frand() as u32 & 0x7F;
        let g = frand() as u32 & 0x7F;
        let b = frand() as u32 & 0x7F;
        let mut a = 255;
        if self.m_translucent.borrow().status() {
            a = 80
        };
        prim.set_line_color(agg::Rgba8::new_params(r, g, b, a));

        let mut ras = agg::RasterizerOutline::new(&mut prim);

        for i in 0..self.m_graph.get_num_edges() {
            let e = self.m_graph.get_edge(i);
            let n1 = self.m_graph.get_node(
                e.node1,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let n2 = self.m_graph.get_node(
                e.node2,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let c = Curve::new(n1.x, n1.y, n2.x, n2.y, 0.5);
            let mut s = StrokeDraft::new(c, self.m_width.borrow().value());

            ras.add_path(&mut s, 0);
        }
    }

    fn draw_dashes_draft(&mut self, rbuf: &mut RenderBuf) {
        let mut pixf = Pixfmt::new_borrowed(rbuf);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf);
        let mut prim = agg::RendererPrimitives::new(&mut rb);
        let r = frand() as u32 & 0x7F;
        let g = frand() as u32 & 0x7F;
        let b = frand() as u32 & 0x7F;
        let mut a = 255;
        if self.m_translucent.borrow().status() {
            a = 80
        };
        prim.set_line_color(agg::Rgba8::new_params(r, g, b, a));
        let mut ras = agg::RasterizerOutline::new(&mut prim);

        for i in 0..self.m_graph.get_num_edges() {
            let e = self.m_graph.get_edge(i);
            let n1 = self.m_graph.get_node(
                e.node1,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let n2 = self.m_graph.get_node(
                e.node2,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let c = Curve::new(n1.x, n1.y, n2.x, n2.y, 0.5);
            let mut s = DashStrokeDraft::new(c, 6.0, 3.0, self.m_width.borrow().value());

            ras.add_path(&mut s, 0);
        }
    }

    fn draw_lines_fine(
        &mut self, _rbuf: &mut RenderBuf, ras: &mut agg::RasterizerScanlineAa, solid: &mut RSlAaS,
        draft: &mut RSlBS,
    ) {
        for i in 0..self.m_graph.get_num_edges() {
            let b = self.m_graph.get_edge(i);
            let n1 = self.m_graph.get_node(
                b.node1,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let n2 = self.m_graph.get_node(
                b.node2,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let l = Line::new(n1.x, n1.y, n2.x, n2.y);
            let mut s = StrokeFineArrow::new(l, self.m_width.borrow().value());
            self.render_edge_fine(ras, solid, draft, &mut s);
        }
    }

    fn draw_curves_fine(
        &mut self, _rbuf: &mut RenderBuf, ras: &mut agg::RasterizerScanlineAa, solid: &mut RSlAaS,
        draft: &mut RSlBS,
    ) {
        for i in 0..self.m_graph.get_num_edges() {
            let b = self.m_graph.get_edge(i);
            let n1 = self.m_graph.get_node(
                b.node1,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let n2 = self.m_graph.get_node(
                b.node2,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let c = Curve::new(n1.x, n1.y, n2.x, n2.y, 0.5);
            let mut s = StrokeFineArrow::new(c, self.m_width.borrow().value());
            self.render_edge_fine(ras, solid, draft, &mut s);
        }
    }

    fn draw_dashes_fine(
        &mut self, _rbuf: &mut RenderBuf, ras: &mut agg::RasterizerScanlineAa, solid: &mut RSlAaS,
        draft: &mut RSlBS,
    ) {
        for i in 0..self.m_graph.get_num_edges() {
            let b = self.m_graph.get_edge(i);
            let n1 = self.m_graph.get_node(
                b.node1,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let n2 = self.m_graph.get_node(
                b.node2,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let c = Curve::new(n1.x, n1.y, n2.x, n2.y, 0.5);
            let mut s = DashStrokeFineArrow::new(c, 6.0, 3.0, self.m_width.borrow().value());
            self.render_edge_fine(ras, solid, draft, &mut s);
        }
    }

    fn draw_polygons(
        &mut self, _rbuf: &mut RenderBuf, ras: &mut agg::RasterizerScanlineAa, solid: &mut RSlAaS,
        draft: &mut RSlBS,
    ) {
        if self.m_type.borrow().cur_item() == 4 {
            ras.set_gamma(&agg::GammaThreshold::new_with_threshold(0.5));
        }
        for i in 0..self.m_graph.get_num_edges() {
            let b = self.m_graph.get_edge(i);
            let n1 = self.m_graph.get_node(
                b.node1,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let n2 = self.m_graph.get_node(
                b.node2,
                self.m_util.borrow().width(),
                self.m_util.borrow().height(),
            );
            let mut c = Curve::new(n1.x, n1.y, n2.x, n2.y, 0.5);
            self.render_edge_fine(ras, solid, draft, &mut c);
        }
        ras.set_gamma(&agg::GammaNone::new());
    }

    fn draw_scene(
        &mut self, rbuf: &mut RenderBuf, ras: &mut agg::RasterizerScanlineAa, solid: &mut RSlAaS,
        draft: &mut RSlBS,
    ) {
        ras.set_gamma(&agg::GammaNone::new());
        unsafe {
            srand(100);
        }
        if self.m_draw_nodes.borrow().status() {
            if self.m_draft.borrow().status() {
                self.draw_nodes_draft(rbuf);
            } else {
                self.draw_nodes_fine(rbuf, ras);
            }
        }

        if self.m_draw_edges.borrow().status() {
            let m = self.m_type.borrow().cur_item();
            if self.m_draft.borrow().status() {
                match m {
                    0 => self.draw_lines_draft(rbuf),
                    1 => self.draw_curves_draft(rbuf),
                    2 => self.draw_dashes_draft(rbuf),
                    _ => (),
                }
            } else {
                match m {
                    0 => self.draw_lines_fine(rbuf, ras, solid, draft),
                    1 => self.draw_curves_fine(rbuf, ras, solid, draft),
                    2 => self.draw_dashes_fine(rbuf, ras, solid, draft),
                    3 | 4 => self.draw_polygons(rbuf, ras, solid, draft),
                    _ => (),
                }
            }
        }
    }
}

type RB<'a> = agg::RendererBase<'a, Pixfmt<'a>>;
type RSlAaS<'a, 'b> = agg::RendererScanlineAASolid<'a, RB<'b>>;
type RSlBS<'a, 'b> = agg::RendererScanlineBinSolid<'a, RB<'b>>;
impl Interface for Application {
    fn on_ctrls(&mut self) -> &mut CtrlContainer {
        &mut self.m_ctrls
    }

    fn new(_format: PixFormat, flip_y: bool, util: Ptr<PlatUtil>) -> Self {
        let mut m_type = Rbox::new(-1., -1., -1., -1., !flip_y);
        let mut m_width = Slider::new(110. + 80., 8.0, 110. + 200.0 + 80., 8.0 + 7.0, !flip_y);
        let mut m_benchmark = Cbox::new(110. + 200. + 80. + 8., 8.0 - 2.0, "Benchmark", !flip_y);
        let mut m_draw_nodes = Cbox::new(
            110. + 200. + 80. + 8.,
            8.0 - 2.0 + 15.0,
            "Draw Nodes",
            !flip_y,
        );
        let mut m_draw_edges = Cbox::new(
            200. + 200. + 80. + 8.,
            8.0 - 2.0 + 15.0,
            "Draw Edges",
            !flip_y,
        );
        let mut m_draft = Cbox::new(200. + 200. + 80. + 8., 8.0 - 2.0, "Draft Mode", !flip_y);
        let m_translucent = Cbox::new(110. + 80., 8.0 - 2.0 + 15.0, "Translucent Mode", !flip_y);

        m_type.set_text_size(8.0, 0.);
        m_type.add_item("Solid lines");
        m_type.add_item("Bezier curves");
        m_type.add_item("Dashed curves");
        m_type.add_item("Poygons AA");
        m_type.add_item("Poygons Bin");
        m_type.set_cur_item(0);

        m_width.set_num_steps(20);
        m_width.set_range(0.0, 5.0);
        m_width.set_value(2.0);
        m_width.set_label("Width=%1.2f");

        m_benchmark.set_text_size(8.0, 0.);
        m_draw_nodes.set_text_size(8.0, 0.);
        m_draft.set_text_size(8.0, 0.);
        m_draw_nodes.set_status(true);
        m_draw_edges.set_status(true);

        let c1 = ColorType::new_params(255, 255, 0, 70);
        let c2 = ColorType::new_params(0, 0, 255, 255);

        let m_type = ctrl_ptr(m_type);
        let m_width = ctrl_ptr(m_width);
        let m_benchmark = ctrl_ptr(m_benchmark);
        let m_draw_nodes = ctrl_ptr(m_draw_nodes);
        let m_draw_edges = ctrl_ptr(m_draw_edges);
        let m_draft = ctrl_ptr(m_draft);
        let m_translucent = ctrl_ptr(m_translucent);

        let mut app = Application {
            m_type: m_type.clone(),
            m_width: m_width.clone(),
            m_benchmark: m_benchmark.clone(),
            m_draw_nodes: m_draw_nodes.clone(),
            m_draw_edges: m_draw_edges.clone(),
            m_draft: m_draft.clone(),
            m_translucent: m_translucent.clone(),
            m_graph: Graph::new(200, 100),
            m_gradient_colors: ColorFunc::new(),
            m_draw: 3,
            m_sl: agg::ScanlineU8::new(),
            m_ctrls: CtrlContainer {
                ctrl: vec![
                    m_type,
                    m_width,
                    m_benchmark,
                    m_draw_nodes,
                    m_draw_edges,
                    m_draft,
                    m_translucent,
                ],
                cur_ctrl: -1,
                num_ctrl: 7,
            },
            m_util: util,
        };
        for i in 0..256 {
            app.m_gradient_colors[i] = c1.gradient(&c2, i as f64 / 255.0);
        }
        app
    }

    fn on_draw(&mut self, rbuf: &mut RenderBuf) {
        let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();

        let mut tmp_rbuf0 = *rbuf;
        let mut tmp_rbuf1 = *rbuf;
        let mut pixf0 = Pixfmt::new_borrowed(&mut tmp_rbuf0);
        let mut rb = agg::RendererBase::new_borrowed(&mut pixf0);
        rb.clear(&ColorType::new_params(255, 255, 255, 255));
        let mut pixf1 = Pixfmt::new_borrowed(&mut tmp_rbuf1);
        let mut rb1 = agg::RendererBase::new_borrowed(&mut pixf1);
        {
            let mut solid = agg::RendererScanlineAASolid::new_borrowed(&mut rb);
            let mut draft = agg::RendererScanlineBinSolid::new_borrowed(&mut rb1);

            self.draw_scene(rbuf, &mut ras, &mut solid, &mut draft);
        }
        ras.set_filling_rule(agg::basics::FillingRule::FillNonZero);
        ctrl::render_ctrl(
            &mut ras,
            &mut self.m_sl,
            &mut rb,
            &mut *self.m_type.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut self.m_sl,
            &mut rb,
            &mut *self.m_width.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut self.m_sl,
            &mut rb,
            &mut *self.m_benchmark.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut self.m_sl,
            &mut rb,
            &mut *self.m_draw_nodes.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut self.m_sl,
            &mut rb,
            &mut *self.m_draw_edges.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut self.m_sl,
            &mut rb,
            &mut *self.m_draft.borrow_mut(),
        );
        ctrl::render_ctrl(
            &mut ras,
            &mut self.m_sl,
            &mut rb,
            &mut *self.m_translucent.borrow_mut(),
        );
    }

    fn on_ctrl_change(&mut self, rbuf: &mut agg::RenderBuf) {
        if self.m_benchmark.borrow().status() {
            //self.on_draw();
            //update_window();

            let mut ras: agg::RasterizerScanlineAa = agg::RasterizerScanlineAa::new();
            let mut tmp_rbuf0 = *rbuf;
            let mut tmp_rbuf1 = *rbuf;
            let mut pixf0 = Pixfmt::new_borrowed(&mut tmp_rbuf0);
            let mut rb0 = agg::RendererBase::new_borrowed(&mut pixf0);
            let mut pixf1 = Pixfmt::new_borrowed(&mut tmp_rbuf1);
            let mut rb1 = agg::RendererBase::new_borrowed(&mut pixf1);

            let mut solid = agg::RendererScanlineAASolid::new_borrowed(&mut rb0);
            let mut draft = agg::RendererScanlineBinSolid::new_borrowed(&mut rb1);

            //rb.clear(ColorType::new_params(1.0, 1.0, 1.0, 1.0));

            let buf;
            if self.m_draft.borrow().status() {
                self.m_util.borrow_mut().start_timer();
                for _ in 0..10 {
                    self.draw_scene(rbuf, &mut ras, &mut solid, &mut draft);
                }
                let time = self.m_util.borrow_mut().elapsed_time();
                buf = format!("{:0.3} milliseconds", time);
            } else {
                let mut times = [0.0; 5];
                for m_draw in 0..4 {
                    self.m_util.borrow_mut().start_timer();
                    for _ in 0..10 {
                        self.draw_scene(rbuf, &mut ras, &mut solid, &mut draft);
                    }
                    times[m_draw as usize] = self.m_util.borrow_mut().elapsed_time();
                }
                self.m_draw = 3;

                times[4] = times[3];
                times[3] -= times[2];
                times[2] -= times[1];
                times[1] -= times[0];

                /*let file_name = full_file_name("benchmark");
                let fd = File::open(file_name).expect("Could not open file");
                fprintf(
                    fd,
                    "{:10.3} {:10.3} {:10.3} {:10.3} {:10.3}\n",
                    times[0],
                    times[1],
                    times[2],
                    times[3],
                    times[4],
                );
                fd.close();*/

                buf = format!("  pipeline  add_path         sort       render       total\n {:10.3} {:10.3} {:10.3} {:10.3} {:10.3}"
                             ,
                        times[0], times[1], times[2], times[3], times[4]);
            }
            self.m_util.borrow_mut().message(&buf);

            self.m_benchmark.borrow_mut().set_status(false);
            //return true
        }
    }
}

fn main() {
    let mut plat = PlatSupport::<Application>::create_plat(PIXEL_FORMAT, FLIP_Y);
    plat.set_caption("AGG Example. Line Join");

    if plat.init(600 + 100, 500 + 30, WindowFlag::Resize as u32) {
        plat.run();
    }
}

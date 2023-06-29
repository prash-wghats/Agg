use std::marker::PhantomData;

use crate::{RasterScanLine, basics::*};
use crate::line_aa_basics::{LineSubpixel, *};
use crate::vertex_sequence::*;
use crate::{Coord, RendererOutline, VertexDistance, VertexSequence, VertexSource};
use std::ops::Index;

#[inline]
fn cmp_dist_start(d: i32) -> bool {
    d > 0
}
#[inline]
fn cmp_dist_end(d: i32) -> bool {
    d <= 0
}

//-----------------------------------------------------------LineAaVertex
// Vertex (x, y) with the distance to the next one. The last vertex has
// the distance between the last and the first points
#[derive(Clone, Copy)]
pub struct LineAaVertex {
    x: i32,
    y: i32,
    len: i32,
}
impl LineAaVertex {
    pub fn new() -> LineAaVertex {
        LineAaVertex { x: 0, y: 0, len: 0 }
    }

    pub fn new_xy(x_: i32, y_: i32) -> LineAaVertex {
        LineAaVertex {
            x: x_,
            y: y_,
            len: 0,
        }
    }
}

impl VertexDistance for LineAaVertex {
    fn calc_distance(&mut self, val: &Self) -> bool {
        let dx = (val.x - self.x) as f64;
        let dy = (val.y - self.y) as f64;
        self.len = uround((dx * dx + dy * dy).sqrt());
        return self.len > (LineSubpixel::Scale as i32 + LineSubpixel::Scale as i32 / 2);
    }
}

use self::OutlineAaJoin::*;

//----------------------------------------------------------OutlineAaJoin
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutlineAaJoin {
    NoJoin,        //-----NoJoin
    Miter,         //-----Miter
    Round,         //-----Round
    MiterAccurate, //-----outline_accurate_join
}

type VertexType = LineAaVertex;
type VertexStorageType = VecSequence<VertexType>;

#[derive(Copy, Clone)]
pub struct DrawVars {
    pub idx: u32,
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub curr: LineParameters,
    pub next: LineParameters,
    pub lcurr: i32,
    pub lnext: i32,
    pub xb1: i32,
    pub yb1: i32,
    pub xb2: i32,
    pub yb2: i32,
    pub flags: u32,
}

impl DrawVars {
    pub fn new() -> Self {
        DrawVars {
            idx: 0,
            x1: 0,
            y1: 0,
            x2: 0,
            y2: 0,
            curr: LineParameters::new_default(),
            next: LineParameters::new_default(),
            lcurr: 0,
            lnext: 0,
            xb1: 0,
            yb1: 0,
            xb2: 0,
            yb2: 0,
            flags: 0,
        }
    }
}
//=======================================================RasterizerOutlineAa
pub struct RasterizerOutlineAa<'a, Ren: RendererOutline, Co: Coord = LineCoord> {
    ren: &'a mut Ren,
    line_join: OutlineAaJoin,
    round_cap: bool,
    start_x: i32,
    start_y: i32,
    src_vertices: VertexStorageType,
    dum_co: PhantomData<Co>,
}

impl<'a, Ren: RendererOutline, Co: Coord> RasterizerOutlineAa<'a, Ren, Co> {
    pub fn new(ren: &'a mut Ren) -> Self {
        let b = ren.accurate_join_only();
        RasterizerOutlineAa {
            ren: ren,
            line_join: if b { MiterAccurate } else { Round },
            round_cap: false,
            start_x: 0,
            start_y: 0,
            src_vertices: VertexStorageType::new(),
            dum_co: PhantomData,
        }
    }

    pub fn attach(&mut self, ren: &'a mut Ren) {
        self.ren = ren;
    }

	pub fn ren_mut(&mut self) -> &mut Ren {
        return self.ren;
    }

    pub fn ren(&self) -> &Ren {
        return &*self.ren;
    }

    pub fn set_line_join(&mut self, join: OutlineAaJoin) {
        self.line_join = if self.ren.accurate_join_only() {
            MiterAccurate
        } else {
            join
        };
    }

    pub fn line_join(&self) -> OutlineAaJoin {
        self.line_join
    }

    pub fn set_round_cap(&mut self, v: bool) {
        self.round_cap = v;
    }

    pub fn round_cap_(&self) -> bool {
        self.round_cap
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.start_x = x;
        self.start_y = y;
        self.src_vertices.modify_last(VertexType::new_xy(x, y));
    }

    pub fn line_to(&mut self, x: i32, y: i32) {
        self.src_vertices.add(VertexType::new_xy(x, y));
    }

    pub fn move_to_d(&mut self, x: f64, y: f64) {
        self.move_to(Co::conv(x), Co::conv(y));
    }

    pub fn line_to_d(&mut self, x: f64, y: f64) {
        self.line_to(Co::conv(x), Co::conv(y));
    }

    pub fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        if is_move_to(cmd) {
            self.render(false);
            self.move_to_d(x, y);
        } else {
            if is_end_poly(cmd) {
                self.render(is_closed(cmd));
                if is_closed(cmd) {
                    self.move_to(self.start_x, self.start_y);
                }
            } else {
                self.line_to_d(x, y);
            }
        }
    }

    pub fn render_all_paths<
        VS: VertexSource,
        CS: Index<usize, Output = Ren::C>,
        PathId: Index<usize, Output = u32>,
    >(
        &mut self, vs: &mut VS, colors: &CS, path_id: &PathId, num_paths: u32,
    ) {
        for i in 0..num_paths as usize {
            self.ren.set_color(colors[i]);
            self.add_path(vs, path_id[i]);
        }
    }
    /*
    pub fn render_ctrl<Ctrl:Ctrl>(&mut self, c: &mut Ctrl) {
        for i in 0..c.num_paths() {
            self.ren.color(c.color(i));
            self.add_path(c, i);
        }
    }
    */
    //////////////

    fn draw(&mut self, dv: &mut DrawVars, start: usize, end: usize) {
        let mut i = start;
        while i < end {
            if self.line_join == Round {
                dv.xb1 = dv.curr.x1 + (dv.curr.y2 - dv.curr.y1);
                dv.yb1 = dv.curr.y1 - (dv.curr.x2 - dv.curr.x1);
                dv.xb2 = dv.curr.x2 + (dv.curr.y2 - dv.curr.y1);
                dv.yb2 = dv.curr.y2 - (dv.curr.x2 - dv.curr.x1);
            }

            match dv.flags {
                0 => self.ren.line3(&dv.curr, dv.xb1, dv.yb1, dv.xb2, dv.yb2),
                1 => self.ren.line2(&dv.curr, dv.xb2, dv.yb2),
                2 => self.ren.line1(&dv.curr, dv.xb1, dv.yb1),
                3 => self.ren.line0(&dv.curr),
                _ => panic!("Invalid flags"),
            }

            if self.line_join == Round && (dv.flags & 2) == 0 {
                self.ren.pie(
                    dv.curr.x2,
                    dv.curr.y2,
                    dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                    dv.curr.y2 - (dv.curr.x2 - dv.curr.x1),
                    dv.curr.x2 + (dv.next.y2 - dv.next.y1),
                    dv.curr.y2 - (dv.next.x2 - dv.next.x1),
                );
            }

            dv.x1 = dv.x2;
            dv.y1 = dv.y2;
            dv.lcurr = dv.lnext;
            dv.lnext = self.src_vertices[dv.idx as usize].len;

            dv.idx += 1;
            if dv.idx >= self.src_vertices.size() as u32 {
                dv.idx = 0;
            }

            let v = &self.src_vertices[dv.idx as usize];
            dv.x2 = v.x;
            dv.y2 = v.y;

            dv.curr = dv.next;
            dv.next = LineParameters::new(dv.x1, dv.y1, dv.x2, dv.y2, dv.lnext);
            dv.xb1 = dv.xb2;
            dv.yb1 = dv.yb2;

            match self.line_join {
                NoJoin => dv.flags = 3,
                Miter => {
                    dv.flags >>= 1;
                    dv.flags |=
                        ((dv.curr.diagonal_quadrant() == dv.next.diagonal_quadrant()) as u32) << 1;
                    if (dv.flags & 2) == 0 {
                        bisectrix(&dv.curr, &dv.next, &mut dv.xb2, &mut dv.yb2);
                    }
                }
                Round => {
                    dv.flags >>= 1;
                    dv.flags |=
                        ((dv.curr.diagonal_quadrant() == dv.next.diagonal_quadrant()) as u32) << 1;
                }
                MiterAccurate => {
                    dv.flags = 0;
                    bisectrix(&dv.curr, &dv.next, &mut dv.xb2, &mut dv.yb2);
                }
            }

            i += 1;
        }
    }

    pub fn render(&mut self, close_polygon: bool) {
        self.src_vertices.close(close_polygon);
        let mut dv = DrawVars::new();
        let mut v: &VertexType;
        let x1: i32;
        let y1: i32;
        let x2: i32;
        let y2: i32;
        let lprev: i32;

        if close_polygon {
            if self.src_vertices.size() >= 3 {
                dv.idx = 2;

                v = &self.src_vertices[self.src_vertices.size() - 1];
                x1 = v.x;
                y1 = v.y;
                lprev = v.len;

                v = &self.src_vertices[0];
                x2 = v.x;
                y2 = v.y;
                dv.lcurr = v.len;
                let prev = LineParameters::new(x1, y1, x2, y2, lprev);

                v = &self.src_vertices[1];
                dv.x1 = v.x;
                dv.y1 = v.y;
                dv.lnext = v.len;
                dv.curr = LineParameters::new(x2, y2, dv.x1, dv.y1, dv.lcurr);

                v = &self.src_vertices[dv.idx as usize];
                dv.x2 = v.x;
                dv.y2 = v.y;
                dv.next = LineParameters::new(dv.x1, dv.y1, dv.x2, dv.y2, dv.lnext);

                dv.xb1 = 0;
                dv.yb1 = 0;
                dv.xb2 = 0;
                dv.yb2 = 0;

                match self.line_join {
                    NoJoin => dv.flags = 3,
                    Miter | Round => {
                        dv.flags = (prev.diagonal_quadrant() == dv.curr.diagonal_quadrant()) as u32
                            | (((dv.curr.diagonal_quadrant() == dv.next.diagonal_quadrant())
                                as u32)
                                << 1);
                    }
                    MiterAccurate => dv.flags = 0,
                }

                if (dv.flags & 1) == 0 && self.line_join != Round {
                    bisectrix(&prev, &dv.curr, &mut dv.xb1, &mut dv.yb1);
                }

                if (dv.flags & 2) == 0 && self.line_join != Round {
                    bisectrix(&dv.curr, &dv.next, &mut dv.xb2, &mut dv.yb2);
                }
                self.draw(&mut dv, 0, self.src_vertices.size());
            }
        } else {
            match self.src_vertices.len() {
                0 | 1 => {}
                2 => {
                    v = &self.src_vertices[0];
                    x1 = v.x;
                    y1 = v.y;
                    lprev = v.len;
                    v = &self.src_vertices[1];
                    x2 = v.x;
                    y2 = v.y;
                    let lp = LineParameters::new(x1, y1, x2, y2, lprev);
                    if self.round_cap {
                        self.ren
                            .semidot(cmp_dist_start, x1, y1, x1 + (y2 - y1), y1 - (x2 - x1));
                    }
                    self.ren.line3(
                        &lp,
                        x1 + (y2 - y1),
                        y1 - (x2 - x1),
                        x2 + (y2 - y1),
                        y2 - (x2 - x1),
                    );
                    if self.round_cap {
                        self.ren
                            .semidot(cmp_dist_end, x2, y2, x2 + (y2 - y1), y2 - (x2 - x1));
                    }
                }
                3 => {
                    let x3: i32;
                    let y3: i32;
                    let lnext: i32;
                    v = &self.src_vertices[0];
                    x1 = v.x;
                    y1 = v.y;
                    lprev = v.len;
                    v = &self.src_vertices[1];
                    x2 = v.x;
                    y2 = v.y;
                    lnext = v.len;
                    v = &self.src_vertices[2];
                    x3 = v.x;
                    y3 = v.y;
                    let lp1 = LineParameters::new(x1, y1, x2, y2, lprev);
                    let lp2 = LineParameters::new(x2, y2, x3, y3, lnext);

                    if self.round_cap {
                        self.ren
                            .semidot(cmp_dist_start, x1, y1, x1 + (y2 - y1), y1 - (x2 - x1));
                    }

                    if self.line_join == Round {
                        self.ren.line3(
                            &lp1,
                            x1 + (y2 - y1),
                            y1 - (x2 - x1),
                            x2 + (y2 - y1),
                            y2 - (x2 - x1),
                        );

                        self.ren.pie(
                            x2,
                            y2,
                            x2 + (y2 - y1),
                            y2 - (x2 - x1),
                            x2 + (y3 - y2),
                            y2 - (x3 - x2),
                        );

                        self.ren.line3(
                            &lp2,
                            x2 + (y3 - y2),
                            y2 - (x3 - x2),
                            x3 + (y3 - y2),
                            y3 - (x3 - x2),
                        );
                    } else {
                        bisectrix(&lp1, &lp2, &mut dv.xb1, &mut dv.yb1);
                        self.ren
                            .line3(&lp1, x1 + (y2 - y1), y1 - (x2 - x1), dv.xb1, dv.yb1);

                        self.ren
                            .line3(&lp2, dv.xb1, dv.yb1, x3 + (y3 - y2), y3 - (x3 - x2));
                    }
                    if self.round_cap {
                        self.ren
                            .semidot(cmp_dist_end, x3, y3, x3 + (y3 - y2), y3 - (x3 - x2));
                    }
                }
                _ => {
                    dv.idx = 3;

                    v = &self.src_vertices[0];
                    x1 = v.x;
                    y1 = v.y;
                    lprev = v.len;

                    v = &self.src_vertices[1];
                    x2 = v.x;
                    y2 = v.y;
                    dv.lcurr = v.len;
                    let prev = LineParameters::new(x1, y1, x2, y2, lprev);

                    v = &self.src_vertices[2];
                    dv.x1 = v.x;
                    dv.y1 = v.y;
                    dv.lnext = v.len;
                    dv.curr = LineParameters::new(x2, y2, dv.x1, dv.y1, dv.lcurr);

                    v = &self.src_vertices[dv.idx as usize];
                    dv.x2 = v.x;
                    dv.y2 = v.y;
                    dv.next = LineParameters::new(dv.x1, dv.y1, dv.x2, dv.y2, dv.lnext);

                    dv.xb1 = 0;
                    dv.yb1 = 0;
                    dv.xb2 = 0;
                    dv.yb2 = 0;

                    match self.line_join {
                        NoJoin => dv.flags = 3,
                        Miter | Round => {
                            dv.flags = (prev.diagonal_quadrant() == dv.curr.diagonal_quadrant())
                                as u32
                                | ((dv.curr.diagonal_quadrant() == dv.next.diagonal_quadrant())
                                    as u32)
                                    << 1
                        }
                        MiterAccurate => dv.flags = 0,
                    }

                    if self.round_cap {
                        self.ren
                            .semidot(cmp_dist_start, x1, y1, x1 + (y2 - y1), y1 - (x2 - x1));
                    }
                    if (dv.flags & 1) == 0 {
                        if self.line_join == Round {
                            self.ren.line3(
                                &prev,
                                x1 + (y2 - y1),
                                y1 - (x2 - x1),
                                x2 + (y2 - y1),
                                y2 - (x2 - x1),
                            );
                            self.ren.pie(
                                prev.x2,
                                prev.y2,
                                x2 + (y2 - y1),
                                y2 - (x2 - x1),
                                dv.curr.x1 + (dv.curr.y2 - dv.curr.y1),
                                dv.curr.y1 - (dv.curr.x2 - dv.curr.x1),
                            );
                        } else {
                            bisectrix(&prev, &dv.curr, &mut dv.xb1, &mut dv.yb1);
                            self.ren
                                .line3(&prev, x1 + (y2 - y1), y1 - (x2 - x1), dv.xb1, dv.yb1);
                        }
                    } else {
                        self.ren.line1(&prev, x1 + (y2 - y1), y1 - (x2 - x1));
                    }
                    if (dv.flags & 2) == 0 && self.line_join != Round {
                        bisectrix(&dv.curr, &dv.next, &mut dv.xb2, &mut dv.yb2);
                    }

                    self.draw(&mut dv, 1, self.src_vertices.len() - 2);

                    if (dv.flags & 1) == 0 {
                        if self.line_join == Round {
                            self.ren.line3(
                                &dv.curr,
                                dv.curr.x1 + (dv.curr.y2 - dv.curr.y1),
                                dv.curr.y1 - (dv.curr.x2 - dv.curr.x1),
                                dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                                dv.curr.y2 - (dv.curr.x2 - dv.curr.x1),
                            );
                        } else {
                            self.ren.line3(
                                &dv.curr,
                                dv.xb1,
                                dv.yb1,
                                dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                                dv.curr.y2 - (dv.curr.x2 - dv.curr.x1),
                            );
                        }
                    } else {
                        self.ren.line2(
                            &dv.curr,
                            dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                            dv.curr.y2 - (dv.curr.x2 - dv.curr.x1),
                        );
                    }
                    if self.round_cap {
                        self.ren.semidot(
                            cmp_dist_end,
                            dv.curr.x2,
                            dv.curr.y2,
                            dv.curr.x2 + (dv.curr.y2 - dv.curr.y1),
                            dv.curr.y2 - (dv.curr.x2 - dv.curr.x1),
                        );
                    }
                }
            }
        }

        self.src_vertices.remove_all();
    }
}

impl<'a, Ren: RendererOutline, Co: Coord> RasterScanLine for RasterizerOutlineAa<'a, Ren, Co> {
	fn add_path<VS: VertexSource>(&mut self, vs: &mut VS, path_id: u32) {
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        let mut cmd;
        vs.rewind(path_id);
        loop {
            cmd = vs.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            self.add_vertex(x, y, cmd);
        }
        self.render(false);
    }
	fn min_x(&self) -> i32 {
        0
    }

    fn min_y(&self) -> i32 {
        0
    }

    fn max_x(&self) -> i32 {
        0
    }

    fn max_y(&self) -> i32 {
        0
    }

}
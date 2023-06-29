use crate::array::{VecPodB};
use crate::basics::{PathCmd, PathFlag, *};
use crate::{VertexConsumer, VertexSource};
use std::marker::PhantomData;
use std::ptr::*;

include!("../gpc/gpc.rs");

#[derive(PartialEq, Eq)]
pub enum GpcOp {
    Or,
    And,
    Xor,
    AMinusB,
    BMinusA,
}

#[derive(PartialEq, Eq)]
enum Status {
    MoveTo,
    LineTo,
    Stop,
}

struct ContourHeader {
    num_vertices: i32,
    hole_flag: i32,
    vertices: Vec<GpcVertex>,
}

//================================================================ConvGpc
pub struct ConvGpc<'a, VS1: VertexSource, VS2: VertexSource> {
    src: [&'a mut dyn VertexSource; 2],
    //src_b: &'a mut VS2,
    status: Status,
    vertex: i32,
    contour: i32,
    operation: GpcOp,
    poly_a: GpcPolygon,
    poly_b: GpcPolygon,
    result: GpcPolygon,
    mem: [Vec<GpcVertexList>; 3],
    vertex_accumulator: VecPodB<GpcVertex>,
    contour_accumulator: [VecPodB<ContourHeader>; 2],
    v1: PhantomData<VS1>,
    v2: PhantomData<VS2>,
}

impl<'a, VS1: VertexSource, VS2: VertexSource> ConvGpc<'a, VS1, VS2>
where
    VS1: VertexSource,
    VS2: VertexSource,
{
    pub fn new(a: &'a mut VS1, b: &'a mut VS2, op: GpcOp) -> ConvGpc<'a, VS1, VS2> {
        ConvGpc {
            src: [a, b],
            //src_b: b,
            status: Status::MoveTo,
            vertex: -1,
            contour: -1,
            operation: op,
            poly_a: GpcPolygon {
                num_contours: 0,
                hole: null_mut(),
                contour: null_mut(),
            },
            poly_b: GpcPolygon {
                num_contours: 0,
                hole: null_mut(),
                contour: null_mut(),
            },
            result: GpcPolygon {
                num_contours: 0,
                hole: null_mut(),
                contour: null_mut(),
            },
            vertex_accumulator: VecPodB::new(),
            contour_accumulator: [VecPodB::new(), VecPodB::new()],
            mem: [Vec::new(), Vec::new(), Vec::new()],
            v1: PhantomData,
            v2: PhantomData,
        }
    }

    pub fn attach1(&mut self, source: &'a mut VS1) {
        self.src[0] = source;
    }

    pub fn attach2(&mut self, source: &'a mut VS2) {
        self.src[1] = source;
    }

    pub fn operation(&mut self, v: GpcOp) {
        self.operation = v;
    }
    #[inline]
    fn add(&mut self, id: usize) {
        let mut cmd: u32;
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;
        let mut start_x: f64 = 0.0;
        let mut start_y: f64 = 0.0;
        let mut line_to: bool = false;
        let mut orientation: u32 = 0;
        //let src = &mut self.src[id];
        self.contour_accumulator[id].clear();
        loop {
            cmd = self.src[id].vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_vertex(cmd) {
                if is_move_to(cmd) {
                    if line_to {
                        self.end_contour(orientation, id);
                        orientation = 0;
                    }
                    self.start_contour(id);
                    start_x = x;
                    start_y = y;
                }
                self.add_vertex(x, y);
                line_to = true;
            } else {
                if is_end_poly(cmd) {
                    orientation = get_orientation(cmd);
                    if line_to && is_closed(cmd) {
                        self.add_vertex(start_x, start_y);
                    }
                }
            }
        }
        if line_to {
            self.end_contour(orientation, id);
        }
        self.make_polygon(id);
    }

    fn free_polygon(&mut self, id: usize) {
        let p;
        p = match id {
            0 => &mut self.poly_a,
            1 => &mut self.poly_b,
            _ => &mut self.result,
        };
        p.contour = null_mut();
        p.hole = null_mut();
        p.num_contours = 0;
        if id < 2 { self.mem[id].clear(); }
    }

    fn free_result(&mut self) {
        if self.result.contour != null_mut() {
            unsafe { gpc_free_polygon(&mut self.result) };
        }
        self.free_polygon(2);
    }

    fn free_gpc_data(&mut self) {
        self.free_polygon(0);
        self.free_polygon(1);
        self.free_result();
    }

    fn start_contour(&mut self, id: usize) {
        let h = ContourHeader {
            num_vertices: 0,
            hole_flag: 0,
            vertices: vec![],
        };
        self.contour_accumulator[id].add(h);
        self.vertex_accumulator.remove_all();
    }

    fn add_vertex(&mut self, x: f64, y: f64) {
        let mut v: GpcVertex = unsafe { std::mem::zeroed() };
        v.x = x;
        v.y = y;
        self.vertex_accumulator.add(v);
    }

    fn end_contour(&mut self, _orientation: u32, id: usize) {
        let alen = self.contour_accumulator[id].len();
        if alen > 0 {
            if self.vertex_accumulator.len() > 2 {
                let mut h: &mut ContourHeader = &mut self.contour_accumulator[id][alen - 1];

                h.num_vertices = self.vertex_accumulator.len() as i32;
                h.hole_flag = 0;

                // TO DO: Clarify the "holes"
                //if(is_cw(orientation)) h.hole_flag = 1;

                h.vertices.clear();
                h.vertices
                    .resize(h.num_vertices as usize, GpcVertex { x: 0., y: 0. });
                for i in 0..h.num_vertices as usize {
                    let s: &GpcVertex = &self.vertex_accumulator[i];
                    h.vertices[i].x = s.x;
                    h.vertices[i].y = s.y;
                }
            } else {
                self.vertex_accumulator.pop(); //remove_last();
            }
        }
    }

    fn make_polygon(&mut self, id: usize) {
        self.free_polygon(id);
        let p;
        p = match id {
            0 => &mut self.poly_a,
            1 => &mut self.poly_b,
            _ => &mut self.result,
        };
        if self.contour_accumulator[id].len() > 0 {
            p.num_contours = self.contour_accumulator[id].len() as i32;

            p.hole = null_mut();
            self.mem[id].clear();
            self.mem[id].resize(
                p.num_contours as usize,
                GpcVertexList {
                    num_vertices: 0,
                    vertex: null_mut(),
                },
            );

            p.contour = self.mem[id].as_mut_ptr();
            for i in 0..p.num_contours as usize {
                let h: &mut ContourHeader = &mut self.contour_accumulator[id][i];
                self.mem[id][i].num_vertices = h.num_vertices;
                self.mem[id][i].vertex = h.vertices.as_mut_ptr();
            }
        }
    }

    fn start_extracting(&mut self) {
        self.status = Status::MoveTo;
        self.contour = -1;
        self.vertex = -1;
    }

    fn next_contour(&mut self) -> bool {
        self.contour += 1;
        if self.contour < self.result.num_contours {
            self.vertex = -1;
            true
        } else {
            false
        }
    }

    fn next_vertex(&mut self, x: &mut f64, y: &mut f64) -> bool {
        let vlist = unsafe { &*self.result.contour.offset(self.contour as isize) };
        self.vertex += 1;
        if self.vertex < vlist.num_vertices {
            let v = unsafe { &*vlist.vertex.offset(self.vertex as isize) };
            *x = v.x;
            *y = v.y;
            true
        } else {
            false
        }
    }
}

impl<'a, VS1: VertexSource, VS2: VertexSource> Drop for ConvGpc<'a, VS1, VS2> {
    fn drop(&mut self) {
        self.free_gpc_data()
    }
}

impl<'a, VS1: VertexSource, VS2: VertexSource> VertexSource for ConvGpc<'a, VS1, VS2> {
    fn rewind(&mut self, path_id: u32) {
        self.free_result();
        self.src[0].rewind(path_id);
        self.src[1].rewind(path_id);

        self.add(0);
        self.add(1);

        unsafe {
            match self.operation {
                GpcOp::Or => {
                    gpc_polygon_clip(
                        GPC_OP_GPC_UNION,
                        &mut self.poly_a,
                        &mut self.poly_b,
                        &mut self.result,
                    );
                }
                GpcOp::And => {
                    gpc_polygon_clip(
                        GPC_OP_GPC_INT,
                        &mut self.poly_a,
                        &mut self.poly_b,
                        &mut self.result,
                    );
                }
                GpcOp::Xor => {
                    gpc_polygon_clip(
                        GPC_OP_GPC_XOR,
                        &mut self.poly_a,
                        &mut self.poly_b,
                        &mut self.result,
                    );
                }
                GpcOp::AMinusB => {
                    gpc_polygon_clip(
                        GPC_OP_GPC_DIFF,
                        &mut self.poly_a,
                        &mut self.poly_b,
                        &mut self.result,
                    );
                }
                GpcOp::BMinusA => {
                    gpc_polygon_clip(
                        GPC_OP_GPC_DIFF,
                        &mut self.poly_b,
                        &mut self.poly_a,
                        &mut self.result,
                    );
                }
            }
        }
        self.start_extracting();
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.status == Status::MoveTo {
            if self.next_contour() {
                if self.next_vertex(x, y) {
                    self.status = Status::LineTo;
                    return PathCmd::MoveTo as u32;
                }
                self.status = Status::Stop;
                return PathCmd::EndPoly as u32 | PathFlag::Close as u32;
            }
        } else {
            if self.next_vertex(x, y) {
                return PathCmd::LineTo as u32;
            } else {
                self.status = Status::MoveTo;
            }
            return PathCmd::EndPoly as u32 | PathFlag::Close as u32;
        }
        return PathCmd::Stop as u32;
    }
}

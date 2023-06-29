use crate::array::PodBVector;
use crate::basics::{is_move_to, is_stop, is_vertex};
use crate::vertex_sequence::{VecSequence, VertexDist};
use crate::VertexSequence;
use crate::{Transformer, VertexSource};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Initial,
    MakingPath,
    Ready,
}
pub type VertexStorage = VecSequence<VertexDist>;
//-------------------------------------------------------TransSinglePath
pub struct TransSinglePath {
    base_length: f64,
    kindex: f64,
    status: Status,
    preserve_x_scale: bool,
    src_vertices: VertexStorage,
}

impl TransSinglePath {
    pub fn new() -> TransSinglePath {
        TransSinglePath {
            base_length: 0.0,
            kindex: 0.0,
            status: Status::Initial,
            preserve_x_scale: true,
            src_vertices: VertexStorage::new(),
        }
    }

    pub fn set_base_length(&mut self, v: f64) {
        self.base_length = v;
    }
    pub fn base_length(&self) -> f64 {
        self.base_length
    }
    pub fn set_preserve_x_scale(&mut self, f: bool) {
        self.preserve_x_scale = f;
    }

    pub fn preserve_x_scale(&self) -> bool {
        self.preserve_x_scale
    }
    pub fn add_path<VS: VertexSource>(&mut self, vs: &mut VS, path_id: u32) {
        let (mut x, mut y) = (0., 0.);
        let mut cmd: u32;
        vs.rewind(path_id);

        loop {
            cmd = vs.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_move_to(cmd) {
                self.move_to(x, y);
            } else {
                if is_vertex(cmd) {
                    self.line_to(x, y);
                }
            }
        }
        self.finalize_path();
    }

    pub fn reset(&mut self) {
        self.src_vertices.remove_all();
        self.kindex = 0.0;
        self.status = Status::Initial;
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        if self.status == Status::Initial {
            self.src_vertices.modify_last(VertexDist::new(x, y));
            self.status = Status::MakingPath;
        } else {
            self.line_to(x, y);
        }
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        if self.status == Status::MakingPath {
            self.src_vertices.add(VertexDist::new(x, y));
        }
    }

    pub fn finalize_path(&mut self) {
        if self.status == Status::MakingPath && self.src_vertices.size() > 1 {
            //let mut i: usize;
            let mut dist: f64;
            let d: f64;

            self.src_vertices.close(false);
            if self.src_vertices.size() > 2 {
                if self.src_vertices[self.src_vertices.size() - 2].dist * 10.0
                    < self.src_vertices[self.src_vertices.size() - 3].dist
                {
                    d = self.src_vertices[self.src_vertices.size() - 3].dist
                        + self.src_vertices[self.src_vertices.size() - 2].dist;
                    let mut len = self.src_vertices.size();
                    self.src_vertices[len - 2] = self.src_vertices[len - 1];

                    self.src_vertices.remove_last();
                    len = self.src_vertices.size();
                    self.src_vertices[len - 2].dist = d;
                }
            }

            dist = 0.0;
            for i in 0..self.src_vertices.size() {
                let v = &mut self.src_vertices[i];
                let d = v.dist;
                v.dist = dist;
                dist += d;
            }
            self.kindex = (self.src_vertices.size() - 1) as f64 / dist;
            self.status = Status::Ready;
        }
    }

    pub fn total_length(&self) -> f64 {
        if self.base_length >= 1e-10 {
            return self.base_length;
        }
        if self.status == Status::Ready {
            return self.src_vertices[self.src_vertices.size() - 1].dist;
        }
        return 0.0;
    }
}

impl Transformer for TransSinglePath {
    fn transform(&self, x: &mut f64, y: &mut f64) {
        if self.status == Status::Ready {
            if self.base_length > 1e-10 {
                *x *= self.src_vertices[self.src_vertices.size() - 1].dist / self.base_length;
            }

            let x1: f64;
            let y1: f64;
            let dx: f64;
            let dy: f64;
            let mut d: f64;
            let dd: f64;
            if *x < 0.0 {
                // Extrapolation on the left
                //--------------------------
                x1 = self.src_vertices[0].x;
                y1 = self.src_vertices[0].y;
                dx = self.src_vertices[1].x - x1;
                dy = self.src_vertices[1].y - y1;
                dd = self.src_vertices[1].dist - self.src_vertices[0].dist;
                d = *x;
            } else if *x > self.src_vertices[self.src_vertices.size() - 1].dist {
                // Extrapolation on the right
                //--------------------------
                let i: usize = self.src_vertices.size() - 2;
                let j: usize = self.src_vertices.size() - 1;
                x1 = self.src_vertices[j].x;
                y1 = self.src_vertices[j].y;
                dx = x1 - self.src_vertices[i].x;
                dy = y1 - self.src_vertices[i].y;
                dd = self.src_vertices[j].dist - self.src_vertices[i].dist;
                d = *x - self.src_vertices[j].dist;
            } else {
                // Interpolation
                //--------------------------
                let mut i: usize = 0;
                let mut j: usize = self.src_vertices.size() - 1;
                if self.preserve_x_scale {
                    let mut k: usize;
                    while (j - i) > 1 {
                        k = (i + j) >> 1;
                        if *x < self.src_vertices[k].dist {
                            j = k;
                        } else {
                            i = k;
                        }
                    }
                    d = self.src_vertices[i].dist;
                    dd = self.src_vertices[j].dist - d;
                    d = *x - d;
                } else {
                    i = (*x * self.kindex) as usize;
                    j = i + 1;
                    dd = self.src_vertices[j].dist - self.src_vertices[i].dist;
                    d = ((*x * self.kindex) - i as f64) * dd;
                }
                x1 = self.src_vertices[i].x;
                y1 = self.src_vertices[i].y;
                dx = self.src_vertices[j].x - x1;
                dy = self.src_vertices[j].y - y1;
            }
            let x2 = x1 + dx * d / dd;
            let y2 = y1 + dy * d / dd;
            *x = x2 - *y * dy / dd;
            *y = y2 + *y * dx / dd;
        }
    }

    fn scaling_abs(&self, x: &mut f64, y: &mut f64) {
        *x = 0.;
        *y = 0.;
        todo!()
    }
}

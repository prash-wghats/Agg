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
//-------------------------------------------------------TransDoublePath
pub struct TransDoublePath {
    src_vertices1: VertexStorage,
    src_vertices2: VertexStorage,
    base_length: f64,
    base_height: f64,
    kindex1: f64,
    kindex2: f64,
    status1: Status,
    status2: Status,
    preserve_x_scale: bool,
}

impl TransDoublePath {
    pub fn new() -> TransDoublePath {
        TransDoublePath {
            src_vertices1: VertexStorage::new(),
            src_vertices2: VertexStorage::new(),
            base_length: 0.0,
            base_height: 1.0,
            kindex1: 0.0,
            kindex2: 0.0,
            status1: Status::Initial,
            status2: Status::Initial,
            preserve_x_scale: true,
        }
    }

    pub fn reset(&mut self) {
        self.src_vertices1.remove_all();
        self.src_vertices2.remove_all();
        self.kindex1 = 0.0;
        self.kindex2 = 0.0;
        self.status1 = Status::Initial;
        self.status2 = Status::Initial;
    }

    pub fn move_to1(&mut self, x: f64, y: f64) {
        if self.status1 == Status::Initial {
            self.src_vertices1.modify_last(VertexDist::new(x, y));
            self.status1 = Status::MakingPath;
        } else {
            self.line_to1(x, y);
        }
    }

    pub fn line_to1(&mut self, x: f64, y: f64) {
        if self.status1 == Status::MakingPath {
            self.src_vertices1.add(VertexDist::new(x, y));
        }
    }

    pub fn move_to2(&mut self, x: f64, y: f64) {
        if self.status2 == Status::Initial {
            self.src_vertices2.modify_last(VertexDist::new(x, y));
            self.status2 = Status::MakingPath;
        } else {
            self.line_to2(x, y);
        }
    }

    pub fn line_to2(&mut self, x: f64, y: f64) {
        if self.status2 == Status::MakingPath {
            self.src_vertices2.add(VertexDist::new(x, y));
        }
    }

    fn finalize_path(vertices: &mut VertexStorage) -> f64 {
        let mut dist: f64;
        let d: f64;

        vertices.close(false);
        if vertices.size() > 2 {
            if vertices[vertices.size() - 2].dist * 10.0 < vertices[vertices.size() - 3].dist {
                d = vertices[vertices.size() - 3].dist + vertices[vertices.size() - 2].dist;
                let mut len = vertices.size();
                vertices[len - 2] = vertices[len - 1];

                vertices.remove_last();
                len = vertices.size();
                vertices[len - 2].dist = d;
            }
        }

        dist = 0.0;
        for i in 0..vertices.size() {
            let v = &mut vertices[i];
            let d = v.dist;
            v.dist = dist;
            dist += d;
        }
        (vertices.size() - 1) as f64 / dist
    }

    pub fn total_length1(&self) -> f64 {
        if self.base_length >= 1e-10 {
            return self.base_length;
        }
        if self.status1 == Status::Ready {
            return self.src_vertices1[self.src_vertices1.size() - 1].dist;
        }
        return 0.0;
    }

    pub fn total_length2(&self) -> f64 {
        if self.base_length >= 1e-10 {
            return self.base_length;
        }
        if self.status1 == Status::Ready {
            return self.src_vertices2[self.src_vertices2.size() - 1].dist;
        }
        return 0.0;
    }

    pub fn set_base_length(&mut self, v: f64) {
        self.base_length = v;
    }

    pub fn base_length(&self) -> f64 {
        self.base_length
    }

    pub fn set_base_height(&mut self, v: f64) {
        self.base_height = v;
    }

    pub fn base_height(&self) -> f64 {
        self.base_height
    }

    pub fn set_preserve_x_scale(&mut self, f: bool) {
        self.preserve_x_scale = f;
    }

    pub fn preserve_x_scale(&self) -> bool {
        self.preserve_x_scale
    }

    pub fn add_paths<VS1: VertexSource, VS2: VertexSource>(
        &mut self, vs1: &mut VS1, vs2: &mut VS2, path1_id: u32, path2_id: u32,
    ) {
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;

        let mut cmd: u32;

        vs1.rewind(path1_id);
        loop {
            cmd = vs1.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_move_to(cmd) {
                self.move_to1(x, y);
            } else {
                if is_vertex(cmd) {
                    self.line_to1(x, y);
                }
            }
        }

        vs2.rewind(path2_id);
        loop {
            cmd = vs2.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_move_to(cmd) {
                self.move_to2(x, y);
            } else {
                if is_vertex(cmd) {
                    self.line_to2(x, y);
                }
            }
        }
        self.finalize_paths();
    }

    pub fn finalize_paths(&mut self) {
        if self.status1 == Status::MakingPath
            && self.src_vertices1.len() > 1
            && self.status2 == Status::MakingPath
            && self.src_vertices2.len() > 1
        {
            self.kindex1 = Self::finalize_path(&mut self.src_vertices1);
            self.kindex2 = Self::finalize_path(&mut self.src_vertices2);
            self.status1 = Status::Ready;
            self.status2 = Status::Ready;
        }
    }

    pub fn transform1(
        &self, vertices: &VertexStorage, kindex: f64, kx: f64, x: &mut f64, y: &mut f64,
    ) {
        let x1;
        let y1;
        let dx;
        let dy;
        let mut d;
        let dd;
        *x *= kx;
        if *x < 0.0 {
            // Extrapolation on the left
            //--------------------------
            x1 = vertices[0].x;
            y1 = vertices[0].y;
            dx = vertices[1].x - x1;
            dy = vertices[1].y - y1;
            dd = vertices[1].dist - vertices[0].dist;
            d = *x;
        } else if *x > vertices[vertices.len() - 1].dist {
            // Extrapolation on the right
            //--------------------------
            let i = vertices.len() - 2;
            let j = vertices.len() - 1;
            x1 = vertices[j].x;
            y1 = vertices[j].y;
            dx = x1 - vertices[i].x;
            dy = y1 - vertices[i].y;
            dd = vertices[j].dist - vertices[i].dist;
            d = *x - vertices[j].dist;
        } else {
            // Interpolation
            //--------------------------
            let mut i = 0;
            let mut j = vertices.len() - 1;
            if self.preserve_x_scale {
                let mut k;
                while (j - i) > 1 {
                    k = (i + j) >> 1;
                    if *x < vertices[k as usize].dist {
                        j = k;
                    } else {
                        i = k;
                    }
                }
                d = vertices[i].dist;
                dd = vertices[j].dist - d;
                d = *x - d;
            } else {
                i = (*x * kindex) as usize;
                j = i + 1;
                dd = vertices[j].dist - vertices[i].dist;
                d = ((*x * kindex) - i as f64) * dd;
            }
            x1 = vertices[i].x;
            y1 = vertices[i].y;
            dx = vertices[j].x - x1;
            dy = vertices[j].y - y1;
        }
        *x = x1 + dx * d / dd;
        *y = y1 + dy * d / dd;
    }
}

impl Transformer for TransDoublePath {
    fn transform(&self, x: &mut f64, y: &mut f64) {
        if self.status1 == Status::Ready && self.status2 == Status::Ready {
            if self.base_length > 1e-10 {
                *x *= self.src_vertices1[self.src_vertices1.len() - 1].dist / self.base_length;
            }
            let mut x1 = *x;
            let mut y1 = *y;
            let mut x2 = *x;
            let mut y2 = *y;
            let dd = self.src_vertices2[self.src_vertices2.len() - 1].dist
                / self.src_vertices1[self.src_vertices1.len() - 1].dist;
            self.transform1(&self.src_vertices1, self.kindex1, 1.0, &mut x1, &mut y1);
            self.transform1(&self.src_vertices2, self.kindex2, dd, &mut x2, &mut y2);
            *x = x1 + *y * (x2 - x1) / self.base_height;
            *y = y1 + *y * (y2 - y1) / self.base_height;
        }
    }

    fn scaling_abs(&self, x: &mut f64, y: &mut f64) {
        *x = 0.;
        *y = 0.;
        todo!()
    }
}

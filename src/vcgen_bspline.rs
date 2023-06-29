use crate::array::{VecPodB, PodBVector};
use crate::basics::{get_close_flag, is_move_to, is_stop, is_vertex, PathCmd, PointD};
use crate::bspline::Bspline;
use crate::{Generator, Point, VertexSource};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Initial,
    Ready,
    Polygon,
    EndPoly,
    Stop,
}

pub type VertexStorage = VecPodB<PointD>;

pub struct VcgenBspline {
    src_vertices: VertexStorage,
    spline_x: Bspline,
    spline_y: Bspline,
    interpolation_step: f64,
    closed: u32,
    status: Status,
    src_vertex: u32,
    cur_abscissa: f64,
    max_abscissa: f64,
}

impl Generator for VcgenBspline {
    fn new() -> Self {
        VcgenBspline {
            src_vertices: VecPodB::new(),
            spline_x: Bspline::new(),
            spline_y: Bspline::new(),
            interpolation_step: 1.0 / 50.0,
            closed: 0,
            status: Status::Initial,
            src_vertex: 0,
            cur_abscissa: 0.0,
            max_abscissa: 0.0,
        }
    }

    fn remove_all(&mut self) {
        self.src_vertices.clear();
        self.closed = 0;
        self.status = Status::Initial;
        self.src_vertex = 0;
    }

    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        self.status = Status::Initial;
        if is_move_to(cmd) {
            self.src_vertices.modify_last(PointD::new(x, y));
        } else {
            if is_vertex(cmd) {
                self.src_vertices.push(PointD::new(x, y));
            } else {
                self.closed = get_close_flag(cmd);
            }
        }
    }
}

impl VertexSource for VcgenBspline {
    fn rewind(&mut self, _: u32) {
        self.cur_abscissa = 0.0;
        self.max_abscissa = 0.0;
        self.src_vertex = 0;
        if self.status == Status::Initial && self.src_vertices.len() > 2 {
            if self.closed != 0 {
                self.spline_x.init(self.src_vertices.len() + 8);
                self.spline_y.init(self.src_vertices.len() + 8);
                self.spline_x
                    .add_point(0.0, self.src_vertices.prev(self.src_vertices.len() - 3).x);
                self.spline_y
                    .add_point(0.0, self.src_vertices.prev(self.src_vertices.len() - 3).y);
                self.spline_x
                    .add_point(1.0, self.src_vertices[self.src_vertices.len() - 3].x);
                self.spline_y
                    .add_point(1.0, self.src_vertices[self.src_vertices.len() - 3].y);
                self.spline_x
                    .add_point(2.0, self.src_vertices[self.src_vertices.len() - 2].x);
                self.spline_y
                    .add_point(2.0, self.src_vertices[self.src_vertices.len() - 2].y);
                self.spline_x
                    .add_point(3.0, self.src_vertices[self.src_vertices.len() - 1].x);
                self.spline_y
                    .add_point(3.0, self.src_vertices[self.src_vertices.len() - 1].y);
            } else {
                self.spline_x.init(self.src_vertices.len());
                self.spline_y.init(self.src_vertices.len());
            }
            for i in 0..self.src_vertices.len() {
                let x = if self.closed != 0 { i + 4 } else { i } as f64;
                self.spline_x.add_point(x, self.src_vertices[i].x);
                self.spline_y.add_point(x, self.src_vertices[i].y);
            }
            self.cur_abscissa = 0.0;
            self.max_abscissa = (self.src_vertices.len() - 1) as f64;
            if self.closed != 0 {
                self.cur_abscissa = 4.0;
                self.max_abscissa += 5.0;
                self.spline_x
                    .add_point((self.src_vertices.len() + 4) as f64, self.src_vertices[0].x);
                self.spline_y
                    .add_point((self.src_vertices.len() + 4) as f64, self.src_vertices[0].y);
                self.spline_x
                    .add_point((self.src_vertices.len() + 5) as f64, self.src_vertices[1].x);
                self.spline_y
                    .add_point((self.src_vertices.len() + 5) as f64, self.src_vertices[1].y);
                self.spline_x
                    .add_point((self.src_vertices.len() + 6) as f64, self.src_vertices[2].x);
                self.spline_y
                    .add_point((self.src_vertices.len() + 6) as f64, self.src_vertices[2].y);
                self.spline_x.add_point(
                    (self.src_vertices.len() + 7) as f64,
                    self.src_vertices.next(2).x,
                );
                self.spline_y.add_point(
                    (self.src_vertices.len() + 7) as f64,
                    self.src_vertices.next(2).y,
                );
            }
            self.spline_x.prepare();
            self.spline_y.prepare();
        }
        self.status = Status::Ready;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::LineTo;
        while !is_stop(cmd as u32) {
            loop {
                match self.status {
                    Status::Initial => {
                        self.rewind(0);
                        self.status = Status::Ready;
                    }
                    Status::Ready => {
                        if self.src_vertices.len() < 2 {
                            cmd = PathCmd::Stop;
                            break;
                        }
                        if self.src_vertices.len() == 2 {
                            *x = self.src_vertices[self.src_vertex as usize].x;
                            *y = self.src_vertices[self.src_vertex as usize].y;
                            self.src_vertex += 1;
                            if self.src_vertex == 1 {
                                return PathCmd::MoveTo as u32;
                            }
                            if self.src_vertex == 2 {
                                return PathCmd::LineTo as u32;
                            }
                            cmd = PathCmd::Stop;
                            break;
                        }
                        cmd = PathCmd::MoveTo;
                        self.status = Status::Polygon;
                        self.src_vertex = 0;
                    }
                    Status::Polygon => {
                        if self.cur_abscissa >= self.max_abscissa {
                            if self.closed != 0 {
                                self.status = Status::EndPoly;
                                break;
                            } else {
                                *x = self.src_vertices[self.src_vertices.len() - 1].x;
                                *y = self.src_vertices[self.src_vertices.len() - 1].y;
                                self.status = Status::EndPoly;
                                return PathCmd::LineTo as u32;
                            }
                        }
                        *x = self.spline_x.get_stateful(self.cur_abscissa);
                        *y = self.spline_y.get_stateful(self.cur_abscissa);
                        self.src_vertex += 1;
                        self.cur_abscissa += self.interpolation_step;
                        return if self.src_vertex == 1 {
                            PathCmd::MoveTo as u32
                        } else {
                            PathCmd::LineTo as u32
                        };
                    }
                    Status::EndPoly => {
                        self.status = Status::Stop;
                        return PathCmd::EndPoly as u32 | self.closed;
                    }
                    Status::Stop => {
                        return PathCmd::Stop as u32;
                    }
                }
            }
        }
        cmd as u32
    }
}

impl VcgenBspline {
    pub fn interpolation_step(&self) -> f64 {
        self.interpolation_step
    }

    pub fn set_interpolation_step(&mut self, v: f64) {
        self.interpolation_step = v;
    }
}

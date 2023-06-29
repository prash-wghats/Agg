use crate::basics::{is_stop, PathCmd};
use crate::trans_affine::TransAffine;
use crate::{Equiv, Transformer, VertexSource, VertexSourceWithMarker};
#[derive(PartialEq)]
enum Status {
    Initial,
    Markers,
    Polygon,
}

pub struct ConvMarkerConcat<'a, M: VertexSourceWithMarker, S: VertexSource> {
    marker_locator_src: Equiv<'a, M>,
    marker_shapes: Equiv<'a, S>,
    transform: TransAffine,
    mtx: TransAffine,
    status: Status,
    cur_src: i32,
    marker: u32,
    num_markers: u32,
}

impl<'a, M: VertexSourceWithMarker, S: VertexSource> ConvMarkerConcat<'a, M, S> {
    pub fn new_owned(ml: M, ms: S) -> Self {
        ConvMarkerConcat {
            marker_locator_src: Equiv::Own(ml),
            marker_shapes: Equiv::Own(ms),
            transform: TransAffine::new_default(),
            mtx: TransAffine::new_default(),
            status: Status::Initial,
            marker: 0,
            num_markers: 1,
            cur_src: 0,
        }
    }

    pub fn new_borrowed(ml: &'a mut M, ms: &'a mut S) -> Self {
        ConvMarkerConcat {
            marker_locator_src: Equiv::Brw(ml),
            marker_shapes: Equiv::Brw(ms),
            transform: TransAffine::new_default(),
            mtx: TransAffine::new_default(),
            status: Status::Initial,
            marker: 0,
            num_markers: 1,
            cur_src: 0,
        }
    }

    pub fn transform_mut(&mut self) -> &mut TransAffine {
        &mut self.transform
    }

    pub fn transform(&self) -> &TransAffine {
        &self.transform
    }

    pub fn locator_mut(&mut self) -> &mut M {
        &mut self.marker_locator_src
    }

    pub fn shapes_mut(&mut self) -> &mut S {
        &mut self.marker_shapes
    }

    pub fn shapes(&self) -> &S {
        &self.marker_shapes
    }

    fn vertex_mrk(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::MoveTo as u32;
        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut x2 = 0.0;
        let mut y2 = 0.0;

        loop {
            if cmd == PathCmd::Stop as u32 {
                break;
            }
            match self.status {
                Status::Initial => {
                    if self.num_markers == 0 {
                        cmd = PathCmd::Stop as u32;
                    } else {
                        self.marker_locator_src.markers_mut().rewind(self.marker);
                        self.marker += 1;
                        self.num_markers = 0;
                        self.status = Status::Markers;
                    }
                }
                Status::Markers => {
                    let cmd_ = self
                        .marker_locator_src
                        .markers_mut()
                        .vertex(&mut x1, &mut y1);
                    if cmd_ == PathCmd::Stop as u32 {
                        self.status = Status::Initial;
                    } else {
                        let cmd_ = self
                            .marker_locator_src
                            .markers_mut()
                            .vertex(&mut x2, &mut y2);
                        if cmd_ == PathCmd::Stop as u32 {
                            self.status = Status::Initial;
                        } else {
                            self.num_markers += 1;
                            self.mtx = self.transform;
                            self.mtx *=
                                TransAffine::trans_affine_rotation((y2 - y1).atan2(x2 - x1));
                            self.mtx *= TransAffine::trans_affine_translation(x1, y1);
                            self.marker_shapes.rewind(self.marker - 1);
                            self.status = Status::Polygon;
                        }
                    }
                }
                Status::Polygon => {
                    cmd = self.marker_shapes.vertex(x, y);
                    if cmd == PathCmd::Stop as u32 {
                        cmd = PathCmd::MoveTo as u32;
                        self.status = Status::Markers;
                    } else {
                        self.mtx.transform(x, y);
                        return cmd;
                    }
                }
            }
        }
        cmd
    }
}

impl<'a, M: VertexSourceWithMarker, S: VertexSource> VertexSource for ConvMarkerConcat<'a, M, S> {
    fn rewind(&mut self, path_id: u32) {
        self.marker_locator_src.rewind(path_id);
        self.status = Status::Initial;
        self.marker = 0;
        self.num_markers = 1;
        self.cur_src = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd;
        if self.cur_src == 0 {
            cmd = self.marker_locator_src.vertex(x, y);
            if !is_stop(cmd) {
                return cmd;
            }
            self.cur_src = 1;
        }
        if self.cur_src == 1 {
            cmd = self.vertex_mrk(x, y);
            if !is_stop(cmd) {
                return cmd;
            }
            self.cur_src = 2;
        }
        return PathCmd::Stop as u32;
    }
}

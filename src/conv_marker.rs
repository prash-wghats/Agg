use crate::basics::PathCmd;
use crate::trans_affine::TransAffine;
use crate::{Equiv, Transformer, VertexSource};
#[derive(PartialEq)]
enum Status {
    Initial,
    Markers,
    Polygon,
}

pub struct ConvMarker<'a, M: VertexSource, S: VertexSource> {
    marker_locator: Equiv<'a, M>,
    marker_shapes: Equiv<'a, S>,
    transform: TransAffine,
    mtx: TransAffine,
    status: Status,
    marker: u32,
    num_markers: u32,
}

impl<'a, M: VertexSource, S: VertexSource> ConvMarker<'a, M, S> {
    pub fn new_borrowed(ml: &'a mut M, ms: &'a mut S) -> Self {
        ConvMarker {
            marker_locator: Equiv::Brw(ml),
            marker_shapes: Equiv::Brw(ms),
            transform: TransAffine::new_default(),
            mtx: TransAffine::new_default(),
            status: Status::Initial,
            marker: 0,
            num_markers: 1,
        }
    }

    pub fn new_owned(ml: M, ms: S) -> Self {
        ConvMarker {
            marker_locator: Equiv::Own(ml),
            marker_shapes: Equiv::Own(ms),
            transform: TransAffine::new_default(),
            mtx: TransAffine::new_default(),
            status: Status::Initial,
            marker: 0,
            num_markers: 1,
        }
    }

    pub fn transform_mut(&mut self) -> &mut TransAffine {
        &mut self.transform
    }

    pub fn transform(&self) -> &TransAffine {
        &self.transform
    }

    pub fn locator_mut(&mut self) -> &mut M {
        &mut self.marker_locator
    }

    pub fn shapes_mut(&mut self) -> &mut S {
        &mut self.marker_shapes
    }

    pub fn shapes(&self) -> &S {
        &self.marker_shapes
    }
}

impl<'a, M: VertexSource, S: VertexSource> VertexSource for ConvMarker<'a, M, S> {
    fn rewind(&mut self, _path_id: u32) {
        self.status = Status::Initial;
        self.marker = 0;
        self.num_markers = 1;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
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
                        self.marker_locator.rewind(self.marker);
                        self.marker += 1;
                        self.num_markers = 0;
                        self.status = Status::Markers;
                    }
                }
                Status::Markers => {
                    let cmd_ = self.marker_locator.vertex(&mut x1, &mut y1);
                    if cmd_ == PathCmd::Stop as u32 {
                        self.status = Status::Initial;
                    } else {
                        let cmd_ = self.marker_locator.vertex(&mut x2, &mut y2);
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

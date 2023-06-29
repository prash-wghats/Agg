use crate::array::{PodBVector, VecPodB};
use crate::basics::{PathCmd, is_move_to, is_vertex};
use crate::{Markers, Generator, VertexSource};
/// XXXX TESTING
//======================================================VcgenMarkersTerm
//
// Terminal markers generator (arrowhead/arrowtail)
//

pub struct VcgenMarkersTerm {
    curr_id: u32,
    curr_idx: u32,
    markers: VecPodB<CoordPoint>,
}
impl Markers for VcgenMarkersTerm {}
impl Generator for VcgenMarkersTerm {
    fn new() -> VcgenMarkersTerm {
        VcgenMarkersTerm {
            curr_id: 0,
            curr_idx: 0,
            markers: Vec::new(),
        }
    }

    fn remove_all(&mut self) {
        self.markers.clear();
    }

    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        if is_move_to(cmd) {
            if self.markers.len() & 1 == 1 {
                // Initial state, the first coordinate was added.
                // If two of more calls of start_vertex() occures
                // we just modify the last one.
                self.markers.modify_last(CoordPoint { x, y })
            } else {
                self.markers.push(CoordPoint { x: x, y: y });
            }
        } else {
            if is_vertex(cmd) {
                if self.markers.len() & 1 == 1 {
                    // Initial state, the first coordinate was added.
                    // Add three more points, 0,1,1,0
                    self.markers.push(CoordPoint { x: x, y: y });
                    self.markers.push(self.markers[self.markers.len() - 1]);
                    self.markers.push(self.markers[self.markers.len() - 3]);
                } else {
                    let len = self.markers.len();
                    if len > 0 {
                        // Replace two last points: 0,1,1,0 -> 0,1,2,1
                        self.markers[len - 1] = self.markers[self.markers.len() - 2];
                        self.markers[len - 2] = CoordPoint { x: x, y: y };
                    }
                }
            }
        }
    }
}

impl VertexSource for VcgenMarkersTerm {
    fn rewind(&mut self, path_id: u32) {
        self.curr_id = path_id * 2;
        self.curr_idx = self.curr_id;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.curr_id > 2 || self.curr_idx as usize >= self.markers.len() {
            return PathCmd::Stop as u32;
        }
        let c = &self.markers[self.curr_idx as usize];
        *x = c.x;
        *y = c.y;
        if self.curr_idx & 1 == 1 {
            self.curr_idx += 3;
            return PathCmd::LineTo as u32;
        }
        self.curr_idx += 1;
        return PathCmd::MoveTo as u32;
    }
}

#[derive(Clone, Copy)]
pub struct CoordPoint {
    pub x: f64,
    pub y: f64,
}

impl CoordPoint {
    pub fn new() -> CoordPoint {
        CoordPoint { x: 0.0, y: 0.0 }
    }

    pub fn new_xy(x: f64, y: f64) -> CoordPoint {
        CoordPoint { x: x, y: y }
    }
}

use crate::VertexSource;
use crate::array::*;
use crate::basics::*;
use crate::basics::{PathCmd, PathFlag};
use crate::shorten_path::*;
use crate::vertex_sequence::{VecSequence, VertexDist};
use crate::{Generator, VertexSequence};

// NOT TESTED

pub type VertexStorage = VecSequence<VertexDist>;

//===================================================VcgenVertexSequence
pub struct VcgenVertexSequence {
    src_vertices: VertexStorage,
    flags: u32,
    cur_vertex: u32,
    shorten: f64,
    ready: bool,
}

impl VcgenVertexSequence {
	pub fn set_shorten(&mut self, s: f64) {
        self.shorten = s;
    }
    pub fn shorten(&self) -> f64 {
        self.shorten
    }
}

impl Generator for VcgenVertexSequence {
	fn new() -> VcgenVertexSequence {
        VcgenVertexSequence {
            src_vertices: VertexStorage::new(),
            flags: 0,
            cur_vertex: 0,
            shorten: 0.0,
            ready: false,
        }
    }

    // Vertex Generator Interface
    fn remove_all(&mut self) {
        self.ready = false;
        self.src_vertices.remove_all();
        self.cur_vertex = 0;
        self.flags = 0;
    }

    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        self.ready = false;
        if is_move_to(cmd) {
            self.src_vertices
                .modify_last(VertexDist::new_with_cmd(x, y, cmd));
        } else {
            if is_vertex(cmd) {
                self.src_vertices.add(VertexDist::new_with_cmd(x, y, cmd));
            } else {
                self.flags = cmd & PathFlag::Mask as u32;
            }
        }
    }
}

impl VertexSource for VcgenVertexSequence {
    // Vertex Source Interface
    fn rewind(&mut self, _path_id: u32) {
        if !self.ready {
            self.src_vertices.close(is_closed(self.flags));
            shorten_path(
                &mut self.src_vertices,
                self.shorten,
                get_close_flag(self.flags),
            );
        }
        self.ready = true;
        self.cur_vertex = 0;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if !self.ready {
            self.rewind(0);
        }

        if self.cur_vertex == self.src_vertices.len() as u32 {
            self.cur_vertex += 1;
            return PathCmd::EndPoly as u32 | self.flags;
        }

        if self.cur_vertex > self.src_vertices.len() as u32 {
            return PathCmd::Stop as u32;
        }

        let v = &mut self.src_vertices[self.cur_vertex as usize];
        *x = v.x;
        *y = v.y;
        self.cur_vertex += 1;
        v.cmd
    }
}


use crate::basics::{PathCmd, PathFlag};
use crate::VertexSource;

/// XXXXX NEEDS TESTING

pub struct Arrowhead {
    head_d1: f64,
    head_d2: f64,
    head_d3: f64,
    head_d4: f64,
    tail_d1: f64,
    tail_d2: f64,
    tail_d3: f64,
    tail_d4: f64,
    head_flag: bool,
    tail_flag: bool,
    curr_id: usize,
    curr_coord: usize,
    coord: [f64; 16],
    cmd: [u32; 8],
}

impl Arrowhead {
    pub fn new() -> Arrowhead {
        Arrowhead {
            head_d1: 1.0,
            head_d2: 1.0,
            head_d3: 1.0,
            head_d4: 0.0,
            tail_d1: 1.0,
            tail_d2: 1.0,
            tail_d3: 1.0,
            tail_d4: 0.0,
            head_flag: false,
            tail_flag: false,
            curr_id: 0,
            curr_coord: 0,
            coord: [0.0; 16],
            cmd: [0; 8],
        }
    }

    pub fn head(&mut self, d1: f64, d2: f64, d3: f64, d4: f64) {
        self.head_d1 = d1;
        self.head_d2 = d2;
        self.head_d3 = d3;
        self.head_d4 = d4;
        self.head_flag = true;
    }

    pub fn head_(&mut self) {
        self.head_flag = true;
    }

    pub fn no_head(&mut self) {
        self.head_flag = false;
    }

    pub fn tail(&mut self, d1: f64, d2: f64, d3: f64, d4: f64) {
        self.tail_d1 = d1;
        self.tail_d2 = d2;
        self.tail_d3 = d3;
        self.tail_d4 = d4;
        self.tail_flag = true;
    }

    pub fn tail_(&mut self) {
        self.tail_flag = true;
    }

    pub fn no_tail(&mut self) {
        self.tail_flag = false;
    }
}

impl  VertexSource for Arrowhead {
	fn rewind(&mut self, path_id: u32) {
        self.curr_id = path_id as usize;
        self.curr_coord = 0;
        if path_id == 0 {
            if !self.tail_flag {
                self.cmd[0] = PathCmd::Stop as u32;
                return;
            }
            self.coord[0] = self.tail_d1;
            self.coord[1] = 0.0;
            self.coord[2] = self.tail_d1 - self.tail_d4;
            self.coord[3] = self.tail_d3;
            self.coord[4] = -self.tail_d2 - self.tail_d4;
            self.coord[5] = self.tail_d3;
            self.coord[6] = -self.tail_d2;
            self.coord[7] = 0.0;
            self.coord[8] = -self.tail_d2 - self.tail_d4;
            self.coord[9] = -self.tail_d3;
            self.coord[10] = self.tail_d1 - self.tail_d4;
            self.coord[11] = -self.tail_d3;

            self.cmd[0] = PathCmd::MoveTo as u32;
            self.cmd[1] = PathCmd::LineTo as u32;
            self.cmd[2] = PathCmd::LineTo as u32;
            self.cmd[3] = PathCmd::LineTo as u32;
            self.cmd[4] = PathCmd::LineTo as u32;
            self.cmd[5] = PathCmd::LineTo as u32;
            self.cmd[7] = PathCmd::EndPoly as u32 | PathFlag::Close as u32 | PathFlag::Ccw as u32;
            self.cmd[6] = PathCmd::Stop as u32;
            return;
        }

        if path_id == 1 {
            if !self.head_flag {
                self.cmd[0] = PathCmd::Stop as u32;
                return;
            }
            self.coord[0] = -self.head_d1;
            self.coord[1] = 0.0;
            self.coord[2] = self.head_d2 + self.head_d4;
            self.coord[3] = -self.head_d3;
            self.coord[4] = self.head_d2;
            self.coord[5] = 0.0;
            self.coord[6] = self.head_d2 + self.head_d4;
            self.coord[7] = self.head_d3;

            self.cmd[0] = PathCmd::MoveTo as u32;
            self.cmd[1] = PathCmd::LineTo as u32;
            self.cmd[2] = PathCmd::LineTo as u32;
            self.cmd[3] = PathCmd::LineTo as u32;
            self.cmd[4] = PathCmd::EndPoly as u32 | PathFlag::Close as u32 | PathFlag::Ccw as u32;
            self.cmd[5] = PathCmd::Stop as u32;
            return;
        }
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.curr_id < 2 {
            let curr_idx = self.curr_coord * 2;
            *x = self.coord[curr_idx];
            *y = self.coord[curr_idx + 1];
            self.curr_coord += 1;
            return self.cmd[self.curr_coord - 1];
        }
        return PathCmd::Stop as u32;
    }
}
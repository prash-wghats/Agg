use crate::basics::{is_stop, PathCmd};
use crate::VertexSource;
use std::f64::consts::*;

pub struct Arc {
    x: f64,
    y: f64,
    rx: f64,
    ry: f64,
    angle: f64,
    start: f64,
    end: f64,
    scale: f64,
    da: f64,
    ccw: bool,
    initialized: bool,
    path_cmd: u32,
}

impl Arc {
    pub fn new_default() -> Arc {
        Arc {
            x: 0.0,
            y: 0.0,
            rx: 0.0,
            ry: 0.0,
            angle: 0.0,
            start: 0.0,
            end: 0.0,
            scale: 1.0,
            da: 0.0,
            ccw: false,
            initialized: false,
            path_cmd: 0,
        }
    }

    pub fn approximation_scale(&self) -> f64 {
        self.scale
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.scale = s;
        if self.initialized {
            self.normalize(self.start, self.end, self.ccw);
        }
    }

    pub fn new(x: f64, y: f64, rx: f64, ry: f64, a1: f64, a2: f64, ccw: bool) -> Self {
        let mut a = Self {
            x: x,
            y: y,
            rx: rx,
            ry: ry,
            scale: 1.0,
            angle: 0.0,
            start: 0.0,
            end: 0.0,
            da: 0.0,
            ccw: false,
            initialized: false,
            path_cmd: 0,
        };
        a.normalize(a1, a2, ccw);
        a
    }

    pub fn init(&mut self, x: f64, y: f64, rx: f64, ry: f64, a1: f64, a2: f64, ccw: bool) {
        self.x = x;
        self.y = y;
        self.rx = rx;
        self.ry = ry;
        self.normalize(a1, a2, ccw);
    }

    fn normalize(&mut self, a1: f64, a2: f64, ccw: bool) {
        let (mut a1, mut a2) = (a1, a2);
        let ra = (self.rx.abs() + self.ry.abs()) / 2.0;
        self.da = (ra / (ra + 0.125 / self.scale)).acos() * 2.0;
        if ccw {
            while a2 < a1 {
                a2 += PI * 2.0;
            }
        } else {
            while a1 < a2 {
                a1 += PI * 2.0;
            }
            self.da = -self.da;
        }
        self.ccw = ccw;
        self.start = a1;
        self.end = a2;
        self.initialized = true;
    }
}

impl VertexSource for Arc {
    fn rewind(&mut self, _: u32) {
        self.path_cmd = PathCmd::MoveTo as u32;
        self.angle = self.start;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if is_stop(self.path_cmd) {
            return PathCmd::Stop as u32;
        }
        if (self.angle < self.end - self.da / 4.0) != self.ccw {
            *x = self.x + self.end.cos() * self.rx;
            *y = self.y + self.end.sin() * self.ry;
            self.path_cmd = PathCmd::Stop as u32;
            return PathCmd::LineTo as u32;
        }

        *x = self.x + self.angle.cos() * self.rx;
        *y = self.y + self.angle.sin() * self.ry;

        self.angle += self.da;

        let pf = self.path_cmd;
        self.path_cmd = PathCmd::LineTo as u32;
        pf
    }
}

use crate::arc::Arc;
use crate::basics::{is_stop, PathCmd, PathFlag};
use crate::VertexSource;

pub struct RoundedRect {
    m_x1: f64,
    m_y1: f64,
    m_x2: f64,
    m_y2: f64,
    m_rx1: f64,
    m_ry1: f64,
    m_rx2: f64,
    m_ry2: f64,
    m_rx3: f64,
    m_ry3: f64,
    m_rx4: f64,
    m_ry4: f64,
    m_status: u32,
    m_arc: Arc,
}

impl RoundedRect {
    pub fn set_approximation_scale(&mut self, s: f64) {
        self.m_arc.set_approximation_scale(s);
    }

    pub fn approximation_scale(&self) -> f64 {
        self.m_arc.approximation_scale()
    }

    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, r: f64) -> RoundedRect {
        let mut rr = RoundedRect {
            m_x1: x1,
            m_y1: y1,
            m_x2: x2,
            m_y2: y2,
            m_rx1: r,
            m_ry1: r,
            m_rx2: r,
            m_ry2: r,
            m_rx3: r,
            m_ry3: r,
            m_rx4: r,
            m_ry4: r,
            m_status: 0,
            m_arc: Arc::new_default(),
        };
        if x1 > x2 {
            rr.m_x1 = x2;
            rr.m_x2 = x1;
        }
        if y1 > y2 {
            rr.m_y1 = y2;
            rr.m_y2 = y1;
        }
        rr
    }

    pub fn rect(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.m_x1 = x1;
        self.m_y1 = y1;
        self.m_x2 = x2;
        self.m_y2 = y2;
        if x1 > x2 {
            self.m_x1 = x2;
            self.m_x2 = x1;
        }
        if y1 > y2 {
            self.m_y1 = y2;
            self.m_y2 = y1;
        }
    }

    pub fn radius(&mut self, r: f64) {
        (
            self.m_rx1, self.m_ry1, self.m_rx2, self.m_ry2, self.m_rx3, self.m_ry3, self.m_rx4,
            self.m_ry4,
        ) = (r, r, r, r, r, r, r, r);
    }

    pub fn radius_xy(&mut self, rx: f64, ry: f64) {
        (self.m_rx1, self.m_rx2, self.m_rx3, self.m_rx4) = (rx, rx, rx, rx);
        (self.m_ry1, self.m_ry2, self.m_ry3, self.m_ry4) = (ry, ry, ry, ry);
    }

    pub fn radius_bottom_top(&mut self, rx_bottom: f64, ry_bottom: f64, rx_top: f64, ry_top: f64) {
        (self.m_rx1, self.m_rx2) = (rx_bottom, rx_bottom);
        (self.m_rx3, self.m_rx4) = (rx_top, rx_top);
        (self.m_ry1, self.m_ry2) = (ry_bottom, ry_bottom);
        (self.m_ry3, self.m_ry4) = (ry_top, ry_top);
    }

    pub fn radius_all(
        &mut self, rx1: f64, ry1: f64, rx2: f64, ry2: f64, rx3: f64, ry3: f64, rx4: f64, ry4: f64,
    ) {
        self.m_rx1 = rx1;
        self.m_ry1 = ry1;
        self.m_rx2 = rx2;
        self.m_ry2 = ry2;
        self.m_rx3 = rx3;
        self.m_ry3 = ry3;
        self.m_rx4 = rx4;
        self.m_ry4 = ry4;
    }

    pub fn normalize_radius(&mut self) {
        let dx = (self.m_y2 - self.m_y1).abs();
        let dy = (self.m_x2 - self.m_x1).abs();

        let mut k = 1.0;
        let mut t;
        t = dx / (self.m_rx1 + self.m_rx2);
        if t < k {
            k = t;
        }
        t = dx / (self.m_rx3 + self.m_rx4);
        if t < k {
            k = t;
        }
        t = dy / (self.m_ry1 + self.m_ry2);
        if t < k {
            k = t;
        }
        t = dy / (self.m_ry3 + self.m_ry4);
        if t < k {
            k = t;
        }

        if k < 1.0 {
            self.m_rx1 *= k;
            self.m_ry1 *= k;
            self.m_rx2 *= k;
            self.m_ry2 *= k;
            self.m_rx3 *= k;
            self.m_ry3 *= k;
            self.m_rx4 *= k;
            self.m_ry4 *= k;
        }
    }
}
impl VertexSource for RoundedRect {
    fn rewind(&mut self, _: u32) {
        self.m_status = 0;
    }

    //--------------------------------------------------------------------
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::Stop as u32;
        loop {
            match self.m_status {
                0 => {
                    self.m_arc.init(
                        self.m_x1 + self.m_rx1,
                        self.m_y1 + self.m_ry1,
                        self.m_rx1,
                        self.m_ry1,
                        std::f64::consts::PI,
                        std::f64::consts::PI + std::f64::consts::PI * 0.5,
                        true,
                    );
                    self.m_arc.rewind(0);
                    self.m_status += 1;
                }
                1 => {
                    cmd = self.m_arc.vertex(x, y);
                    if is_stop(cmd) {
                        self.m_status += 1;
                    } else {
                        return cmd;
                    }
                }
                2 => {
                    self.m_arc.init(
                        self.m_x2 - self.m_rx2,
                        self.m_y1 + self.m_ry2,
                        self.m_rx2,
                        self.m_ry2,
                        std::f64::consts::PI + std::f64::consts::PI * 0.5,
                        0.0,
                        true,
                    );
                    self.m_arc.rewind(0);
                    self.m_status += 1;
                }
                3 => {
                    cmd = self.m_arc.vertex(x, y);
                    if is_stop(cmd) {
                        self.m_status += 1;
                    } else {
                        return PathCmd::LineTo as u32;
                    }
                }
                4 => {
                    self.m_arc.init(
                        self.m_x2 - self.m_rx3,
                        self.m_y2 - self.m_ry3,
                        self.m_rx3,
                        self.m_ry3,
                        0.0,
                        std::f64::consts::PI * 0.5,
                        true,
                    );
                    self.m_arc.rewind(0);
                    self.m_status += 1;
                }
                5 => {
                    cmd = self.m_arc.vertex(x, y);
                    if is_stop(cmd) {
                        self.m_status += 1;
                    } else {
                        return PathCmd::LineTo as u32;
                    }
                }
                6 => {
                    self.m_arc.init(
                        self.m_x1 + self.m_rx4,
                        self.m_y2 - self.m_ry4,
                        self.m_rx4,
                        self.m_ry4,
                        std::f64::consts::PI * 0.5,
                        std::f64::consts::PI,
                        true,
                    );
                    self.m_arc.rewind(0);
                    self.m_status += 1;
                }
                7 => {
                    cmd = self.m_arc.vertex(x, y);
                    if is_stop(cmd) {
                        self.m_status += 1;
                    } else {
                        return PathCmd::LineTo as u32;
                    }
                }
                8 => {
                    cmd = PathCmd::EndPoly as u32 | PathFlag::Close as u32 | PathFlag::Ccw as u32;
                    self.m_status += 1;
                    break;
                }
                _ => {
                    break;
                } //abort XXX
            }
        }
        cmd
    }
}

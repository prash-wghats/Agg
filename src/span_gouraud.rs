//----------------------------------------------------------------------------
// Anti-Grain Geometry - Version 2.4
// Copyright (C) 2002-2005 Maxim Shemanarev (http://www.antigrain.com)
//
// Permission to copy, use, modify, sell and distribute this software
// is granted provided this copyright notice appears in all copies.
// This software is provided "as is" without express or implied
// warranty, and with no claim as to its suitability for any purpose.
//
//----------------------------------------------------------------------------
// Contact: mcseem@antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------

use crate::basics::PathCmd;
use crate::math::{calc_intersection, dilate_triangle};
use crate::Color;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct CoordD<C: Color> {
    pub x: f64,
    pub y: f64,
    pub color: C,
}

impl<C: Color> CoordD<C> {
    pub fn new() -> Self {
        Self {
            x: 0.,
            y: 0.,
            color: C::new(),
        }
    }
}
//============================================================SpanGouraud

pub struct SpanGouraud<C: Color> {
    m_vertex: u32,
    m_cmd: [PathCmd; 8],
    m_x: [f64; 8],
    m_y: [f64; 8],
    m_coords: [CoordD<C>; 3],
}

impl<C: Color> SpanGouraud<C> {
    pub fn new() -> Self {
        SpanGouraud {
            m_vertex: 0,
            m_cmd: [PathCmd::Stop; 8],
            m_x: [0.0; 8],
            m_y: [0.0; 8],
            m_coords: [CoordD::new(); 3],
        }
    }

    pub fn new_with_color(
        c1: C, c2: C, c3: C, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, d: f64,
    ) -> Self {
        let mut sg = SpanGouraud {
            m_vertex: 0,
            m_cmd: [PathCmd::Stop; 8],
            m_x: [0.0; 8],
            m_y: [0.0; 8],
            m_coords: [CoordD::new(); 3],
        };
        sg.colors(c1, c2, c3);
        sg.triangle(x1, y1, x2, y2, x3, y3, d);
        sg
    }

    pub fn colors(&mut self, c1: C, c2: C, c3: C) {
        self.m_coords[0].color = c1;
        self.m_coords[1].color = c2;
        self.m_coords[2].color = c3;
    }

    pub fn triangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, d: f64) {
        self.m_coords[0].x = x1;
        self.m_x[0] = x1;
        self.m_coords[0].y = y1;
        self.m_y[0] = y1;
        self.m_coords[1].x = x2;
        self.m_x[1] = x2;
        self.m_coords[1].y = y2;
        self.m_y[1] = y2;
        self.m_coords[2].x = x3;
        self.m_x[2] = x3;
        self.m_coords[2].y = y3;
        self.m_y[2] = y3;
        self.m_cmd[0] = PathCmd::MoveTo;
        self.m_cmd[1] = PathCmd::LineTo;
        self.m_cmd[2] = PathCmd::LineTo;
        self.m_cmd[3] = PathCmd::Stop;

        if d != 0.0 {
            dilate_triangle(
                self.m_coords[0].x,
                self.m_coords[0].y,
                self.m_coords[1].x,
                self.m_coords[1].y,
                self.m_coords[2].x,
                self.m_coords[2].y,
                &mut self.m_x,
                &mut self.m_y,
                d,
            );

            calc_intersection(
                self.m_x[4],
                self.m_y[4],
                self.m_x[5],
                self.m_y[5],
                self.m_x[0],
                self.m_y[0],
                self.m_x[1],
                self.m_y[1],
                &mut self.m_coords[0].x,
                &mut self.m_coords[0].y,
            );

            calc_intersection(
                self.m_x[0],
                self.m_y[0],
                self.m_x[1],
                self.m_y[1],
                self.m_x[2],
                self.m_y[2],
                self.m_x[3],
                self.m_y[3],
                &mut self.m_coords[1].x,
                &mut self.m_coords[1].y,
            );

            calc_intersection(
                self.m_x[2],
                self.m_y[2],
                self.m_x[3],
                self.m_y[3],
                self.m_x[4],
                self.m_y[4],
                self.m_x[5],
                self.m_y[5],
                &mut self.m_coords[2].x,
                &mut self.m_coords[2].y,
            );
            self.m_cmd[3] = PathCmd::LineTo;
            self.m_cmd[4] = PathCmd::LineTo;
            self.m_cmd[5] = PathCmd::LineTo;
            self.m_cmd[6] = PathCmd::Stop;
        }
    }

    pub fn rewind(&mut self, _: u32) {
        self.m_vertex = 0;
    }

    pub fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        *x = self.m_x[self.m_vertex as usize];
        *y = self.m_y[self.m_vertex as usize];
        self.m_vertex += 1;
        self.m_cmd[(self.m_vertex - 1) as usize] as u32
    }

    pub(crate) fn arrange_vertices(&self, coord: &mut [CoordD<C>; 3]) {
        coord[0] = self.m_coords[0];
        coord[1] = self.m_coords[1];
        coord[2] = self.m_coords[2];

        if self.m_coords[0].y > self.m_coords[2].y {
            coord[0] = self.m_coords[2];
            coord[2] = self.m_coords[0];
        }

        let mut tmp: CoordD<C>;
        if coord[0].y > coord[1].y {
            tmp = coord[1];
            coord[1] = coord[0];
            coord[0] = tmp;
        }

        if coord[1].y > coord[2].y {
            tmp = coord[2];
            coord[2] = coord[1];
            coord[1] = tmp;
        }
    }
}

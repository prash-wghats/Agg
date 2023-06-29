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

use crate::basics::{
    is_curve, is_end_poly, is_move_to, is_next_poly, is_stop, is_vertex, set_orientation, PathCmd,
    PathFlag, VertexBase,
};

use crate::math::{calc_distance, VERTEX_DIST_EPSILON};
use crate::{bezier_arc::*, AggPrimitive};
use crate::{Transformer, VertexContainer, VertexSource};

pub type PathStorage = PathBase<VertexStlStorage>;

//-----------------------------------------------------VertexStlStorage
pub struct VertexStlStorage {
    m_vertices: Vec<VertexBase<f64>>,
}

impl VertexContainer for VertexStlStorage {
    fn new() -> VertexStlStorage {
        VertexStlStorage {
            m_vertices: Vec::new(),
        }
    }

    fn remove_all(&mut self) {
        self.m_vertices.clear();
    }

    fn free_all(&mut self) {
        self.m_vertices.clear();
    }

    fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        self.m_vertices.push(VertexBase::new(x, y, cmd));
    }

    fn modify_vertex(&mut self, idx: u32, x: f64, y: f64) {
        self.m_vertices[idx as usize].x = x;
        self.m_vertices[idx as usize].y = y;
    }

    fn modify_vertex_with_cmd(&mut self, idx: u32, x: f64, y: f64, cmd: u32) {
        self.m_vertices[idx as usize].x = x;
        self.m_vertices[idx as usize].y = y;
        self.m_vertices[idx as usize].cmd = cmd;
    }

    fn modify_command(&mut self, idx: u32, cmd: u32) {
        self.m_vertices[idx as usize].cmd = cmd;
    }

    fn swap_vertices(&mut self, v1: u32, v2: u32) {
        self.m_vertices.swap(v1 as usize, v2 as usize);
    }

    fn last_command(&self) -> u32 {
        if self.m_vertices.len() == 0 {
            return PathCmd::Stop as u32;
        }
        self.m_vertices[self.m_vertices.len() - 1].cmd
    }

    fn last_vertex(&self, x: &mut f64, y: &mut f64) -> u32 {
        if self.m_vertices.len() == 0 {
            *x = 0.0;
            *y = 0.0;
            return PathCmd::Stop as u32;
        }
        return self.vertex(self.m_vertices.len() as u32 - 1, x, y);
    }

    fn prev_vertex(&self, x: &mut f64, y: &mut f64) -> u32 {
        if self.m_vertices.len() < 2 {
            *x = 0.0;
            *y = 0.0;
            return PathCmd::Stop as u32;
        }
        return self.vertex(self.m_vertices.len() as u32 - 2, x, y);
    }

    fn last_x(&self) -> f64 {
        if self.m_vertices.len() == 0 {
            return 0.0;
        }
        self.m_vertices[self.m_vertices.len() - 1].x
    }

    fn last_y(&self) -> f64 {
        if self.m_vertices.len() == 0 {
            return 0.0;
        }
        self.m_vertices[self.m_vertices.len() - 1].y
    }

    fn total_vertices(&self) -> u32 {
        self.m_vertices.len() as u32
    }

    fn vertex(&self, idx: u32, x: &mut f64, y: &mut f64) -> u32 {
        let v = &self.m_vertices[idx as usize];
        *x = v.x;
        *y = v.y;
        v.cmd
    }

    fn command(&self, idx: u32) -> u32 {
        let len = self.m_vertices.len();
        self.m_vertices[idx as usize % len].cmd
    }
}

//-----------------------------------------------------PolyPlainAdaptor
pub struct PolyPlainAdaptor<'a, T: AggPrimitive> {
    data: &'a [T],
    ptr: usize,
    end: usize,
    closed: bool,
    stop: bool,
}

impl<'a, T: AggPrimitive> PolyPlainAdaptor<'a, T> {
    pub fn new() -> Self {
        PolyPlainAdaptor {
            data: &[],
            ptr: 0,
            end: 0,
            closed: false,
            stop: false,
        }
    }

    pub fn new_init(data: &'a [T], num_points: usize, closed: bool) -> Self {
        PolyPlainAdaptor {
            data: data,
            ptr: 0,
            end: num_points * 2,
            closed: closed,
            stop: false,
        }
    }

    pub fn init(&mut self, data: &'a [T], num_points: usize, closed: bool) {
        self.data = data;
        self.ptr = 0;
        self.end = num_points * 2;
        self.closed = closed;
        self.stop = false;
    }
}

impl<'a, T: AggPrimitive> VertexSource for PolyPlainAdaptor<'a, T> {
    fn rewind(&mut self, _: u32) {
        self.ptr = 0;
        self.stop = false;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.ptr < self.end {
            let first = self.ptr == 0;
            *x = self.data[self.ptr].into_f64();
            self.ptr += 1;
            *y = self.data[self.ptr].into_f64();
            self.ptr += 1;
            if first {
                return PathCmd::MoveTo as u32;
            } else {
                return PathCmd::LineTo as u32;
            }
        }
        *x = 0.0;
        *y = 0.0;
        if self.closed && !self.stop {
            self.stop = true;
            return PathCmd::EndPoly as u32 | PathFlag::Close as u32;
        }
        return PathCmd::Stop as u32;
    }
}

//---------------------------------------------------------------PathBase
// A container to store vertices with their flags.
// A path consists of a number of contours separated with "move_to"
// commands. The path storage can keep and maintain more than one
// path.
// To navigate to the beginning of a particular path, use rewind(path_id);
// Where path_id is what start_new_path() returns. So, when you call
// start_new_path() you need to store its return value somewhere else
// to navigate to the path afterwards.
//
// See also: vertex_source concept
//------------------------------------------------------------------------

pub struct PathBase<VC: VertexContainer> {
    m_vertices: VC,
    m_iterator: u32,
}

impl<VC: VertexContainer> PathBase<VC> {
    pub fn new() -> Self {
        PathBase {
            m_vertices: VC::new(),
            m_iterator: 0,
        }
    }
    pub fn remove_all(&mut self) {
        self.m_vertices.remove_all();
        self.m_iterator = 0;
    }

    pub fn free_all(&mut self) {
        self.m_vertices.free_all();
        self.m_iterator = 0;
    }

    pub fn concat_path<VS: VertexSource>(&mut self, vs: &mut VS, path_id: u32) {
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;
        let mut cmd: u32;
		vs.rewind(path_id);
        
        loop {
           
			cmd = vs.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            self.m_vertices.add_vertex(x, y, cmd);
        }
    }

    pub fn join_path<VS: VertexSource>(&mut self, vs: &mut VS, path_id: u32) {
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;
        let mut cmd: u32;
        vs.rewind(path_id);
        cmd = vs.vertex(&mut x, &mut y);
        if !is_stop(cmd) {
            if is_vertex(cmd) {
                let mut x0: f64 = 0.;
                let mut y0: f64 = 0.;
                let cmd0 = self.last_vertex(&mut x0, &mut y0);
                if is_vertex(cmd0) {
                    if calc_distance(x, y, x0, y0) > VERTEX_DIST_EPSILON {
                        if is_move_to(cmd) {
                            cmd = PathCmd::LineTo as u32;
                        }
                        self.m_vertices.add_vertex(x, y, cmd);
                    }
                } else {
                    if is_stop(cmd0) {
                        cmd = PathCmd::MoveTo as u32;
                    } else {
                        if is_move_to(cmd) {
                            cmd = PathCmd::LineTo as u32;
                        }
                    }
                    self.m_vertices.add_vertex(x, y, cmd);
                }
            }
            cmd = vs.vertex(&mut x, &mut y);
            while !is_stop(cmd) {
                self.m_vertices.add_vertex(
                    x,
                    y,
                    if is_move_to(cmd) {
                        PathCmd::LineTo as u32
                    } else {
                        cmd
                    },
                );
            }
        }
    }

    pub fn concat_poly<T: AggPrimitive>(&mut self, data: &[T], num_points: usize, closed: bool) {
        let mut poly = PolyPlainAdaptor::<T>::new_init(data, num_points, closed);
        self.concat_path(&mut poly, 0);
    }

    pub fn join_poly<T: AggPrimitive>(&mut self, data: &[T], num_points: usize, closed: bool) {
        let mut poly = PolyPlainAdaptor::<T>::new_init(data, num_points, closed);
        self.join_path(&mut poly, 0);
    }

    pub fn transform<T: Transformer>(&mut self, trans: &T, path_id: u32) {
        let num_ver = self.m_vertices.total_vertices();
        for i in path_id..num_ver {
            let mut x: f64 = 0.;
            let mut y: f64 = 0.;
            let cmd = self.m_vertices.vertex(i, &mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_vertex(cmd) {
                trans.transform(&mut x, &mut y);
                self.m_vertices.modify_vertex(i, x, y);
            }
        }
    }

    pub fn transform_all_paths<T: Transformer>(&mut self, trans: &T) {
        let num_ver = self.m_vertices.total_vertices();
        for i in 0..num_ver {
            let mut x: f64 = 0.;
            let mut y: f64 = 0.;
            if is_vertex(self.m_vertices.vertex(i, &mut x, &mut y)) {
                trans.transform(&mut x, &mut y);
                self.m_vertices.modify_vertex(i, x, y);
            }
        }
    }

    pub fn start_new_path(&mut self) -> u32 {
        if !is_stop(self.m_vertices.last_command()) {
            self.m_vertices.add_vertex(0.0, 0.0, PathCmd::Stop as u32);
        }
        self.m_vertices.total_vertices()
    }

    pub fn rel_to_abs(&self, x: &mut f64, y: &mut f64) {
        if self.m_vertices.total_vertices() > 0 {
            let mut x2: f64 = 0.0;
            let mut y2: f64 = 0.0;
            if is_vertex(self.m_vertices.last_vertex(&mut x2, &mut y2)) {
                *x += x2;
                *y += y2;
            }
        }
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.m_vertices.add_vertex(x, y, PathCmd::MoveTo as u32);
    }

    pub fn move_rel(&mut self, dx: f64, dy: f64) {
        let (mut dx, mut dy) = (dx, dy);
        self.rel_to_abs(&mut dx, &mut dy);
        self.m_vertices.add_vertex(dx, dy, PathCmd::MoveTo as u32);
    }

    pub fn line_to(&mut self, x: f64, y: f64) {
        self.m_vertices.add_vertex(x, y, PathCmd::LineTo as u32);
    }

    pub fn line_rel(&mut self, dx: f64, dy: f64) {
        let (mut dx, mut dy) = (dx, dy);
        self.rel_to_abs(&mut dx, &mut dy);
        self.m_vertices.add_vertex(dx, dy, PathCmd::LineTo as u32);
    }

    pub fn hline_to(&mut self, x: f64) {
        self.m_vertices
            .add_vertex(x, self.last_y(), PathCmd::LineTo as u32);
    }

    pub fn hline_rel(&mut self, dx: f64) {
        let mut dy: f64 = 0.0;
        let mut dx = dx;
        self.rel_to_abs(&mut dx, &mut dy);
        self.m_vertices.add_vertex(dx, dy, PathCmd::LineTo as u32);
    }

    pub fn vline_to(&mut self, y: f64) {
        self.m_vertices
            .add_vertex(self.last_x(), y, PathCmd::LineTo as u32);
    }

    pub fn vline_rel(&mut self, dy: f64) {
        let mut dx: f64 = 0.0;
        let mut dy = dy;
        self.rel_to_abs(&mut dx, &mut dy);
        self.m_vertices.add_vertex(dx, dy, PathCmd::LineTo as u32);
    }

    pub fn arc_to(
        &mut self, rx: f64, ry: f64, angle: f64, large_arc_flag: bool, sweep_flag: bool, x: f64,
        y: f64,
    ) {
        if self.m_vertices.total_vertices() > 0 && is_vertex(self.m_vertices.last_command()) {
            let epsilon = 1e-30;
            let mut x0 = 0.0;
            let mut y0 = 0.0;
            self.m_vertices.last_vertex(&mut x0, &mut y0);

            let rx = rx.abs();
            let ry = ry.abs();

            // Ensure radii are valid
            //-------------------------
            if rx < epsilon || ry < epsilon {
                self.line_to(x, y);
                return;
            }

            if calc_distance(x0, y0, x, y) < epsilon {
                // If the endpoints (x, y) and (x0, y0) are identical, then this
                // is equivalent to omitting the elliptical arc segment entirely.
                return;
            }
            let mut a = BezierArcSvg::new_with_params(
                x0,
                y0,
                rx,
                ry,
                angle,
                large_arc_flag,
                sweep_flag,
                x,
                y,
            );
            if a.radii_ok() {
                self.join_path(&mut a, 0);
            } else {
                self.line_to(x, y);
            }
        } else {
            self.move_to(x, y);
        }
    }

    pub fn arc_rel(
        &mut self, rx: f64, ry: f64, angle: f64, large_arc_flag: bool, sweep_flag: bool, dx: f64,
        dy: f64,
    ) {
        let (mut dx, mut dy) = (dx, dy);
        self.rel_to_abs(&mut dx, &mut dy);
        self.arc_to(rx, ry, angle, large_arc_flag, sweep_flag, dx, dy);
    }

    pub fn curve3_ctrl(&mut self, x_ctrl: f64, y_ctrl: f64, x_to: f64, y_to: f64) {
        self.m_vertices
            .add_vertex(x_ctrl, y_ctrl, PathCmd::Curve3 as u32);
        self.m_vertices
            .add_vertex(x_to, y_to, PathCmd::Curve3 as u32);
    }

    pub fn curve3_rel_ctrl(&mut self, dx_ctrl: f64, dy_ctrl: f64, dx_to: f64, dy_to: f64) {
        let (mut dx_ctrl, mut dy_ctrl, mut dx_to, mut dy_to) = (dx_ctrl, dy_ctrl, dx_to, dy_to);
        self.rel_to_abs(&mut dx_ctrl, &mut dy_ctrl);
        self.rel_to_abs(&mut dx_to, &mut dy_to);
        self.m_vertices
            .add_vertex(dx_ctrl, dy_ctrl, PathCmd::Curve3 as u32);
        self.m_vertices
            .add_vertex(dx_to, dy_to, PathCmd::Curve3 as u32);
    }

    pub fn curve3(&mut self, x_to: f64, y_to: f64) {
        let mut x0 = 0.;
        let mut y0 = 0.;
        if is_vertex(self.m_vertices.last_vertex(&mut x0, &mut y0)) {
            let mut x_ctrl = 0.;
            let mut y_ctrl = 0.;
            let cmd = self.m_vertices.prev_vertex(&mut x_ctrl, &mut y_ctrl);
            if is_curve(cmd) {
                x_ctrl = x0 + x0 - x_ctrl;
                y_ctrl = y0 + y0 - y_ctrl;
            } else {
                x_ctrl = x0;
                y_ctrl = y0;
            }
            self.curve3_ctrl(x_ctrl, y_ctrl, x_to, y_to);
        }
    }

    pub fn curve3_rel(&mut self, dx_to: f64, dy_to: f64) {
        let (mut dx_to, mut dy_to) = (dx_to, dy_to);
        self.rel_to_abs(&mut dx_to, &mut dy_to);
        self.curve3(dx_to, dy_to);
    }

    pub fn curve4_ctrl(
        &mut self, x_ctrl1: f64, y_ctrl1: f64, x_ctrl2: f64, y_ctrl2: f64, x_to: f64, y_to: f64,
    ) {
        self.m_vertices
            .add_vertex(x_ctrl1, y_ctrl1, PathCmd::Curve4 as u32);
        self.m_vertices
            .add_vertex(x_ctrl2, y_ctrl2, PathCmd::Curve4 as u32);
        self.m_vertices
            .add_vertex(x_to, y_to, PathCmd::Curve4 as u32);
    }

    pub fn curve4_rel_ctrl(
        &mut self, dx_ctrl1: f64, dy_ctrl1: f64, dx_ctrl2: f64, dy_ctrl2: f64, dx_to: f64,
        dy_to: f64,
    ) {
        let (mut dx_ctrl1, mut dy_ctrl1, mut dx_ctrl2, mut dy_ctrl2, mut dx_to, mut dy_to) =
            (dx_ctrl1, dy_ctrl1, dx_ctrl2, dy_ctrl2, dx_to, dy_to);
        self.rel_to_abs(&mut dx_ctrl1, &mut dy_ctrl1);
        self.rel_to_abs(&mut dx_ctrl2, &mut dy_ctrl2);
        self.rel_to_abs(&mut dx_to, &mut dy_to);
        self.m_vertices
            .add_vertex(dx_ctrl1, dy_ctrl1, PathCmd::Curve4 as u32);
        self.m_vertices
            .add_vertex(dx_ctrl2, dy_ctrl2, PathCmd::Curve4 as u32);
        self.m_vertices
            .add_vertex(dx_to, dy_to, PathCmd::Curve4 as u32);
    }

    pub fn curve4(&mut self, x_ctrl2: f64, y_ctrl2: f64, x_to: f64, y_to: f64) {
        let mut x0: f64 = 0.0;
        let mut y0: f64 = 0.0;
        if is_vertex(self.last_vertex(&mut x0, &mut y0)) {
            let mut x_ctrl1: f64 = 0.0;
            let mut y_ctrl1: f64 = 0.0;
            let cmd = self.prev_vertex(&mut x_ctrl1, &mut y_ctrl1);
            if is_curve(cmd) {
                x_ctrl1 = x0 + x0 - x_ctrl1;
                y_ctrl1 = y0 + y0 - y_ctrl1;
            } else {
                x_ctrl1 = x0;
                y_ctrl1 = y0;
            }
            self.curve4_ctrl(x_ctrl1, y_ctrl1, x_ctrl2, y_ctrl2, x_to, y_to);
        }
    }

    pub fn curve4_rel(&mut self, dx_ctrl2: f64, dy_ctrl2: f64, dx_to: f64, dy_to: f64) {
        let (mut dx_ctrl2, mut dy_ctrl2, mut dx_to, mut dy_to) = (dx_ctrl2, dy_ctrl2, dx_to, dy_to);
        self.rel_to_abs(&mut dx_ctrl2, &mut dy_ctrl2);
        self.rel_to_abs(&mut dx_to, &mut dy_to);
        self.curve4(dx_ctrl2, dy_ctrl2, dx_to, dy_to);
    }

    pub fn end_poly(&mut self, flags: u32) {
        if is_vertex(self.m_vertices.last_command()) {
            self.m_vertices
                .add_vertex(0.0, 0.0, PathCmd::EndPoly as u32 | flags);
        }
    }

    pub fn close_polygon(&mut self, flags: u32) {
        self.end_poly(PathFlag::Close as u32 | flags);
    }

    pub fn total_vertices(&self) -> u32 {
        self.m_vertices.total_vertices()
    }

    pub fn last_vertex(&self, x: &mut f64, y: &mut f64) -> u32 {
        self.m_vertices.last_vertex(x, y)
    }

    pub fn prev_vertex(&self, x: &mut f64, y: &mut f64) -> u32 {
        self.m_vertices.prev_vertex(x, y)
    }

    pub fn last_x(&self) -> f64 {
        self.m_vertices.last_x()
    }

    pub fn last_y(&self) -> f64 {
        self.m_vertices.last_y()
    }

    pub fn vertex_idx(&self, idx: u32, x: &mut f64, y: &mut f64) -> u32 {
        self.m_vertices.vertex(idx, x, y)
    }

    pub fn command(&self, idx: u32) -> u32 {
        self.m_vertices.command(idx)
    }

    pub fn modify_vertex(&mut self, idx: u32, x: f64, y: f64) {
        self.m_vertices.modify_vertex(idx, x, y);
    }

    pub fn modify_vertex_cmd(&mut self, idx: u32, x: f64, y: f64, cmd: u32) {
        self.m_vertices.modify_vertex_with_cmd(idx, x, y, cmd);
    }

    pub fn modify_command(&mut self, idx: u32, cmd: u32) {
        self.m_vertices.modify_command(idx, cmd);
    }

    fn perceive_polygon_orientation(&self, start: u32, end: u32) -> u32 {
        // Calculate signed area (double area to be exact)
        //---------------------
        let np = end - start;
        let mut area = 0.0;
        let (mut x1, mut y1, mut x2, mut y2) = (0., 0., 0., 0.);
        for i in 0..np {
            self.m_vertices.vertex(start + i, &mut x1, &mut y1);
            self.m_vertices
                .vertex(start + (i + 1) % np, &mut x2, &mut y2);
            area += x1 * y2 - y1 * x2;
        }
        if area < 0.0 {
            PathFlag::Cw as u32
        } else {
            PathFlag::Ccw as u32
        }
    }

    fn invert_polygon_end(&mut self, start: u32, end: u32) {
        let mut start = start;
        //let mut end = end;
        let tmp_cmd = self.m_vertices.command(start);

        // Make "end" inclusive
        let mut end = end - 1;

        // Shift all commands to one position
        for i in start..end {
            self.m_vertices
                .modify_command(i, self.m_vertices.command(i + 1));
        }

        // Assign starting command to the ending command
        self.m_vertices.modify_command(end, tmp_cmd);

        // Reverse the polygon
        while end > start {
            self.m_vertices.swap_vertices(start, end);
            start += 1;
            end -= 1;
        }
    }

    pub fn invert_polygon(&mut self, start: u32) {
        let mut start = start;
        // Skip all non-m_vertices at the beginning
        while start < self.m_vertices.total_vertices() && !is_vertex(self.m_vertices.command(start))
        {
            start += 1;
        }

        // Skip all insignificant move_to
        while start + 1 < self.m_vertices.total_vertices()
            && is_move_to(self.m_vertices.command(start))
            && is_move_to(self.m_vertices.command(start + 1))
        {
            start += 1;
        }

        // Find the last vertex
        let mut end = start + 1;
        while end < self.m_vertices.total_vertices() && !is_next_poly(self.m_vertices.command(end))
        {
            end += 1;
        }

        self.invert_polygon_end(start, end);
    }

    fn arrange_polygon_orientation(&mut self, start0: u32, orientation: PathFlag) -> u32 {
        let mut start = start0;
        if orientation == PathFlag::None {
            return start;
        }

        // Skip all non-m_vertices at the beginning
        while start < self.m_vertices.total_vertices() && !is_vertex(self.m_vertices.command(start))
        {
            start += 1;
        }

        // Skip all insignificant move_to
        while start + 1 < self.m_vertices.total_vertices()
            && is_move_to(self.m_vertices.command(start))
            && is_move_to(self.m_vertices.command(start + 1))
        {
            start += 1;
        }

        // Find the last vertex
        let mut end = start + 1;
        while end < self.m_vertices.total_vertices() && !is_next_poly(self.m_vertices.command(end))
        {
            end += 1;
        }

        if end - start > 2 {
            if self.perceive_polygon_orientation(start, end) != orientation as u32 {
                // Invert polygon, set orientation flag, and skip all end_poly
                self.invert_polygon_end(start, end);
                let mut cmd;
                while end < self.m_vertices.total_vertices()
                    && is_end_poly(self.m_vertices.command(end))
                {
                    cmd = self.m_vertices.command(end);
                    self.m_vertices
                        .modify_command(end, set_orientation(cmd, orientation as u32));
                    end += 1;
                }
            }
        }
        end
    }

    fn arrange_orientations(&mut self, start0: u32, orientation: PathFlag) -> u32 {
        let mut start = start0;
        if orientation != PathFlag::None {
            while start < self.m_vertices.total_vertices() {
                start = self.arrange_polygon_orientation(start, orientation);
                if is_stop(self.m_vertices.command(start)) {
                    start += 1;
                    break;
                }
            }
        }
        start
    }

    pub fn arrange_orientations_all_paths(&mut self, orientation: PathFlag) {
        if orientation != PathFlag::None {
            let mut start = 0;
            while start < self.m_vertices.total_vertices() {
                start = self.arrange_orientations(start, orientation);
            }
        }
    }

    pub fn flip_x(&mut self, x1: f64, x2: f64) {
        let mut i = 0;
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;
        while i < self.m_vertices.total_vertices() {
            let cmd = self.m_vertices.vertex(i, &mut x, &mut y);
            if is_vertex(cmd) {
                self.m_vertices.modify_vertex(i, x2 - x + x1, y);
            }
            i += 1;
        }
    }

    pub fn flip_y(&mut self, y1: f64, y2: f64) {
        let mut i: u32 = 0;
        let mut x: f64 = 0.;
        let mut y: f64 = 0.;
        while i < self.m_vertices.total_vertices() {
            let cmd = self.m_vertices.vertex(i, &mut x, &mut y);
            if is_vertex(cmd) {
                self.m_vertices.modify_vertex(i, x, y2 - y + y1);
            }
            i += 1;
        }
    }

    pub fn translate(&mut self, dx: f64, dy: f64, path_id_: u32) {
        let mut path_id = path_id_;
        let num_ver = self.m_vertices.total_vertices();
        while path_id < num_ver {
            let mut x: f64 = 0.;
            let mut y: f64 = 0.;
            let cmd = self.m_vertices.vertex(path_id, &mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            if is_vertex(cmd) {
                x += dx;
                y += dy;
                self.m_vertices.modify_vertex(path_id, x, y);
            }
            path_id += 1;
        }
    }

    pub fn translate_all_paths(&mut self, dx: f64, dy: f64) {
        let num_ver = self.m_vertices.total_vertices();
        for idx in 0..num_ver {
            let mut x = 0.0;
            let mut y = 0.0;
            if is_vertex(self.m_vertices.vertex(idx, &mut x, &mut y)) {
                x += dx;
                y += dy;
                self.m_vertices.modify_vertex(idx, x, y);
            }
        }
    }
}

impl<VC: VertexContainer> VertexSource for PathBase<VC> {
    fn rewind(&mut self, path_id: u32) {
        self.m_iterator = path_id;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if self.m_iterator >= self.m_vertices.total_vertices() {
            return PathCmd::Stop as u32;
        }
        let i = self.m_iterator;

        self.m_iterator += 1;
        self.m_vertices.vertex(i, x, y)
    }
}

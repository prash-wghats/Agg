use crate::basics::*;
use crate::{RenderPrim, VertexSource};
use std::ops::Index;

//======================================================rasterizer_outline
//pub struct RasterizerOutline<R: RendererOutline> {
pub struct RasterizerOutline<'a, R: RenderPrim> {
    ren: &'a mut R,
    start_x: i32,
    start_y: i32,
    vertices: u32,
}

impl<'a, R: RenderPrim> RasterizerOutline<'a, R> {
    pub fn new(ren: &'a mut R) -> Self {
        RasterizerOutline {
            ren: ren,
            start_x: 0,
            start_y: 0,
            vertices: 0,
        }
    }

    pub fn attach(&mut self, ren: &'a mut R) {
        self.ren = ren;
    }

    pub fn ren_mut(&mut self) -> &mut R {
        return self.ren;
    }

    pub fn ren(&self) -> &R {
        return &*self.ren;
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.vertices = 1;
        self.start_x = x;
        self.start_y = y;
        self.ren.move_to(x, y);
    }

    pub fn line_to(&mut self, x: i32, y: i32) {
        self.vertices += 1;
        self.ren.line_to(x, y, false);
    }

    pub fn move_to_d(&mut self, x: f64, y: f64) {
        self.move_to(self.ren.coord(x), self.ren.coord(y));
    }

    pub fn line_to_d(&mut self, x: f64, y: f64) {
        self.line_to(self.ren.coord(x), self.ren.coord(y));
    }

    pub fn close(&mut self) {
        if self.vertices > 2 {
            self.line_to(self.start_x, self.start_y);
        }
        self.vertices = 0;
    }

    pub fn add_vertex(&mut self, x: f64, y: f64, cmd: u32) {
        if is_move_to(cmd) {
            self.move_to_d(x, y);
        } else {
            if is_end_poly(cmd) {
                if is_closed(cmd) {
                    self.close();
                }
            } else {
                self.line_to_d(x, y);
            }
        }
    }

    pub fn add_path<VS: VertexSource>(&mut self, vs: &mut VS, path_id: u32) {
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;

        let mut cmd;
        vs.rewind(path_id);
        loop {
            cmd = vs.vertex(&mut x, &mut y);
            if is_stop(cmd) {
                break;
            }
            self.add_vertex(x, y, cmd);
        }
    }

    pub fn render_all_paths<
        VS: VertexSource,
        CS: Index<usize, Output = R::ColorType>,
        PathId: Index<usize, Output = u32>,
    >(
        &mut self, vs: &mut VS, colors: &CS, path_id: &PathId, num_paths: u32,
    ) {
        for i in 0..num_paths as usize {
            self.ren.set_line_color(colors[i]);
            self.add_path(vs, path_id[i]);
        }
    }

    /*pub fn render_ctrl<C: Ctrl>(&mut self, c: &mut C) {
        for i in 0..c.num_paths() {
            self.ren.line_color(c.color(i));
            self.add_path(c, i);
        }
    }*/
}

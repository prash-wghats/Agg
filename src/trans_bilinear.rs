
use crate::simul_eq::SimulEq;
use crate::Transformer;
//==========================================================TransBilinear
#[derive(Clone, Copy)]
pub struct TransBilinear {
    mtx: [[f64; 2]; 4],
    valid: bool,
}

impl TransBilinear {
    // Arbitrary quadrangle transformations
    pub fn new(src: &[f64], dst: &[f64]) -> TransBilinear {
        let mut t = TransBilinear {
            mtx: [[0.0; 2]; 4],
            valid: false,
        };
        t.quad_to_quad(src, dst);
        t
    }

    // Direct transformations
    pub fn new_rect_to_quad(x1: f64, y1: f64, x2: f64, y2: f64, quad: &[f64]) -> TransBilinear {
        let mut t = TransBilinear {
            mtx: [[0.0; 2]; 4],
            valid: false,
        };
        t.rect_to_quad(x1, y1, x2, y2, quad);
        t
    }

    // Reverse transformations
    pub fn new_quad_to_rect(quad: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) -> TransBilinear {
        let mut t = TransBilinear {
            mtx: [[0.0; 2]; 4],
            valid: false,
        };
        t.quad_to_rect(quad, x1, y1, x2, y2);
        t
    }

    // Set the transformations using two arbitrary quadrangles.
    fn quad_to_quad(&mut self, src: &[f64], dst: &[f64]) {
        let mut left = [[0.0; 4]; 4];
        let mut right = [[0.0; 2]; 4];

        for i in 0..4 {
            let ix = i * 2;
            let iy = ix + 1;
            left[i][0] = 1.0;
            left[i][1] = src[ix] * src[iy];
            left[i][2] = src[ix];
            left[i][3] = src[iy];

            right[i][0] = dst[ix];
            right[i][1] = dst[iy];
        }
        self.valid = SimulEq::<4, 2, {4+2}>::solve(&left, &right, &mut self.mtx);
    }

    // Set the direct transformations, i.e., rectangle -> quadrangle
    fn rect_to_quad(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, quad: &[f64]) {
        let src = [x1, y1, x2, y1, x2, y2, x1, y2];
        self.quad_to_quad(&src, quad);
    }

    // Set the reverse transformations, i.e., quadrangle -> rectangle
    fn quad_to_rect(&mut self, quad: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) {
        let dst = [x1, y1, x2, y1, x2, y2, x1, y2];

        self.quad_to_quad(quad, &dst);
    }

    // Check if the equations were solved successfully
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

impl Transformer for TransBilinear {
    // Transform a point (x, y)
    fn transform(&self, x: &mut f64, y: &mut f64) {
        let tx = *x;
        let ty = *y;
        let xy = tx * ty;
        *x = self.mtx[0][0] + self.mtx[1][0] * xy + self.mtx[2][0] * tx + self.mtx[3][0] * ty;
        *y = self.mtx[0][1] + self.mtx[1][1] * xy + self.mtx[2][1] * tx + self.mtx[3][1] * ty;
    }

    fn scaling_abs(&self, x: &mut f64, y: &mut f64) {
        *x = 0.;
        *y = 0.;
        todo!()
    }
}

pub struct IteratorX {
    inc_x: f64,
    inc_y: f64,
    x: f64,
    y: f64,
    mtx: [[f64; 2]; 4],
    _valid: bool,
}

impl IteratorX {
    pub fn new(tx: f64, ty: f64, step: f64, m: &[[f64; 2]; 4]) -> IteratorX {
        IteratorX {
            inc_x: m[1][0] * step * ty + m[2][0] * step,
            inc_y: m[1][1] * step * ty + m[2][1] * step,
            x: m[0][0] + m[1][0] * tx * ty + m[2][0] * tx + m[3][0] * ty,
            y: m[0][1] + m[1][1] * tx * ty + m[2][1] * tx + m[3][1] * ty,
            mtx: [[0.0; 2]; 4],
            _valid: false,
        }
    }

    pub fn inc(&mut self) {
        self.x += self.inc_x;
        self.y += self.inc_y;
    }

    pub fn begin(&self, x: f64, y: f64, step: f64) -> IteratorX {
        IteratorX::new(x, y, step, &self.mtx)
    }
}

use crate::basics::is_equal_eps;
use crate::trans_affine::TransAffine;
use crate::Transformer;

//=======================================================TransPerspective
#[derive(Copy, Clone)]
pub struct TransPerspective {
    pub sx: f64,
    pub shy: f64,
    pub w0: f64,
    pub shx: f64,
    pub sy: f64,
    pub w1: f64,
    pub tx: f64,
    pub ty: f64,
    pub w2: f64,
}

impl Transformer for TransPerspective {

    fn transform(&self, px: &mut f64, py: &mut f64) {
        let x = *px;
        let y = *py;
        let m = 1.0 / (x * self.w0 + y * self.w1 + self.w2);
        *px = m * (x * self.sx + y * self.shx + self.tx);
        *py = m * (x * self.shy + y * self.sy + self.ty);
    }

    fn scaling_abs(&self, x: &mut f64, y: &mut f64) {
        *x = (self.sx * self.sx + self.shx * self.shx).sqrt();
        *y = (self.shy * self.shy + self.sy * self.sy).sqrt();
    }
}

impl TransPerspective {
    //------------------------------------------------------- Construction
    // Identity matrix
    pub fn new() -> TransPerspective {
        TransPerspective {
            sx: 1.0,
            shy: 0.0,
            w0: 0.0,
            shx: 0.0,
            sy: 1.0,
            w1: 0.0,
            tx: 0.0,
            ty: 0.0,
            w2: 1.0,
        }
    }

    // Custom matrix
    pub fn new_custom(
        v0: f64, v1: f64, v2: f64, v3: f64, v4: f64, v5: f64, v6: f64, v7: f64, v8: f64,
    ) -> TransPerspective {
        TransPerspective {
            sx: v0,
            shy: v1,
            w0: v2,
            shx: v3,
            sy: v4,
            w1: v5,
            tx: v6,
            ty: v7,
            w2: v8,
        }
    }

    // Custom matrix from m[9]
    pub fn new_custom_from_array(m: &[f64; 9]) -> TransPerspective {
        TransPerspective {
            sx: m[0],
            shy: m[1],
            w0: m[2],
            shx: m[3],
            sy: m[4],
            w1: m[5],
            tx: m[6],
            ty: m[7],
            w2: m[8],
        }
    }

    // From affine
    pub fn new_from_affine(a: &TransAffine) -> TransPerspective {
        TransPerspective {
            sx: a.sx,
            shy: a.shy,
            w0: 0.0,
            shx: a.shx,
            sy: a.sy,
            w1: 0.0,
            tx: a.tx,
            ty: a.ty,
            w2: 1.0,
        }
    }

    // Rectangle to quadrilateral
    pub fn new_rect_to_quad(x1: f64, y1: f64, x2: f64, y2: f64, quad: &[f64]) -> TransPerspective {
        let mut m = TransPerspective::new();
        m.rect_to_quad(x1, y1, x2, y2, quad);
        m
    }

    // Quadrilateral to rectangle
    pub fn new_quad_to_rect(quad: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) -> TransPerspective {
        let mut m = TransPerspective::new();
        m.quad_to_rect(quad, x1, y1, x2, y2);
        m
    }

    // Arbitrary quadrilateral transformations
    pub fn new_quad_to_quad(src: &[f64], dst: &[f64]) -> TransPerspective {
        let mut m = TransPerspective::new();
        m.quad_to_quad(src, dst);
        m
    }

    pub fn reset(&mut self) {
        self.sx = 0.0;
        self.shy = 0.0;
        self.w0 = 0.0;
        self.shx = 0.0;
        self.sy = 0.0;
        self.w1 = 0.0;
        self.tx = 0.0;
        self.ty = 0.0;
        self.w2 = 0.0;
    }

    pub fn square_to_quad(&mut self, q: &[f64]) -> bool {
        let dx = q[0] - q[2] + q[4] - q[6];
        let dy = q[1] - q[3] + q[5] - q[7];
        if dx == 0.0 && dy == 0.0 {
            // Affine case (parallelogram)
            //---------------
            self.sx = q[2] - q[0];
            self.shy = q[3] - q[1];
            self.w0 = 0.0;
            self.shx = q[4] - q[2];
            self.sy = q[5] - q[3];
            self.w1 = 0.0;
            self.tx = q[0];
            self.ty = q[1];
            self.w2 = 1.0;
        } else {
            let dx1 = q[2] - q[4];
            let dy1 = q[3] - q[5];
            let dx2 = q[6] - q[4];
            let dy2 = q[7] - q[5];
            let den = dx1 * dy2 - dx2 * dy1;
            if den == 0.0 {
                // Singular case
                //---------------
                self.sx = 0.0;
                self.shy = 0.0;
                self.w0 = 0.0;
                self.shx = 0.0;
                self.sy = 0.0;
                self.w1 = 0.0;
                self.tx = 0.0;
                self.ty = 0.0;
                self.w2 = 0.0;
                return false;
            }
            // General case
            //---------------
            let u = (dx * dy2 - dy * dx2) / den;
            let v = (dy * dx1 - dx * dy1) / den;
            self.sx = q[2] - q[0] + u * q[2];
            self.shy = q[3] - q[1] + u * q[3];
            self.w0 = u;
            self.shx = q[6] - q[0] + v * q[6];
            self.sy = q[7] - q[1] + v * q[7];
            self.w1 = v;
            self.tx = q[0];
            self.ty = q[1];
            self.w2 = 1.0;
        }
        true
    }

    pub fn invert(&mut self) -> bool {
        let d0 = self.sy * self.w2 - self.w1 * self.ty;
        let d1 = self.w0 * self.ty - self.shy * self.w2;
        let d2 = self.shy * self.w1 - self.w0 * self.sy;
        let mut d = self.sx * d0 + self.shx * d1 + self.tx * d2;
        if d == 0.0 {
            self.sx = 0.0;
            self.shy = 0.0;
            self.w0 = 0.0;
            self.shx = 0.0;
            self.sy = 0.0;
            self.w1 = 0.0;
            self.tx = 0.0;
            self.ty = 0.0;
            self.w2 = 0.0;
            return false;
        }
        d = 1.0 / d;
        let a = *self;
        self.sx = d * d0;
        self.shy = d * d1;
        self.w0 = d * d2;
        self.shx = d * (a.w1 * a.tx - a.shx * a.w2);
        self.sy = d * (a.sx * a.w2 - a.w0 * a.tx);
        self.w1 = d * (a.w0 * a.shx - a.sx * a.w1);
        self.tx = d * (a.shx * a.ty - a.sy * a.tx);
        self.ty = d * (a.shy * a.tx - a.sx * a.ty);
        self.w2 = d * (a.sx * a.sy - a.shy * a.shx);
        true
    }

    pub fn quad_to_square(&mut self, q: &[f64]) -> bool {
        if !self.square_to_quad(q) {
            return false;
        }
        self.invert();
        true
    }

    //-------------------------------------- Quadrilateral transformations
    // The arguments are double[8] that are mapped to quadrilaterals:
    // x1,y1, x2,y2, x3,y3, x4,y4
    pub fn quad_to_quad(&mut self, qs: &[f64], qd: &[f64]) -> bool {
        let mut p = TransPerspective::new();
        if !self.quad_to_square(qs) {
            return false;
        }
        if !p.square_to_quad(qd) {
            return false;
        }
        self.multiply(&p);
        true
    }
    pub fn rect_to_quad(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, q: &[f64]) -> bool {
        let mut r: [f64; 8] = [0.0; 8];
        r[0] = x1;
        r[6] = x1;
        r[2] = x2;
        r[4] = x2;
        r[1] = y1;
        r[3] = y1;
        r[5] = y2;
        r[7] = y2;
        self.quad_to_quad(&r, q)
    }


    pub fn quad_to_rect(&mut self, q: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
        let mut r: [f64; 8] = [0.0; 8];
        r[0] = x1;
        r[6] = x1;
        r[2] = x2;
        r[4] = x2;
        r[1] = y1;
        r[3] = y1;
        r[5] = y2;
        r[7] = y2;
        self.quad_to_quad(q, &r)
    }

    pub fn multiply(&mut self, a: &TransPerspective) -> &Self {
        let b = *self;
        self.sx = a.sx * b.sx + a.shx * b.shy + a.tx * b.w0;
        self.shx = a.sx * b.shx + a.shx * b.sy + a.tx * b.w1;
        self.tx = a.sx * b.tx + a.shx * b.ty + a.tx * b.w2;
        self.shy = a.shy * b.sx + a.sy * b.shy + a.ty * b.w0;
        self.sy = a.shy * b.shx + a.sy * b.sy + a.ty * b.w1;
        self.ty = a.shy * b.tx + a.sy * b.ty + a.ty * b.w2;
        self.w0 = a.w0 * b.sx + a.w1 * b.shy + a.w2 * b.w0;
        self.w1 = a.w0 * b.shx + a.w1 * b.sy + a.w2 * b.w1;
        self.w2 = a.w0 * b.tx + a.w1 * b.ty + a.w2 * b.w2;
        self
    }

    pub fn multiply_affine(&mut self, a: &TransAffine) -> &Self {
        let b = *self;
        self.sx = a.sx * b.sx + a.shx * b.shy + a.tx * b.w0;
        self.shx = a.sx * b.shx + a.shx * b.sy + a.tx * b.w1;
        self.tx = a.sx * b.tx + a.shx * b.ty + a.tx * b.w2;
        self.shy = a.shy * b.sx + a.sy * b.shy + a.ty * b.w0;
        self.sy = a.shy * b.shx + a.sy * b.sy + a.ty * b.w1;
        self.ty = a.shy * b.tx + a.sy * b.ty + a.ty * b.w2;
        self
    }

    pub fn premultiply_affine(&mut self, b: &TransAffine) {
        let a = *self;
        self.sx = a.sx * b.sx + a.shx * b.shy;
        self.shx = a.sx * b.shx + a.shx * b.sy;
        self.tx = a.sx * b.tx + a.shx * b.ty + a.tx;
        self.shy = a.shy * b.sx + a.sy * b.shy;
        self.sy = a.shy * b.shx + a.sy * b.sy;
        self.ty = a.shy * b.tx + a.sy * b.ty + a.ty;
        self.w0 = a.w0 * b.sx + a.w1 * b.shy;
        self.w1 = a.w0 * b.shx + a.w1 * b.sy;
        self.w2 = a.w0 * b.tx + a.w1 * b.ty + a.w2;
    }

    pub fn multiply_inv(&mut self, m: &TransPerspective) {
        let mut t = *m;
        t.invert();
        self.multiply(&t);
    }

    pub fn multiply_inv_affine(&mut self, m: &TransAffine) {
        let mut t = *m;
        t.invert();
        self.multiply_affine(&t);
    }

    pub fn premultiply_inv(&mut self, m: &TransPerspective) {
        let mut t = *m;
        t.invert();
        *self = *t.multiply(self);
    }


    pub fn translate(&mut self, x: f64, y: f64) -> &mut TransPerspective {
        self.tx += x;
        self.ty += y;
        self
    }


    pub fn rotate(&mut self, a: f64) -> &mut TransPerspective {
        self.multiply_affine(&TransAffine::trans_affine_rotation(a));
        self
    }


    pub fn scale_s(&mut self, s: f64) -> &mut TransPerspective {
        self.multiply_affine(&TransAffine::trans_affine_scaling_eq(s));
        self
    }


    pub fn scale_xy(&mut self, x: f64, y: f64) -> &mut TransPerspective {
        self.multiply_affine(&TransAffine::trans_affine_scaling(x, y));
        self
    }


    pub fn transform_affine(&self, x: &mut f64, y: &mut f64) {
        let tmp = *x;
        *x = tmp * self.sx + *y * self.shx + self.tx;
        *y = tmp * self.shy + *y * self.sy + self.ty;
    }


    pub fn transform_2x2(&self, x: &mut f64, y: &mut f64) {
        let tmp = *x;
        *x = tmp * self.sx + *y * self.shx;
        *y = tmp * self.shy + *y * self.sy;
    }


    pub fn inverse_transform(&self, x: &mut f64, y: &mut f64) {
        let mut t = *self;
        if t.invert() {
            t.transform(x, y);
        }
    }


    pub fn store_to(&self, m: &mut [f64]) {
        m[0] = self.sx;
        m[1] = self.shy;
        m[2] = self.w0;
        m[3] = self.shx;
        m[4] = self.sy;
        m[5] = self.w1;
        m[6] = self.tx;
        m[7] = self.ty;
        m[8] = self.w2;
    }


    pub fn load_from(&mut self, m: &[f64]) -> &mut TransPerspective {
        self.sx = m[0];
        self.shy = m[1];
        self.w0 = m[2];
        self.shx = m[3];
        self.sy = m[4];
        self.w1 = m[5];
        self.tx = m[6];
        self.ty = m[7];
        self.w2 = m[8];
        self
    }


    pub fn from_affine(&mut self, a: &TransAffine) -> &mut TransPerspective {
        self.sx = a.sx;
        self.shy = a.shy;
        self.w0 = 0.0;
        self.shx = a.shx;
        self.sy = a.sy;
        self.w1 = 0.0;
        self.tx = a.tx;
        self.ty = a.ty;
        self.w2 = 1.0;
        self
    }


    pub fn determinant(&self) -> f64 {
        self.sx * (self.sy * self.w2 - self.ty * self.w1)
            + self.shx * (self.ty * self.w0 - self.shy * self.w2)
            + self.tx * (self.shy * self.w1 - self.sy * self.w0)
    }


    pub fn determinant_reciprocal(&self) -> f64 {
        1.0 / self.determinant()
    }

    pub fn is_valid(&self, epsilon: f64) -> bool {
        (self.sx.abs() > epsilon) && (self.sy.abs() > epsilon) && (self.w2.abs() > epsilon)
    }

    pub fn is_identity(&self, epsilon: f64) -> bool {
        is_equal_eps(self.sx, 1.0, epsilon)
            && is_equal_eps(self.shy, 0.0, epsilon)
            && is_equal_eps(self.w0, 0.0, epsilon)
            && is_equal_eps(self.shx, 0.0, epsilon)
            && is_equal_eps(self.sy, 1.0, epsilon)
            && is_equal_eps(self.w1, 0.0, epsilon)
            && is_equal_eps(self.tx, 0.0, epsilon)
            && is_equal_eps(self.ty, 0.0, epsilon)
            && is_equal_eps(self.w2, 1.0, epsilon)
    }

    pub fn is_equal(&self, m: &TransPerspective, epsilon: f64) -> bool {
        is_equal_eps(self.sx, m.sx, epsilon)
            && is_equal_eps(self.shy, m.shy, epsilon)
            && is_equal_eps(self.w0, m.w0, epsilon)
            && is_equal_eps(self.shx, m.shx, epsilon)
            && is_equal_eps(self.sy, m.sy, epsilon)
            && is_equal_eps(self.w1, m.w1, epsilon)
            && is_equal_eps(self.tx, m.tx, epsilon)
            && is_equal_eps(self.ty, m.ty, epsilon)
            && is_equal_eps(self.w2, m.w2, epsilon)
    }

    pub fn scale(&self) -> f64 {
        let x = 0.707106781 * self.sx + 0.707106781 * self.shx;
        let y = 0.707106781 * self.shy + 0.707106781 * self.sy;
        (x * x + y * y).sqrt()
    }

    pub fn rotation(&self) -> f64 {
        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut x2 = 1.0;
        let mut y2 = 0.0;
        self.transform(&mut x1, &mut y1);
        self.transform(&mut x2, &mut y2);
        y2.atan2(x2)
    }

    pub fn translation(&self) -> (f64, f64) {
        (self.tx, self.ty)
    }

    pub fn scaling(&self) -> (f64, f64) {
        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut x2 = 1.0;
        let mut y2 = 1.0;
        let mut t = *self;
        //t *= TransAffine::trans_affine_rotation(-self.rotation());
        t.multiply_affine(&TransAffine::trans_affine_rotation(-self.rotation()));
        t.transform(&mut x1, &mut y1);
        t.transform(&mut x2, &mut y2);
        (x2 - x1, y2 - y1)
    }

	pub fn begin(&self, px: f64, py:f64, step: f64) -> IteratorX {
		IteratorX::new(px, py, step, self)
	}
}

#[derive(Default)]
pub struct IteratorX {
    den: f64,
    den_step: f64,
    nom_x: f64,
    nom_x_step: f64,
    nom_y: f64,
    nom_y_step: f64,
    pub x: f64,
    pub y: f64,
}

impl IteratorX {
	pub fn new_default() -> Self {
		Self {
			..Default::default()
		}
	}
    
	pub fn new(px: f64, py: f64, step: f64, m: &TransPerspective) -> IteratorX {
		let den = px * m.w0 + py * m.w1 + m.w2;
		let nx = px * m.sx + py * m.shx + m.tx;
		let ny = px * m.shy + py * m.sy + m.ty;
        IteratorX {
            den,
            den_step: m.w0 * step,
            nom_x: nx,
            nom_x_step: step * m.sx,
            nom_y: ny,
            nom_y_step: step * m.shy,
            x: nx / den,
            y: ny / den,
        }
    }

    pub fn inc(&mut self) {
        self.den += self.den_step;
        self.nom_x += self.nom_x_step;
        self.nom_y += self.nom_y_step;
        let d = 1.0 / self.den;
        self.x = self.nom_x * d;
        self.y = self.nom_y * d;
    }
}

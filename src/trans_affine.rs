// Affine transformation are linear transformations in Cartesian coordinates
// (strictly speaking not only in Cartesian, but for the beginning we will
// think so). They are rotation, scaling, translation and skewing.
// After any affine transformation a line segment remains a line segment
// and it will never become a curve.
//
// There will be no math about matrix calculations, since it has been
// described many times. Ask yourself a very simple question:
// "why do we need to understand and use some matrix stuff instead of just
// rotating, scaling and so on". The answers are:
//
// 1. Any combination of transformations can be done by only 4 multiplications
//    and 4 additions in floating point.
// 2. One matrix transformation is equivalent to the number of consecutive
//    discrete transformations, i.e. the matrix "accumulates" all transformations
//    in the order of their settings. Suppose we have 4 transformations:
//       * rotate by 30 degrees,
//       * scale X to 2.0,
//       * scale Y to 1.5,
//       * move to (100, 100).
//    The result will depend on the order of these transformations,
//    and the advantage of matrix is that the sequence of discret calls:
//    rotate(30), scaleX(2.0), scaleY(1.5), move(100,100)
//    will have exactly the same result as the following matrix transformations:
//
//    affine_matrix m;
//    m *= rotate_matrix(30);
//    m *= scaleX_matrix(2.0);
//    m *= scaleY_matrix(1.5);
//    m *= move_matrix(100,100);
//
//    m.transform_my_point_at_last(x, y);
//
// What is the good of it? In real life we will set-up the matrix only once
// and then transform many points, let alone the convenience to set any
// combination of transformations.
//
// So, how to use it? Very easy - literally as it's shown above. Not quite,
// let us write a correct example:
//
// agg::TransAffine m;
// m *= agg::trans_affine_rotation(30.0 * 3.1415926 / 180.0);
// m *= agg::trans_affine_scaling(2.0, 1.5);
// m *= agg::trans_affine_translation(100.0, 100.0);
// m.transform(&x, &y);
//
// The affine matrix is all you need to perform any linear transformation,
// but all transformations have origin point (0,0). It means that we need to
// use 2 translations if we want to rotate someting around (100,100):
//
// m *= agg::trans_affine_translation(-100.0, -100.0);         // move to (0,0)
// m *= agg::trans_affine_rotation(30.0 * 3.1415926 / 180.0);  // rotate
// m *= agg::trans_affine_translation(100.0, 100.0);           // move back to (100,100)
//----------------------------------------------------------------------
use std::cmp::PartialEq;
use std::ops::{Div, DivAssign, Mul, MulAssign, Neg};

use crate::{basics::is_equal_eps, Transformer};

pub const AFFINE_EPSILON: f64 = 1e-14;
#[derive(Clone, Copy)]
pub struct TransAffine {
    pub sx: f64,
    pub shy: f64,
    pub shx: f64,
    pub sy: f64,
    pub tx: f64,
    pub ty: f64,
}

impl Transformer for TransAffine {
    // Direct transformation of x and y
    #[inline]
    fn transform(&self, x: &mut f64, y: &mut f64) {
        let tmp = *x;
        *x = tmp * self.sx + *y * self.shx + self.tx;
        *y = tmp * self.shy + *y * self.sy + self.ty;
    }

    #[inline]
    fn scaling_abs(&self, x: &mut f64, y: &mut f64) {
        // Used to calculate scaling coefficients in image resampling.
        // When there is considerable shear this method gives us much
        // better estimation than just sx, sy.
        *x = (self.sx * self.sx + self.shx * self.shx).sqrt();
        *y = (self.shy * self.shy + self.sy * self.sy).sqrt();
    }
}

impl TransAffine {
    pub fn new_default() -> TransAffine {
        TransAffine {
            sx: 1.0,
            shy: 0.0,
            shx: 0.0,
            sy: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
    pub fn new(v0: f64, v1: f64, v2: f64, v3: f64, v4: f64, v5: f64) -> TransAffine {
        TransAffine {
            sx: v0,
            shy: v1,
            shx: v2,
            sy: v3,
            tx: v4,
            ty: v5,
        }
    }
    pub fn new_from_array(m: &[f64]) -> TransAffine {
        TransAffine {
            sx: m[0],
            shy: m[1],
            shx: m[2],
            sy: m[3],
            tx: m[4],
            ty: m[5],
        }
    }

    //====================================================trans_affine_rotation
    // Rotation matrix. sin() and cos() are calculated twice for the same angle.
    // There's no harm because the performance of sin()/cos() is very good on all
    // modern processors. Besides, this operation is not going to be invoked too
    // often.
    pub fn trans_affine_rotation(a: f64) -> TransAffine {
        TransAffine::new(a.cos(), a.sin(), -a.sin(), a.cos(), 0.0, 0.0)
    }

    // Translation matrix
    pub fn trans_affine_translation(x: f64, y: f64) -> TransAffine {
        TransAffine::new(1.0, 0.0, 0.0, 1.0, x, y)
    }

    // Sckewing (shear) matrix
    pub fn trans_affine_skewing(x: f64, y: f64) -> TransAffine {
        TransAffine::new(1.0, y.tan(), x.tan(), 1.0, 0., 0.)
    }

    //====================================================trans_affine_scaling
    // Scaling matrix. x, y - scale coefficients by X and Y respectively
    pub fn trans_affine_scaling(x: f64, y: f64) -> TransAffine {
        TransAffine::new(x, 0.0, 0.0, y, 0.0, 0.0)
    }

    pub fn trans_affine_scaling_eq(s: f64) -> TransAffine {
        TransAffine::new(s, 0.0, 0.0, s, 0.0, 0.0)
    }

    //===============================================trans_affine_line_segment
    // Rotate, Scale and Translate, associating 0...dist with line segment
    // x1,y1,x2,y2
    pub fn trans_affine_line_segment(x1: f64, y1: f64, x2: f64, y2: f64, dist: f64) -> TransAffine {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let mut t = TransAffine::new_default();
        if dist > 0.0 {
            t.multiply(&Self::trans_affine_scaling_eq(
                (dx * dx + dy * dy).sqrt() / dist,
            ));
        }
        t.multiply(&Self::trans_affine_rotation(dy.atan2(dx)));
        t.multiply(&Self::trans_affine_translation(x1, y1));
        t
    }

    //============================================trans_affine_reflection_unit
    // Reflection matrix. Reflect coordinates across the line through
    // the origin containing the unit vector (ux, uy).
    // Contributed by John Horigan
    pub fn trans_affine_reflection_unit(ux: f64, uy: f64) -> TransAffine {
        TransAffine::new(
            2.0 * ux * ux - 1.0,
            2.0 * ux * uy,
            2.0 * ux * uy,
            2.0 * uy * uy - 1.0,
            0.0,
            0.0,
        )
    }

    //=================================================trans_affine_reflection
    // Reflection matrix. Reflect coordinates across the line through
    // the origin at the angle a or containing the non-unit vector (x, y).
    // Contributed by John Horigan
    pub fn trans_affine_reflection_angle(a: f64) -> TransAffine {
        Self::trans_affine_reflection_unit(a.cos(), a.sin())
    }

    pub fn trans_affine_reflection_xy(x: f64, y: f64) -> TransAffine {
        Self::trans_affine_reflection_unit(x / (x * x + y * y).sqrt(), y / (x * x + y * y).sqrt())
    }

    // Rectangle to a parallelogram.
    pub fn new_parl(x1: f64, y1: f64, x2: f64, y2: f64, parl: &[f64]) -> TransAffine {
        Self::rect_to_parl(x1, y1, x2, y2, parl)
    }

    // Parallelogram to a rectangle.
    pub fn new_rect(parl: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) -> TransAffine {
        Self::parl_to_rect(parl, x1, y1, x2, y2)
    }

    // Arbitrary parallelogram transformation.
    pub fn new_arb_parl(src: &[f64], dst: &[f64]) -> TransAffine {
        Self::parl_to_parl(src, dst)
    }

    //---------------------------------- Parellelogram transformations
    // transform a parallelogram to another one. Src and dst are
    // pointers to arrays of three points (double[6], x1,y1,...) that
    // identify three corners of the parallelograms assuming implicit
    // fourth point. The arguments are arrays of double[6] mapped
    // to x1,y1, x2,y2, x3,y3  where the coordinates are:
    //        *-----------------*
    //       /          (x3,y3)/
    //      /                 /
    //     /(x1,y1)   (x2,y2)/
    //    *-----------------*
    //------------------------------------------------------------------------
    pub fn parl_to_parl(src: &[f64], dst: &[f64]) -> TransAffine {
        let mut r = Self {
            sx: src[2] - src[0],
            shy: src[3] - src[1],
            shx: src[4] - src[0],
            sy: src[5] - src[1],
            tx: src[0],
            ty: src[1],
        };

        r.invert();
        r.multiply(&TransAffine {
            sx: dst[2] - dst[0],
            shy: dst[3] - dst[1],
            shx: dst[4] - dst[0],
            sy: dst[5] - dst[1],
            tx: dst[0],
            ty: dst[1],
        });
        r
    }

    pub fn rect_to_parl(x1: f64, y1: f64, x2: f64, y2: f64, parl: &[f64]) -> TransAffine {
        let src: [f64; 6] = [x1, y1, x2, y1, x2, y2];
        Self::parl_to_parl(&src, parl)
    }

    pub fn parl_to_rect(parl: &[f64], x1: f64, y1: f64, x2: f64, y2: f64) -> TransAffine {
        let dst: [f64; 6] = [x1, y1, x2, y1, x2, y2];
        Self::parl_to_parl(parl, &dst)
    }

    //------------------------------------------- Load/Store
    // Store matrix to an array [6] of double
    pub fn store_to(&self, m: &mut [f64; 6]) {
        m[0] = self.sx;
        m[1] = self.shy;
        m[2] = self.shx;
        m[3] = self.sy;
        m[4] = self.tx;
        m[5] = self.ty;
    }

    // Load matrix from an array [6] of double
    pub fn load_from(&mut self, m: &mut [f64; 6]) -> &TransAffine {
        self.sx = m[0];
        self.shy = m[1];
        self.shx = m[2];
        self.sy = m[3];
        self.tx = m[4];
        self.ty = m[5];
        self
    }

    //-------------------------------------------- Auxiliary
    // Calculate the determinant of matrix
    pub fn determinant(&self) -> f64 {
        self.sx * self.sy - self.shy * self.shx
    }

    // Calculate the reciprocal of the determinant
    pub fn determinant_reciprocal(&self) -> f64 {
        1.0 / (self.sx * self.sy - self.shy * self.shx)
    }

    //-------------------------------------------- Transformations

    // Direct transformation of x and y, 2x2 matrix only, no translation
    #[inline]
    pub fn transform_2x2(&self, x: &mut f64, y: &mut f64) {
        let tmp = *x;
        *x = tmp * self.sx + *y * self.shx;
        *y = tmp * self.shy + *y * self.sy;
    }

    // Inverse transformation of x and y. It works slower than the
    // direct transformation. For massive operations it's better to
    // invert() the matrix and then use direct transformations.
    #[inline]
    pub fn inverse_transform(&self, x: &mut f64, y: &mut f64) {
        let d = self.determinant_reciprocal();
        let a = (*x - self.tx) * d;
        let b = (*y - self.ty) * d;
        *x = a * self.sy - b * self.shx;
        *y = b * self.sx - a * self.shy;
    }

    // Get the average scale (by X and Y).
    // Basically used to calculate the approximation_scale when
    // decomposinting curves into line segments.
    #[inline]
    pub fn scale(&self) -> f64 {
        let x = 0.707106781 * self.sx + 0.707106781 * self.shx;
        let y = 0.707106781 * self.shy + 0.707106781 * self.sy;
        (x * x + y * y).sqrt()
    }

    // Multiply matrix to another one
    pub fn multiply(&mut self, m: &TransAffine) -> &TransAffine {
        let t0 = self.sx * m.sx + self.shy * m.shx;
        let t2 = self.shx * m.sx + self.sy * m.shx;
        let t4 = self.tx * m.sx + self.ty * m.shx + m.tx;
        self.shy = self.sx * m.shy + self.shy * m.sy;
        self.sy = self.shx * m.shy + self.sy * m.sy;
        self.ty = self.tx * m.shy + self.ty * m.sy + m.ty;
        self.sx = t0;
        self.shx = t2;
        self.tx = t4;
        self
    }

    // Invert matrix. Do not try to invert degenerate matrices,
    // there's no check for validity. If you set scale to 0 and
    // then try to invert matrix, expect unpredictable result.
    pub fn invert(&mut self) -> &TransAffine {
        let d = self.determinant_reciprocal();

        let t0 = self.sy * d;
        self.sy = self.sx * d;
        self.shy = -self.shy * d;
        self.shx = -self.shx * d;

        let t4 = -self.tx * t0 - self.ty * self.shx;
        self.ty = -self.tx * self.shy - self.ty * self.sy;

        self.sx = t0;
        self.tx = t4;
        self
    }

    // Direct transformations operations
    #[inline]
    pub fn translate(&mut self, x: &f64, y: &f64) -> &TransAffine {
        self.tx += x;
        self.ty += y;
        self
    }

    #[inline]
    pub fn rotate(&mut self, a: &f64) -> &TransAffine {
        let ca = (a).cos();
        let sa = (a).sin();
        let t0 = self.sx * ca - self.shy * sa;
        let t2 = self.shx * ca - self.sy * sa;
        let t4 = self.tx * ca - self.ty * sa;
        self.shy = self.sx * sa + self.shy * ca;
        self.sy = self.shx * sa + self.sy * ca;
        self.ty = self.tx * sa + self.ty * ca;
        self.sx = t0;
        self.shx = t2;
        self.tx = t4;
        self
    }

    #[inline]
    pub fn scale_xy(&mut self, x: &f64, y: &f64) -> &TransAffine {
        let mm0 = x; // Possible hint for the optimizer
        let mm3 = y;
        self.sx *= mm0;
        self.shx *= mm0;
        self.tx *= mm0;
        self.shy *= mm3;
        self.sy *= mm3;
        self.ty *= mm3;
        self
    }

    #[inline]
    pub fn scale_s(&mut self, s: &f64) -> &TransAffine {
        let m = s; // Possible hint for the optimizer
        self.sx *= m;
        self.shx *= m;
        self.tx *= m;
        self.shy *= m;
        self.sy *= m;
        self.ty *= m;
        self
    }

    // Multiply "m" to "this" and assign the result to "this"
    #[inline]
    pub fn premultiply(&mut self, m: &TransAffine) -> &TransAffine {
        let mut t = m.clone();
        let c = self.clone();
        t.multiply(&c);
        *self = t;
        self
    }

    // Multiply matrix to inverse of another one
    #[inline]
    pub fn multiply_inv(&mut self, m: &TransAffine) -> &TransAffine {
        let mut t = m.clone();
        t.invert();
        self.multiply(&t);
        self
    }

    // Multiply inverse of "m" to "this" and assign the result to "this"
    #[inline]
    pub fn premultiply_inv(&mut self, m: &mut TransAffine) -> &TransAffine {
        let mut t = m.clone();
        t.invert();
        let c = self.clone();
        t.multiply(&c);
        *self = t;
        self
    }

    // Mirroring around X
    pub fn flip_x(&mut self) -> &TransAffine {
        self.sx = -self.sx;
        self.shy = -self.shy;
        self.tx = -self.tx;
        self
    }

    // Mirroring around X
    pub fn flip_y(&mut self) -> &TransAffine {
        self.shx = -self.shx;
        self.sy = -self.sy;
        self.ty = -self.ty;
        self
    }

    // Reset - load an identity matrix
    pub fn reset(&mut self) -> &TransAffine {
        self.sx = 1.0;
        self.sy = 1.0;
        self.shy = 0.0;
        self.shx = 0.0;
        self.tx = 0.0;
        self.ty = 0.0;
        self
    }

    // Check to see if it's an identity matrix
    pub fn is_identity(&self, epsilon: f64) -> bool {
        is_equal_eps(self.sx, 1.0, epsilon)
            && is_equal_eps(self.shy, 0.0, epsilon)
            && is_equal_eps(self.shx, 0.0, epsilon)
            && is_equal_eps(self.sy, 1.0, epsilon)
            && is_equal_eps(self.tx, 0.0, epsilon)
            && is_equal_eps(self.ty, 0.0, epsilon)
    }

    // Check to see if the matrix is not degenerate
    pub fn is_valid(&self, epsilon: f64) -> bool {
        (self.sx).abs() > epsilon && (self.sy).abs() > epsilon
    }

    // Check to see if two matrices are equal
    pub fn is_equal(&self, m: &TransAffine, epsilon: f64) -> bool {
        is_equal_eps(self.sx, m.sx, epsilon)
            && is_equal_eps(self.shy, m.shy, epsilon)
            && is_equal_eps(self.shx, m.shx, epsilon)
            && is_equal_eps(self.sy, m.sy, epsilon)
            && is_equal_eps(self.tx, m.tx, epsilon)
            && is_equal_eps(self.ty, m.ty, epsilon)
    }

    // Determine the major parameters. Use with caution considering
    // possible degenerate cases.

    pub fn rotation(&self) -> f64 {
        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut x2 = 1.0;
        let mut y2 = 0.0;
        self.transform(&mut x1, &mut y1);
        self.transform(&mut x2, &mut y2);
        (y2 - y1).atan2(x2 - x1)
    }

    pub fn translation(&self, dx: &mut f64, dy: &mut f64) {
        *dx = self.tx;
        *dy = self.ty;
    }

    pub fn scaling(&self, x: &mut f64, y: &mut f64) {
        let mut x1 = 0.0;
        let mut y1 = 0.0;
        let mut x2 = 1.0;
        let mut y2 = 1.0;
        let mut t = self.clone();
        t *= Self::trans_affine_rotation(-self.rotation());
        t.transform(&mut x1, &mut y1);
        t.transform(&mut x2, &mut y2);
        *x = x2 - x1;
        *y = y2 - y1;
    }
}

// Multiply the matrix by another one
impl MulAssign for TransAffine {
    fn mul_assign(&mut self, rhs: TransAffine) {
        self.multiply(&rhs);
    }
}

// Multiply the matrix by inverse of another one
impl DivAssign for TransAffine {
    fn div_assign(&mut self, rhs: TransAffine) {
        self.multiply_inv(&rhs);
    }
}

// Multiply the matrix by another one and return
// the result in a separete matrix.
impl Mul for TransAffine {
    type Output = TransAffine;
    fn mul(self, rhs: Self) -> Self {
        let mut tmp = self;
        tmp.multiply(&rhs);
        tmp
    }
}

// Multiply the matrix by inverse of another one
// and return the result in a separete matrix.
impl Div for TransAffine {
    type Output = TransAffine;
    fn div(self, rhs: Self) -> Self {
        let mut tmp = self;
        tmp.multiply_inv(&rhs);
        tmp
    }
}

// Calculate and return the inverse matrix
impl Neg for TransAffine {
    type Output = TransAffine;
    fn neg(self) -> TransAffine {
        let mut tmp = self;
        tmp.invert();
        tmp
    }
}

impl PartialEq for TransAffine {
    // Equal operator with default epsilon
    fn eq(&self, m: &TransAffine) -> bool {
        self.is_equal(m, AFFINE_EPSILON)
    }

    // Not Equal operator with default epsilon
    fn ne(&self, m: &TransAffine) -> bool {
        !self.is_equal(m, AFFINE_EPSILON)
    }
}

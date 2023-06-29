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
// Bessel function (besj) was adapted for use in AGG library by Andy Wilk
// Contact: castor.vulgaris@gmail.com
//----------------------------------------------------------------------------

use crate::vertex_sequence::{VecSequence, VertexDist};
use crate::VertexSequence;

//------------------------------------------------------VERTEX_DIST_EPSILON
// Coinciding points maximal distance (Epsilon)
pub const VERTEX_DIST_EPSILON: f64 = 1e-14;

//-----------------------------------------------------INTERSECTION_EPSILON
// See calc_intersection
const INTERSECTION_EPSILON: f64 = 1.0e-30;

//-------------------------------------------------------calc_polygon_area
//pub fn calc_polygon_area<Storage>(st: &Storage) -> f64
pub fn calc_polygon_area(st: &VecSequence<VertexDist>) -> f64 {
    let mut sum = 0.0;
    let mut x = st[0].x;
    let mut y = st[0].y;
    let xs = x;
    let ys = y;

    for i in 1..st.size() {
        let v = st[i];
        sum += x * v.y - y * v.x;
        x = v.x;
        y = v.y;
    }
    return (sum + x * ys - y * xs) * 0.5;
}

//------------------------------------------------------------cross_product
#[inline]
pub fn cross_product(x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> f64 {
    return (x - x2) * (y2 - y1) - (y - y2) * (x2 - x1);
}

//--------------------------------------------------------point_in_triangle
#[inline]
pub fn point_in_triangle(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x: f64, y: f64,
) -> bool {
    let cp1 = cross_product(x1, y1, x2, y2, x, y) < 0.0;
    let cp2 = cross_product(x2, y2, x3, y3, x, y) < 0.0;
    let cp3 = cross_product(x3, y3, x1, y1, x, y) < 0.0;
    return cp1 == cp2 && cp2 == cp3 && cp3 == cp1;
}

//-----------------------------------------------------------calc_distance
#[inline]
pub fn calc_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return (dx * dx + dy * dy).sqrt();
}

//--------------------------------------------------------calc_sq_distance
#[inline]
pub fn calc_sq_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    return dx * dx + dy * dy;
}

//------------------------------------------------calc_line_point_distance
#[inline]
pub fn calc_line_point_distance(x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let d = (dx * dx + dy * dy).sqrt();
    if d < VERTEX_DIST_EPSILON {
        return calc_distance(x1, y1, x, y);
    }
    return ((x - x2) * dy - (y - y2) * dx) / d;
}

//-------------------------------------------------------calc_line_point_u
#[inline]
pub fn calc_segment_point_u(x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;

    if dx == 0. && dy == 0. {
        return 0.;
    }

    let pdx = x - x1;
    let pdy = y - y1;

    return (pdx * dx + pdy * dy) / (dx * dx + dy * dy);
}

//---------------------------------------------calc_line_point_sq_distance
#[inline]
pub fn calc_segment_point_sq_distance_u(
    x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64, u: f64,
) -> f64 {
    if u <= 0. {
        return calc_sq_distance(x, y, x1, y1);
    } else if u >= 1. {
        return calc_sq_distance(x, y, x2, y2);
    }
    return calc_sq_distance(x, y, x1 + u * (x2 - x1), y1 + u * (y2 - y1));
}

//---------------------------------------------calc_line_point_sq_distance
#[inline]
pub fn calc_segment_point_sq_distance(x1: f64, y1: f64, x2: f64, y2: f64, x: f64, y: f64) -> f64 {
    return calc_segment_point_sq_distance_u(
        x1,
        y1,
        x2,
        y2,
        x,
        y,
        calc_segment_point_u(x1, y1, x2, y2, x, y),
    );
}

//-------------------------------------------------------calc_intersection
#[inline]
pub fn calc_intersection(
    ax: f64, ay: f64, bx: f64, by: f64, cx: f64, cy: f64, dx: f64, dy: f64, x: &mut f64,
    y: &mut f64,
) -> bool {
    let num = (ay - cy) * (dx - cx) - (ax - cx) * (dy - cy);
    let den = (bx - ax) * (dy - cy) - (by - ay) * (dx - cx);
    if (den.abs()) < INTERSECTION_EPSILON {
        return false;
    }
    let r = num / den;
    *x = ax + r * (bx - ax);
    *y = ay + r * (by - ay);
    return true;
}

//-----------------------------------------------------intersection_exists
#[inline]
pub fn intersection_exists(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x4: f64, y4: f64,
) -> bool {
    // It's less expensive but you can't control the
    // boundary conditions: Less or LessEqual
    let dx1 = x2 - x1;
    let dy1 = y2 - y1;
    let dx2 = x4 - x3;
    let dy2 = y4 - y3;
    return ((x3 - x2) * dy1 - (y3 - y2) * dx1 < 0.0) != ((x4 - x2) * dy1 - (y4 - y2) * dx1 < 0.0)
        && ((x1 - x4) * dy2 - (y1 - y4) * dx2 < 0.0) != ((x2 - x4) * dy2 - (y2 - y4) * dx2 < 0.0);

    // It's is more expensive but more flexible
    // in terms of boundary conditions.
    //--------------------
    //double den  = (x2-x1) * (y4-y3) - (y2-y1) * (x4-x3);
    //if(fabs(den) < INTERSECTION_EPSILON) return false;
    //double nom1 = (x4-x3) * (y1-y3) - (y4-y3) * (x1-x3);
    //double nom2 = (x2-x1) * (y1-y3) - (y2-y1) * (x1-x3);
    //double ua = nom1 / den;
    //double ub = nom2 / den;
    //return ua >= 0.0 && ua <= 1.0 && ub >= 0.0 && ub <= 1.0;
}

//--------------------------------------------------------calc_orthogonal
#[inline]
pub fn calc_orthogonal(
    thickness: f64, x1: f64, y1: f64, x2: f64, y2: f64, x: &mut f64, y: &mut f64,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let d = (dx * dx + dy * dy).sqrt();
    *x = thickness * dy / d;
    *y = -thickness * dx / d;
}

//--------------------------------------------------------dilate_triangle
#[inline]
pub fn dilate_triangle(
    x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, x: &mut [f64], y: &mut [f64], d_: f64,
) {
    let mut dx1 = 0.0;
    let mut dy1 = 0.0;
    let mut dx2 = 0.0;
    let mut dy2 = 0.0;
    let mut dx3 = 0.0;
    let mut dy3 = 0.0;
    let mut d = d_;
    let loc = cross_product(x1, y1, x2, y2, x3, y3);
    if (loc).abs() > INTERSECTION_EPSILON {
        if cross_product(x1, y1, x2, y2, x3, y3) > 0.0 {
            d = -d;
        }
        calc_orthogonal(d, x1, y1, x2, y2, &mut dx1, &mut dy1);
        calc_orthogonal(d, x2, y2, x3, y3, &mut dx2, &mut dy2);
        calc_orthogonal(d, x3, y3, x1, y1, &mut dx3, &mut dy3);
    }
    x[0] = x1 + dx1;
    y[0] = y1 + dy1;
    x[1] = x2 + dx1;
    y[1] = y2 + dy1;
    x[2] = x2 + dx2;
    y[2] = y2 + dy2;
    x[3] = x3 + dx2;
    y[3] = y3 + dy2;
    x[4] = x3 + dx3;
    y[4] = y3 + dy3;
    x[5] = x1 + dx3;
    y[5] = y1 + dy3;
}

//------------------------------------------------------calc_triangle_area
#[inline]
pub fn calc_triangle_area(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> f64 {
    return (x1 * y2 - x2 * y1 + x2 * y3 - x3 * y2 + x3 * y1 - x1 * y3) * 0.5;
}

/*
//------------------------------------------------------------------------
// Tables for fast sqrt
extern int16u g_sqrt_table[1024];
extern int8   g_elder_bit_table[256];


//---------------------------------------------------------------fast_sqrt
//Fast integer Sqrt - really fast: no cycles, divisions or multiplications
#if defined(_MSC_VER)
#pragma warning(push)
#pragma warning(disable : 4035) //Disable warning "no return value"
#endif
#[inline]
unsigned fast_sqrt(unsigned val)
{
#if defined(_M_IX86) && defined(_MSC_VER) && !defined(AGG_NO_ASM)
    //For Ix86 family processors this assembler code is used.
    //The key command here is bsr - determination the number of the most
    //significant bit of the value. For other processors
    //(and maybe compilers) the pure C "#else" section is used.
    __asm
    {
        mov ebx, val
        mov edx, 11
        bsr ecx, ebx
        sub ecx, 9
        jle less_than_9_bits
        shr ecx, 1
        adc ecx, 0
        sub edx, ecx
        shl ecx, 1
        shr ebx, cl
less_than_9_bits:
        xor eax, eax
        mov  ax, g_sqrt_table[ebx*2]
        mov ecx, edx
        shr eax, cl
    }
#else

    //This code is actually pure C and portable to most
    //arcitectures including 64bit ones.
    unsigned t = val;
    int bit=0;
    unsigned shift = 11;

    //The following piece of code is just an emulation of the
    //Ix86 assembler command "bsr" (see above). However on old
    //Intels (like Intel MMX 233MHz) this code is about twice
    //faster (sic!) then just one "bsr". On PIII and PIV the
    //bsr is optimized quite well.
    bit = t >> 24;
    if(bit)
    {
        bit = g_elder_bit_table[bit] + 24;
    }
    else
    {
        bit = (t >> 16) & 0xFF;
        if(bit)
        {
            bit = g_elder_bit_table[bit] + 16;
        }
        else
        {
            bit = (t >> 8) & 0xFF;
            if(bit)
            {
                bit = g_elder_bit_table[bit] + 8;
            }
            else
            {
                bit = g_elder_bit_table[t];
            }
        }
    }

    //This code calculates the sqrt.
    bit -= 9;
    if(bit > 0)
    {
        bit = (bit >> 1) + (bit & 1);
        shift -= bit;
        val >>= (bit << 1);
    }
    return g_sqrt_table[val] >> shift;
#endif
}
#if defined(_MSC_VER)
#pragma warning(pop)
#endif

*/

//--------------------------------------------------------------------besj
// Function BESJ calculates Bessel function of first kind of order n
// Arguments:
//     n - an integer (>=0), the order
//     x - value at which the Bessel function is required
//--------------------
// C++ Mathematical Library
// Convereted from equivalent FORTRAN library
// Converetd by Gareth Walker for use by course 392 computational project
// All functions tested and yield the same results as the corresponding
// FORTRAN versions.
//
// If you have any problems using these functions please report them to
// M.Muldoon@UMIST.ac.uk
//
// Documentation available on the web
// http://www.ma.umist.ac.uk/mrm/Teaching/392/libs/392.html
// Version 1.0   8/98
// 29 October, 1999
//--------------------
// Adapted for use in AGG library by Andy Wilk (castor.vulgaris@gmail.com)
//------------------------------------------------------------------------
#[inline]
pub fn besj(x: f64, n: i32) -> f64 {
    if n < 0 {
        return 0.;
    }
    let d = 1E-6;
    let mut b = 0.;
    if (x).abs() <= d {
        if n != 0 {
            return 0.;
        }
        return 1.;
    }
    let mut b1 = 0.; // b1 is the value from the previous iteration
                     // Set up a starting order for recurrence
    let mut m1 = (x).abs() as i32 + 6;
    if (x).abs() > 5. {
        m1 = ((1.4 * x + 60. / x).abs()) as i32;
    }
    let mut m2 = (n as f64 + 2. + (x).abs() / 4.) as i32;
    if m1 > m2 {
        m2 = m1;
    }

    // Apply recurrence down from curent max order
    loop {
        let mut c3 = 0.;
        let mut c2 = 1E-30;
        let mut c4 = 0.;
        let mut m8 = 1;
        if m2 / 2 * 2 == m2 {
            m8 = -1;
        }
        let imax = m2 - 2;
        for i in 1..=imax {
            let c6 = 2. * (m2 - i) as f64 * c2 / x - c3;
            c3 = c2;
            c2 = c6;
            if m2 - i - 1 == n {
                b = c6;
            }
            m8 = -1 * m8;
            if m8 > 0 {
                c4 = c4 + 2. * c6;
            }
        }
        let c6 = 2. * c2 / x - c3;
        if n == 0 {
            b = c6;
        }
        c4 += c6;
        b /= c4;
        if (b - b1).abs() < d {
            return b;
        }
        b1 = b;
        m2 += 3;
    }
}

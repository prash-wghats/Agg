use std::marker::PhantomData;

use crate::basics::iround;
use crate::dda_line::DdaLineIp;
use crate::math::cross_product;
use crate::span_gouraud::{CoordD, SpanGouraud};
use crate::Color;
use crate::{AggPrimitive, SpanGenerator, VertexSource, GrayArgs};

const SUBPIXEL_SHIFT: u32 = 4;
const SUBPIXEL_SCALE: u32 = 1 << SUBPIXEL_SHIFT;

struct GrayCalc<C: Color + GrayArgs> {
    x1: f64,
    y1: f64,
    dx: f64,
    m_1dy: f64,
    r1: i32,
    a1: i32,
    dr: i32,
    da: i32,
    r: i32,
    a: i32,
    x: i32,
    dum: PhantomData<C>,
}

impl<C: Color + GrayArgs> GrayCalc<C> {
    fn new() -> Self {
        Self {
            x1: 0.,
            y1: 0.,
            dx: 0.,
            m_1dy: 0.,
            r1: 0,
            a1: 0,
            dr: 0,
            da: 0,
            r: 0,
            a: 0,
            x: 0,
            dum: PhantomData,
        }
    }
    fn init(&mut self, c1: &CoordD<C>, c2: &CoordD<C>) {
        self.x1 = c1.x - 0.5;
        self.y1 = c1.y - 0.5;
        self.dx = c2.x - c1.x;
        let dy = c2.y - c1.y;
        self.m_1dy = if dy < 1e-5 { 1e5 } else { 1.0 / dy };
        self.r1 = c1.color.v().into_u32() as i32;
        self.a1 = c1.color.a().into_u32() as i32;
        self.dr = c2.color.v().into_u32() as i32 - self.r1;
        self.da = c2.color.a().into_u32() as i32 - self.a1;
    }

    fn calc(&mut self, y: f64) {
        let k = (y - self.y1) * self.m_1dy;
        let k = if k < 0.0 {
            0.0
        } else if k > 1.0 {
            1.0
        } else {
            k
        };
        self.r = self.r1 + iround(self.dr as f64 * k);
        self.a = self.a1 + iround(self.da as f64 * k);
        self.x = iround((self.x1 + self.dx as f64 * k) * SUBPIXEL_SCALE as f64);
    }
}
//=======================================================SpanGouraudGray
pub struct SpanGouraudGray<C: Color + GrayArgs> {
    pub m_swap: bool,
    pub m_y2: i32,
    m_rgba1: GrayCalc<C>,
    m_rgba2: GrayCalc<C>,
    m_rgba3: GrayCalc<C>,
    m_base_type: SpanGouraud<C>,
}

impl<C: Color + GrayArgs> SpanGouraudGray<C> {
    pub fn new_default() -> Self {
        SpanGouraudGray {
            m_swap: false,
            m_y2: 0,
            m_rgba1: GrayCalc::new(),
            m_rgba2: GrayCalc::new(),
            m_rgba3: GrayCalc::new(),
            m_base_type: SpanGouraud::new(),
        }
    }

    pub fn new(
        c1: C, c2: C, c3: C, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, d: f64,
    ) -> Self {
        SpanGouraudGray {
            m_swap: false,
            m_y2: 0,
            m_rgba1: GrayCalc::new(),
            m_rgba2: GrayCalc::new(),
            m_rgba3: GrayCalc::new(),
            m_base_type: SpanGouraud::new_with_color(c1, c2, c3, x1, y1, x2, y2, x3, y3, d),
        }
    }

    pub fn set_colors(&mut self, c1: C, c2: C, c3: C) {
        self.m_base_type.colors(c1, c2, c3);
    }
    pub fn set_triangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64, d: f64) {
        self.m_base_type.triangle(x1, y1, x2, y2, x3, y3, d);
    }
}

impl<C: Color + GrayArgs> VertexSource for SpanGouraudGray<C> {
    fn rewind(&mut self, i: u32) {
        self.m_base_type.rewind(i);
    }
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.m_base_type.vertex(x, y)
    }
}

impl<Col: Color + GrayArgs> SpanGenerator for SpanGouraudGray<Col> {
    type C = Col;

    fn prepare(&mut self) {
        let mut coord: [CoordD<Col>; 3] = [CoordD::new(); 3];
        self.m_base_type.arrange_vertices(&mut coord);

        self.m_y2 = coord[1].y as i32;

        self.m_swap = cross_product(
            coord[0].x, coord[0].y, coord[2].x, coord[2].y, coord[1].x, coord[1].y,
        ) < 0.0;

        self.m_rgba1.init(&coord[0], &coord[2]);
        self.m_rgba2.init(&coord[0], &coord[1]);
        self.m_rgba3.init(&coord[1], &coord[2]);
    }

    //--------------------------------------------------------------------
    fn generate(&mut self, span: &mut [Col], x: i32, y: i32, len_: u32) {
        let mut len = len_;
        self.m_rgba1.calc(y as f64);
        let mut pc1 = &mut self.m_rgba1;
        let mut pc2 = &mut self.m_rgba2;
        let mut i: usize = 0;

        let lim = Self::C::BASE_MASK as i32;
        if y <= self.m_y2 {
            // Bottom part of the triangle (first subtriangle)
            //-------------------------
            let tmp = pc2.m_1dy;
            pc2.calc(y as f64 + tmp);
        } else {
            // Upper part (second subtriangle)
            //-------------------------
            self.m_rgba3.calc(y as f64 - self.m_rgba3.m_1dy);
            pc2 = &mut self.m_rgba3;
        }

        if self.m_swap {
            // It means that the triangle is oriented clockwise,
            // so that we need to swap the controlling structures
            //-------------------------
            let t = pc2;
            pc2 = pc1;
            pc1 = t;
        }

        // Get the horizontal length with subpixel accuracy
        // and protect it from division by zero
        //-------------------------
        let mut nlen = (pc2.x - pc1.x).abs() as i32;
        if nlen <= 0 {
            nlen = 1;
        }

        let mut r = DdaLineIp::<14>::new(pc1.r, pc2.r, nlen as u32);
        let mut a = DdaLineIp::<14>::new(pc1.a, pc2.a, nlen as u32);

        // Calculate the starting point of the gradient with subpixel
        // accuracy and correct (roll back) the interpolators.
        // This operation will also clip the beginning of the span
        // if necessary.
        //-------------------------
        let mut start: i32 = pc1.x - (x << SUBPIXEL_SHIFT);
        r.dec_by(start as u32);
        a.dec_by(start as u32);
        nlen += start;

        let mut vr;
        let mut va;
        // Beginning part of the span. Since we rolled back the
        // interpolators, the color values may have overflow.
        // So that, we render the beginning part with checking
        // for overflow. It lasts until "start" is positive;
        // typically it's 1-2 pixels, but may be more in some cases.
        //-------------------------
        while len > 0 && start > 0 {
            vr = r.y();
            va = a.y();
            if vr < 0 {
                vr = 0;
            }
            if vr > lim {
                vr = lim;
            }
            if va < 0 {
                va = 0;
            }
            if va > lim {
                va = lim;
            }

            *span[i].v_mut() = Col::ValueType::from_u32(vr as u32);
            *span[i].a_mut() = Col::ValueType::from_u32(va as u32);
            r.inc_by(SUBPIXEL_SCALE);
            a.inc_by(SUBPIXEL_SCALE);
            nlen -= SUBPIXEL_SCALE as i32;
            start -= SUBPIXEL_SCALE as i32;
            i += 1;
            len -= 1;
        }

        // Middle part, no checking for overflow.
        // Actual spans can be longer than the calculated length
        // because of anti-aliasing, thus, the interpolators can
        // overflow. But while "nlen" is positive we are safe.
        //-------------------------
        while len > 0 && nlen > 0 {
            *span[i].v_mut() = Col::ValueType::from_u32(r.y() as u32);
            *span[i].a_mut() = Col::ValueType::from_u32(a.y() as u32);
            r.inc_by(SUBPIXEL_SCALE);
            a.inc_by(SUBPIXEL_SCALE);
            nlen -= SUBPIXEL_SCALE as i32;
            i += 1;
            len -= 1;
        }

        // Ending part; checking for overflow.
        // Typically it's 1-2 pixels, but may be more in some cases.
        //-------------------------
        while len > 0 {
            vr = r.y();
            va = a.y();
            if vr < 0 {
                vr = 0;
            }
            if vr > lim {
                vr = lim;
            }

            if va < 0 {
                va = 0;
            }
            if va > lim {
                va = lim;
            }
            *span[i].v_mut() = Col::ValueType::from_u32(vr as u32);
            *span[i].a_mut() = Col::ValueType::from_u32(va as u32);
            r.inc_by(SUBPIXEL_SCALE);
            a.inc_by(SUBPIXEL_SCALE);
            i += 1;
            len -= 1;
        }
    }
}

use crate::{AggPrimitive, Args, Color, ImageAccessorRgb, Order, RgbArgs, SpanGenerator};

// NOT TESTED

//========================================================SpanPatternRgb
pub struct SpanPatternRgb<S: ImageAccessorRgb> {
    src: S,
    offset_x: u32,
    offset_y: u32,
    alpha: u8,
}

impl<S: ImageAccessorRgb> SpanPatternRgb<S> {
    pub fn new(src: S, offset_x: u32, offset_y: u32) -> Self {
        SpanPatternRgb {
            src,
            offset_x,
            offset_y,
            alpha: S::ColorType::BASE_MASK as u8,
        }
    }

    pub fn attach(&mut self, src: S) -> S {
        let tmp = std::mem::replace(&mut self.src, src);
        tmp
    }

    pub fn source(&self) -> &S {
        &self.src
    }

    pub fn source_mut(&mut self) -> &mut S {
        &mut self.src
    }

    pub fn offset_x(&self) -> u32 {
        self.offset_x
    }

    pub fn offset_y(&self) -> u32 {
        self.offset_y
    }

    pub fn alpha(&self) -> u8 {
        self.alpha
    }

    pub fn offset_x_mut(&mut self) -> &mut u32 {
        &mut self.offset_x
    }

    pub fn offset_y_mut(&mut self) -> &mut u32 {
        &mut self.offset_y
    }

    pub fn alpha_mut(&mut self) -> &mut u8 {
        &mut self.alpha
    }
}

impl<S: ImageAccessorRgb> SpanGenerator for SpanPatternRgb<S> {
    type C = S::ColorType;

    fn prepare(&mut self) {}

    fn generate(&mut self, span: &mut [S::ColorType], x: i32, y: i32, len: u32) {
        let x = x + self.offset_x as i32;
        let y = y + self.offset_y as i32;
        let mut p = self.src.span(x, y, len).as_ptr() as *const u8
            as *mut <S::ColorType as Args>::ValueType;
        for i in 0..len {
            unsafe {
                *span[i as usize].r_mut() = *p.offset(S::OrderType::R as isize);
                *span[i as usize].g_mut() = *p.offset(S::OrderType::G as isize);
                *span[i as usize].b_mut() = *p.offset(S::OrderType::B as isize);
            }
            *span[i as usize].a_mut() = AggPrimitive::from_u8(self.alpha);
            p = self.src.next_x().as_ptr() as *const u8 as *mut <S::ColorType as Args>::ValueType;
        }
    }
}

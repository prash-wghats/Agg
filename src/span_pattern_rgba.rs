use crate::{AggPrimitive, Args, ImageAccessorRgb, Order, RgbArgs, SpanGenerator};

// NOT TESTED

//======================================================SpanPatternRgba
pub struct SpanPatternRgba<S: ImageAccessorRgb> {
    src: S,
    offset_x: u32,
    offset_y: u32,
}

impl<S: ImageAccessorRgb> SpanPatternRgba<S> {
    pub fn new(src: S, offset_x: u32, offset_y: u32) -> Self {
        SpanPatternRgba {
            src,
            offset_x,
            offset_y,
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

    pub fn set_offset_x(&mut self, x: u32) {
        self.offset_x = x;
    }

    pub fn set_offset_y(&mut self, y: u32) {
        self.offset_y = y;
    }

    pub fn alpha(&self) -> <<S as ImageAccessorRgb>::ColorType as Args>::ValueType {
        AggPrimitive::from_u32(0)
    }

    pub fn set_alpha(&mut self, _a: <<S as ImageAccessorRgb>::ColorType as Args>::ValueType) {}

    pub fn offset_x_mut(&mut self) -> &mut u32 {
        &mut self.offset_x
    }

    pub fn offset_y_mut(&mut self) -> &mut u32 {
        &mut self.offset_y
    }
}
impl<S: ImageAccessorRgb> SpanGenerator for SpanPatternRgba<S> {
    type C = S::ColorType;

    fn prepare(&mut self) {}

    fn generate(&mut self, span: &mut [S::ColorType], x_: i32, y_: i32, len: u32) {
        let x = x_ + self.offset_x as i32;
        let y = y_ + self.offset_y as i32;
        let mut p = self.src.span(x, y, len).as_ptr() as *const u8
            as *mut <S::ColorType as Args>::ValueType;
        for i in 0..len {
            unsafe {
                *span[i as usize].r_mut() = *p.offset(S::OrderType::R as isize);
                *span[i as usize].g_mut() = *p.offset(S::OrderType::G as isize);
                *span[i as usize].b_mut() = *p.offset(S::OrderType::B as isize);
                *span[i as usize].a_mut() = *p.offset(S::OrderType::A as isize);
            }
            p = self.src.next_x().as_ptr() as *const u8 as *mut <S::ColorType as Args>::ValueType;
        }
    }
}

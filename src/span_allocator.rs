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

use crate::{Color, SpanAllocator};

pub struct VecSpan<C: Color> {
    span: Vec<C>,
}

impl<C: Color> VecSpan<C> {
    pub fn new() -> Self {
        VecSpan { span: Vec::new() }
    }

    pub fn span(&self) -> &[C] {
        &self.span[..]
    }

    pub fn max_span_len(&self) -> usize {
        self.span.len()
    }
}
impl<C: Color> SpanAllocator for VecSpan<C> {
    type C = C;
    fn allocate(&mut self, span_len: usize) -> &mut [C] {
        if span_len > self.span.len() {
            // To reduce the number of reallocs we align the
            // span_len to 256 color elements.
            // Well, I just like this number and it looks reasonable.
            //-----------------------
            let co = ((span_len + 255) >> 8) << 8;
            self.span.resize(co, C::new());
        }
        &mut self.span[..span_len]
    }
}

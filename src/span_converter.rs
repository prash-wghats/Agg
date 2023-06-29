

use crate::{SpanGenerator, SpanConverter};
// NOT TESTED


//----------------------------------------------------------SpanProcess
pub struct SpanProcess<'a, Sg: SpanGenerator, Sc: SpanConverter<C= Sg::C>> {
    span_gen: &'a mut Sg,
    span_cnv: &'a mut Sc,
}

impl<'a, Sg: SpanGenerator, Sc: SpanConverter<C= Sg::C>> SpanProcess<'a, Sg, Sc>
{
    pub fn new(span_gen: &'a mut Sg, span_cnv: &'a mut Sc) -> Self {
        SpanProcess {
            span_gen: span_gen,
            span_cnv: span_cnv,
        }
    }

    pub fn attach_generator(&mut self, span_gen: &'a mut Sg) {
        self.span_gen = span_gen;
    }

    pub fn attach_converter(&mut self, span_cnv: &'a mut Sc) {
        self.span_cnv = span_cnv;
    }
}

impl<'a, Sg: SpanGenerator, Sc: SpanConverter<C= Sg::C>> SpanGenerator for SpanProcess<'a, Sg, Sc> {
	type C = Sg::C;

    fn prepare(&mut self) {
        self.span_gen.prepare();
        self.span_cnv.prepare();
    }


    fn generate(&mut self, span: &mut [Sg::C], x: i32, y: i32, len: u32) {
        self.span_gen.generate(span, x, y, len);
        self.span_cnv.generate(span, x, y, len);
    }
}

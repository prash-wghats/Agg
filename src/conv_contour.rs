use crate::conv_adaptor_vcgen::ConvAdaptorVcgen;
use crate::math_stroke::{InnerJoin, LineJoin};
use crate::vcgen_contour::VcgenContour;
use crate::{VertexSource};

pub struct ConvContour<'a, VS: VertexSource> {
    base_type: ConvAdaptorVcgen<'a, VS, VcgenContour>,
}

impl<'a, VS: VertexSource> ConvContour<'a, VS> {
    pub fn new_owned(vs: VS) -> Self {
        Self {
            base_type: ConvAdaptorVcgen::new_owned(vs),
        }
    }

    pub fn new_borrowed(vs: &'a mut VS) -> Self {
        Self {
            base_type: ConvAdaptorVcgen::new_borrowed(vs),
        }
    }

    pub fn source_mut(&mut self) -> &mut VS {
        self.base_type.source_mut()
    }

    pub fn source(&self) -> &VS {
        self.base_type.source()
    }

    pub fn set_line_join(&mut self, lj: LineJoin) {
        self.base_type.generator_mut().set_line_join(lj);
    }
    pub fn set_inner_join(&mut self, ij: InnerJoin) {
        self.base_type.generator_mut().set_inner_join(ij);
    }
    pub fn set_width(&mut self, w: f64) {
        self.base_type.generator_mut().set_width(w);
    }
    pub fn set_miter_limit(&mut self, ml: f64) {
        self.base_type.generator_mut().set_miter_limit(ml);
    }
    pub fn set_miter_limit_theta(&mut self, t: f64) {
        self.base_type.generator_mut().set_miter_limit_theta(t);
    }
    pub fn set_inner_miter_limit(&mut self, ml: f64) {
        self.base_type.generator_mut().set_inner_miter_limit(ml);
    }
    pub fn set_approximation_scale(&mut self, a: f64) {
        self.base_type.generator_mut().set_approximation_scale(a);
    }
    pub fn set_auto_detect_orientation(&mut self, v: bool) {
        self.base_type
            .generator_mut()
            .set_auto_detect_orientation(v);
    }

    pub fn line_join(&self) -> LineJoin {
        self.base_type.generator().line_join()
    }
    pub fn inner_join(&self) -> InnerJoin {
        self.base_type.generator().inner_join()
    }
    pub fn width(&self) -> f64 {
        self.base_type.generator().width()
    }
    pub fn miter_limit(&self) -> f64 {
        self.base_type.generator().miter_limit()
    }
    pub fn inner_miter_limit(&self) -> f64 {
        self.base_type.generator().inner_miter_limit()
    }
    pub fn approximation_scale(&self) -> f64 {
        self.base_type.generator().approximation_scale()
    }
    pub fn auto_detect_orientation(&self) -> bool {
        self.base_type.generator().auto_detect_orientation()
    }
}

impl<'a, VS: VertexSource> VertexSource for ConvContour<'a, VS> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

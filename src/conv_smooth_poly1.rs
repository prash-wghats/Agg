use crate::conv_adaptor_vcgen::{ConvAdaptorVcgen, NullMarkers};
use crate::conv_curve::ConvCurve;
use crate::vcgen_smooth_poly1::VcgenSmoothPoly1;
use crate::VertexSource;

pub struct ConvSmoothPoly1<'a, VS: VertexSource> {
    pub base_type: ConvAdaptorVcgen<'a, VS, VcgenSmoothPoly1, NullMarkers>,
}

impl<'a, VS: VertexSource> ConvSmoothPoly1<'a, VS> {
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

    pub fn set_smooth_value(&mut self, v: f64) {
        self.base_type.generator_mut().set_smooth_value(v);
    }

    pub fn smooth_value(&self) -> f64 {
        self.base_type.generator().smooth_value()
    }
}

impl<'a, VS: VertexSource> VertexSource for ConvSmoothPoly1<'a, VS> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

pub struct ConvSmoothPoly1Curve<'a, VS: VertexSource> {
    base_type: ConvCurve<'a, ConvSmoothPoly1<'a, VS>>,
}

impl<'a, VS: VertexSource> ConvSmoothPoly1Curve<'a, VS> {
    pub fn new_borrowed(vs: &'a mut VS) -> Self {
        Self {
            base_type: ConvCurve::new_owned(ConvSmoothPoly1::new_borrowed(vs)),
        }
    }

    pub fn new_owned(vs: VS) -> Self {
        Self {
            base_type: ConvCurve::new_owned(ConvSmoothPoly1::new_owned(vs)),
        }
    }

    pub fn set_smooth_value(&mut self, v: f64) {
        self.base_type.source_mut().set_smooth_value(v);
    }

    pub fn smooth_value(&self) -> f64 {
        self.base_type.source().smooth_value()
    }

	pub fn set_approximation_scale(&mut self, s: f64) {
		self.base_type.set_approximation_scale(s);
	}

    /*pub fn base_curve_mut(&mut self) -> &'a mut ConvCurve<ConvSmoothPoly1<VS>> {
        &mut self.base_type
    }

    pub fn base_curve(&self) -> &'a ConvCurve<ConvSmoothPoly1<VS>> {
        &self.base_type
    }*/
	pub fn source(&self) -> &VS {
		self.base_type.source().source()
	}

	pub fn source_mut(&mut self) -> &mut VS {
		self.base_type.source_mut().source_mut()
	}
}

impl<'a, VS: VertexSource> VertexSource for ConvSmoothPoly1Curve<'a, VS> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

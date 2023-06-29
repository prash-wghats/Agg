use crate::conv_adaptor_vcgen::{ConvAdaptorVcgen, NullMarkers};
use crate::vcgen_bspline::VcgenBspline;
use crate::{Markers, VertexSource};

//---------------------------------------------------------conv_bspline
pub struct ConvBspline<'a, VS: VertexSource, Mrk: Markers = NullMarkers> {
    pub base_type: ConvAdaptorVcgen<'a, VS, VcgenBspline, Mrk>,
}

impl<'a, VS: VertexSource> ConvBspline<'a, VS> {
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

    pub fn set_interpolation_step(&mut self, v: f64) {
        self.base_type.generator_mut().set_interpolation_step(v);
    }

    pub fn interpolation_step(&self) -> f64 {
        self.base_type.generator().interpolation_step()
    }
}

impl<'a, VS: VertexSource> VertexSource for ConvBspline<'a, VS> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

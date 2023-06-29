use crate::conv_adaptor_vpgen::ConvAdaptorVpgen;
use crate::vpgen_segmentator::VpgenSegmentator;
use crate::VertexSource;

//========================================================conv_segmentator
pub struct ConvSegmentator<'a, VS: VertexSource> {
    pub base_type: ConvAdaptorVpgen<'a, VS, VpgenSegmentator>,
}

impl<'a, VS: VertexSource> ConvSegmentator<'a, VS> {
    pub fn new_owned(vs: VS) -> Self {
        Self {
            base_type: ConvAdaptorVpgen::new_owned(vs),
        }
    }

    pub fn new_borrowed(vs: &'a mut VS) -> Self {
        Self {
            base_type: ConvAdaptorVpgen::new_borrowed(vs),
        }
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.base_type.vpgen_mut().set_approximation_scale(s)
    }

    pub fn approximation_scale(&mut self) -> f64 {
        self.base_type.vpgen().approximation_scale()
    }
    pub fn source_mut(&mut self) -> &mut VS {
        self.base_type.source_mut()
    }
    pub fn source(&self) -> &VS {
        self.base_type.source()
    }
}

impl<'a, VS: VertexSource> VertexSource for ConvSegmentator<'a, VS> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

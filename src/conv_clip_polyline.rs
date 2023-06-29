use crate::conv_adaptor_vpgen::ConvAdaptorVpgen;
use crate::vpgen_clip_polyline::VpgenClipPolyline;

use crate::VertexSource;
pub struct ConvClipPolyline<'a, VS: VertexSource> {
    base_type: ConvAdaptorVpgen<'a, VS, VpgenClipPolyline>,
}

impl<'a, VS: VertexSource> ConvClipPolyline<'a, VS> {
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

    pub fn clip_box(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.base_type.vpgen_mut().clip_box(x1, y1, x2, y2);
    }

    pub fn x1(&self) -> f64 {
        self.base_type.vpgen().x1()
    }

    pub fn y1(&self) -> f64 {
        self.base_type.vpgen().y1()
    }

    pub fn x2(&self) -> f64 {
        self.base_type.vpgen().x2()
    }

    pub fn y2(&self) -> f64 {
        self.base_type.vpgen().y2()
    }
}

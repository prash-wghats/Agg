use crate::conv_adaptor_vcgen::*;
use crate::vcgen_vertex_sequence::*;
use crate::{Markers, VertexSource, VertexSourceWithMarker};

pub struct ConvMarkerAdaptor<'a, VS: VertexSource, M: Markers = NullMarkers> {
    base_type: ConvAdaptorVcgen<'a, VS, VcgenVertexSequence, M>,
}

impl<'a, VS: VertexSource, M: Markers> VertexSourceWithMarker for ConvMarkerAdaptor<'a, VS, M> {
    type Mrk = M;
    fn markers_mut(&mut self) -> &mut M {
        self.base_type.markers_mut()
    }
}

impl<'a, VS: VertexSource, M: Markers> ConvMarkerAdaptor<'a, VS, M> {
    pub fn new_owned(vs: VS) -> Self {
        ConvMarkerAdaptor {
            base_type: ConvAdaptorVcgen::new_owned(vs),
        }
    }

    pub fn new_borrowed(vs: &'a mut VS) -> Self {
        ConvMarkerAdaptor {
            base_type: ConvAdaptorVcgen::new_borrowed(vs),
        }
    }

    pub fn set_shorten(&mut self, s: f64) {
        self.base_type.generator_mut().set_shorten(s);
    }

    pub fn shorten(&self) -> f64 {
        self.base_type.generator().shorten()
    }
    pub fn source_mut(&mut self) -> &mut VS {
        self.base_type.source_mut()
    }

    pub fn source(&self) -> &VS {
        self.base_type.source()
    }

    pub fn generator(&self) -> &VcgenVertexSequence {
        self.base_type.generator()
    }

    pub fn generator_mut(&mut self) -> &mut VcgenVertexSequence {
        self.base_type.generator_mut()
    }

    pub fn markers(&self) -> &M {
        self.base_type.markers()
    }
}

impl<'a, VS: VertexSource, M: Markers> VertexSource for ConvMarkerAdaptor<'a, VS, M> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

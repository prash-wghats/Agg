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
//
// ConvDash
//
//----------------------------------------------------------------------------
use crate::conv_adaptor_vcgen::{ConvAdaptorVcgen, NullMarkers};
use crate::vcgen_dash::VcgenDash;
use crate::{Markers, VertexSource, VertexSourceWithMarker};

//----------------------------------------------------------conv_stroke
pub struct ConvDash<'a, VS: VertexSource, Mrk: Markers = NullMarkers> {
    pub base_type: ConvAdaptorVcgen<'a, VS, VcgenDash, Mrk>,
}

impl<'a, VS: VertexSource, Mrk: Markers> VertexSourceWithMarker for ConvDash<'a, VS, Mrk> {
    type Mrk = Mrk;
    fn markers_mut(&mut self) -> &mut Mrk {
        self.base_type.markers_mut()
    }
}

impl<'a, VS: VertexSource, Mrk: Markers> ConvDash<'a, VS, Mrk> {
    pub fn new_owned(vs: VS) -> Self {
        ConvDash {
            base_type: ConvAdaptorVcgen::new_owned(vs),
        }
    }

    pub fn new_borrowed(vs: &'a mut VS) -> Self {
        ConvDash {
            base_type: ConvAdaptorVcgen::new_borrowed(vs),
        }
    }

    pub fn source_mut(&mut self) -> &mut VS {
        self.base_type.source_mut()
    }

    pub fn source(&self) -> &VS {
        self.base_type.source()
    }

    pub fn generator(&self) -> &VcgenDash {
        self.base_type.generator()
    }

    pub fn generator_mut(&mut self) -> &mut VcgenDash {
        self.base_type.generator_mut()
    }

    pub fn markers(&self) -> &Mrk {
        self.base_type.markers()
    }

    pub fn remove_all_dashes(&mut self) {
        self.base_type.generator_mut().remove_all_dashes();
    }

    pub fn add_dash(&mut self, dash_len: f64, gap_len: f64) {
        self.base_type.generator_mut().add_dash(dash_len, gap_len);
    }

    pub fn dash_start(&mut self, ds: f64) {
        self.base_type.generator_mut().dash_start(ds);
    }

    pub fn set_shorten(&mut self, s: f64) {
        self.base_type.generator_mut().shorten(s);
    }

    pub fn shorten(&self) -> f64 {
        self.base_type.generator().get_shorten()
    }
}

impl<'a, VS: VertexSource, Mrk: Markers> VertexSource for ConvDash<'a, VS, Mrk> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

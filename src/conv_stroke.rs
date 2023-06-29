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
// ConvStroke
//
//----------------------------------------------------------------------------

use crate::conv_adaptor_vcgen::{ConvAdaptorVcgen, NullMarkers};
use crate::math_stroke::{InnerJoin, LineCap, LineJoin};
use crate::vcgen_stroke::VcgenStroke;
use crate::{Markers, VertexSource, VertexSourceWithMarker};

//----------------------------------------------------------ConvStroke
pub struct ConvStroke<'a, VS: VertexSource, Mrk: Markers = NullMarkers> {
    pub base_type: ConvAdaptorVcgen<'a, VS, VcgenStroke, Mrk>,
}

impl<'a, VS: VertexSource, Mrk: Markers> VertexSourceWithMarker for ConvStroke<'a, VS, Mrk> {
    type Mrk = Mrk;
    fn markers_mut(&mut self) -> &mut Mrk {
        self.base_type.markers_mut()
    }
}
//----------------------------------------------------------ConvStroke
impl<'a, VS: VertexSource, Mrk: Markers> ConvStroke<'a, VS, Mrk> {
    pub fn new_owned(vs: VS) -> Self {
        ConvStroke {
            base_type: ConvAdaptorVcgen::new_owned(vs),
        }
    }

    pub fn new_borrowed(vs: &'a mut VS) -> Self {
        ConvStroke {
            base_type: ConvAdaptorVcgen::new_borrowed(vs),
        }
    }

    pub fn set_source_owned(&mut self, source: VS) {
        self.base_type.set_source_owned(source)
    }

    pub fn set_source_borrowed(&mut self, source: &'a mut VS) {
        self.base_type.set_source_borrowed(source)
    }

    pub fn source_mut(&mut self) -> &mut VS {
        self.base_type.source_mut()
    }

    pub fn source(&self) -> &VS {
        self.base_type.source()
    }

    pub fn generator(&self) -> &VcgenStroke {
        self.base_type.generator()
    }

    pub fn generator_mut(&mut self) -> &mut VcgenStroke {
        self.base_type.generator_mut()
    }

    pub fn markers(&self) -> &Mrk {
        self.base_type.markers()
    }

    pub fn set_line_cap(&mut self, lc: LineCap) {
        self.base_type.generator_mut().set_line_cap(lc);
    }
    pub fn set_line_join(&mut self, lj: LineJoin) {
        self.base_type.generator_mut().set_line_join(lj);
    }
    pub fn set_inner_join(&mut self, ij: InnerJoin) {
        self.base_type.generator_mut().set_inner_join(ij);
    }
    pub fn line_cap(&self) -> LineCap {
        self.base_type.generator().line_cap()
    }
    pub fn line_join(&self) -> LineJoin {
        self.base_type.generator().line_join()
    }
    pub fn inner_join(&self) -> InnerJoin {
        self.base_type.generator().inner_join()
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
    pub fn set_shorten(&mut self, s: f64) {
        self.base_type.generator_mut().set_shorten(s);
    }
    pub fn shorten(&self) -> f64 {
        self.base_type.generator().shorten()
    }
}

impl<'a, VS: VertexSource, Mrk: Markers> VertexSource for ConvStroke<'a, VS, Mrk> {
    fn rewind(&mut self, path_id: u32) {
        self.base_type.rewind(path_id)
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        self.base_type.vertex(x, y)
    }
}

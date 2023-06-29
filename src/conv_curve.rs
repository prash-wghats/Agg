//---------------------------------------------------------------conv_curve
// Curve converter class. Any path storage can have Bezier curves defined
// by their control points. There're two types of curves supported: curve3
// and curve4. Curve3 is a conic Bezier curve with 2 endpoints and 1 control
// point. Curve4 has 2 control points (4 points in total) and can be used
// to interpolate more complicated curves. Curve4, unlike curve3 can be used
// to approximate arcs, both circular and elliptical. Curves are approximated
// with straight lines and one of the approaches is just to store the whole
// sequence of vertices that approximate our curve. It takes additional
// memory, and at the same time the consecutive vertices can be calculated
// on demand.
//
// Initially, path storages are not suppose to keep all the vertices of the
// curves (although, nothing prevents us from doing so). Instead, path_storage
// keeps only vertices, needed to calculate a curve on demand. Those vertices
// are marked with special commands. So, if the path_storage contains curves
// (which are not real curves yet), and we render this storage directly,
// all we will see is only 2 or 3 straight line segments (for curve3 and
// curve4 respectively). If we need to see real curves drawn we need to
// include this class into the conversion pipeline.
//
// Class conv_curve recognizes commands PathCmd::Curve3 and PathCmd::Curve4
// and converts these vertices into a move_to/line_to sequence.
//-----------------------------------------------------------------------

use crate::basics::{is_stop, PathCmd};
use crate::curves::{Curve3, Curve4, CurveApproximationMethod};
use crate::{CurveType3, CurveType4, Equiv, VertexSource};

pub struct ConvCurve<'a, VS: VertexSource, C3: CurveType3 = Curve3, C4: CurveType4 = Curve4> {
    source: Equiv<'a, VS>,
    last_x: f64,
    last_y: f64,
    curve3: C3,
    curve4: C4,
}

impl<'a, VS: VertexSource, C3: CurveType3, C4: CurveType4> ConvCurve<'a, VS, C3, C4> {
    pub fn new_owned(source: VS) -> Self {
        ConvCurve {
            source: Equiv::Own(source),
            last_x: 0.0,
            last_y: 0.0,
            curve3: C3::new(),
            curve4: C4::new(),
        }
    }

    pub fn new_borrowed(source: &'a mut VS) -> Self {
        ConvCurve {
            source: Equiv::Brw(source),
            last_x: 0.0,
            last_y: 0.0,
            curve3: C3::new(),
            curve4: C4::new(),
        }
    }

    pub fn set_source_owned(&mut self, source: VS) {
        self.source = Equiv::Own(source);
    }

    pub fn set_source_borrowed(&mut self, source: &'a mut VS) {
        self.source = Equiv::Brw(source);
    }

    pub fn source_mut(&mut self) -> &mut VS {
        &mut self.source
    }
    pub fn source(&self) -> &VS {
        &self.source
    }

    pub fn set_approximation_method(&mut self, v: CurveApproximationMethod) {
        self.curve3.set_approximation_method(v);
        self.curve4.set_approximation_method(v);
    }

    pub fn approximation_method_(&self) -> CurveApproximationMethod {
        self.curve4.approximation_method()
    }

    pub fn set_approximation_scale(&mut self, s: f64) {
        self.curve3.set_approximation_scale(s);
        self.curve4.set_approximation_scale(s);
    }

    pub fn approximation_scale_(&self) -> f64 {
        self.curve4.approximation_scale()
    }

    pub fn set_angle_tolerance(&mut self, v: f64) {
        self.curve3.set_angle_tolerance(v);
        self.curve4.set_angle_tolerance(v);
    }

    pub fn angle_tolerance_(&self) -> f64 {
        self.curve4.angle_tolerance()
    }

    pub fn set_cusp_limit(&mut self, v: f64) {
        self.curve3.set_cusp_limit(v);
        self.curve4.set_cusp_limit(v);
    }

    pub fn cusp_limit_(&self) -> f64 {
        self.curve4.cusp_limit()
    }
}

impl<'a, VS: VertexSource, C3: CurveType3, C4: CurveType4> VertexSource
    for ConvCurve<'a, VS, C3, C4>
{
    fn rewind(&mut self, path_id: u32) {
        self.source.rewind(path_id);
        self.last_x = 0.0;
        self.last_y = 0.0;
        self.curve3.reset();
        self.curve4.reset();
    }

    //------------------------------------------------------------------------
    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        if !is_stop(self.curve3.vertex(x, y)) {
            self.last_x = *x;
            self.last_y = *y;
            return PathCmd::LineTo as u32;
        }

        if !is_stop(self.curve4.vertex(x, y)) {
            self.last_x = *x;
            self.last_y = *y;
            return PathCmd::LineTo as u32;
        }

        let mut ct2_x: f64 = 0.;
        let mut ct2_y: f64 = 0.;
        let mut end_x: f64 = 0.;
        let mut end_y: f64 = 0.;

        let mut cmd = self.source.vertex(x, y);
        match cmd {
            mcmd if mcmd == PathCmd::Curve3 as u32 => {
                self.source.vertex(&mut end_x, &mut end_y);

                self.curve3
                    .init(self.last_x, self.last_y, *x, *y, end_x, end_y);

                self.curve3.vertex(x, y); // First call returns PathCmd::MoveTo
                self.curve3.vertex(x, y); // This is the first vertex of the curve
                cmd = PathCmd::LineTo as u32;
            }
            mcmd if mcmd == PathCmd::Curve4 as u32 => {
                self.source.vertex(&mut ct2_x, &mut ct2_y);
                self.source.vertex(&mut end_x, &mut end_y);

                self.curve4
                    .init(self.last_x, self.last_y, *x, *y, ct2_x, ct2_y, end_x, end_y);

                self.curve4.vertex(x, y); // First call returns PathCmd::MoveTo
                self.curve4.vertex(x, y); // This is the first vertex of the curve
                cmd = PathCmd::LineTo as u32;
            }
            _ => {}
        }

        self.last_x = *x;
        self.last_y = *y;
        cmd
    }
}

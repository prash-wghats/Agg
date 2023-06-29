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

use crate::basics::{is_end_poly, is_move_to, is_stop, is_vertex, PathCmd};
use crate::{Generator, Markers, VertexSource, Equiv};

pub struct NullMarkers;
impl Markers for NullMarkers {}
impl Generator for NullMarkers {
    fn new() -> Self {
        Self {}
    }

    fn remove_all(&mut self) {}

    fn add_vertex(&mut self, _x: f64, _y: f64, _cmd: u32) {}

    //fn prepare_src(&mut self) {}
}

impl VertexSource for NullMarkers {
    fn rewind(&mut self, _path_id: u32) {}

    fn vertex(&mut self, _x: &mut f64, _y: &mut f64) -> u32 {
        return PathCmd::Stop as u32;
    }
}

enum Status {
    Initial,
    Accumulate,
    Generate,
}

//------------------------------------------------------ConvAdaptorVcgen
pub struct ConvAdaptorVcgen<'a,VS: VertexSource, Gen: Generator, Mrk: Markers = NullMarkers> {
    source: Equiv<'a, VS>,
    generator: Gen,
    markers: Mrk,
    status: Status,
    last_cmd: u32,
    start_x: f64,
    start_y: f64,
}

impl<'a,VS: VertexSource, Gen: Generator, Mrk: Markers> ConvAdaptorVcgen<'a,VS, Gen, Mrk> {
    pub fn new_owned(source: VS) -> Self {
        ConvAdaptorVcgen {
            source: Equiv::Own(source),
            generator: <Gen as Generator>::new(),
            markers: Mrk::new(),
            status: Status::Initial,
            last_cmd: 0,
            start_x: 0.0,
            start_y: 0.0,
        }
    }

	pub fn new_borrowed(source: &'a mut VS) -> Self {
        ConvAdaptorVcgen {
            source: Equiv::Brw(source),
            generator: <Gen as Generator>::new(),
            markers: Mrk::new(),
            status: Status::Initial,
            last_cmd: 0,
            start_x: 0.0,
            start_y: 0.0,
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

    pub fn generator(&self) -> &Gen {
        &self.generator
    }

    pub fn generator_mut(&mut self) -> &mut Gen {
        &mut self.generator
    }

    pub fn markers_mut(&mut self) -> &mut Mrk {
        &mut self.markers
    }

    pub fn markers(&self) -> &Mrk {
        &self.markers
    }
}
impl<'a,VS: VertexSource, Gen: Generator, Mrk: Markers> VertexSource
    for ConvAdaptorVcgen<'a,VS, Gen, Mrk>
{
    fn rewind(&mut self, path_id: u32) {
        self.source.rewind(path_id);
        self.status = Status::Initial;
    }

    fn vertex(&mut self, x: &mut f64, y: &mut f64) -> u32 {
        let mut cmd = PathCmd::Stop as u32;
        let mut done = false;
        while !done {
            match self.status {
                Status::Initial => {
                    self.markers.remove_all();
                    self.last_cmd = self
                        .source
                        .vertex(&mut self.start_x, &mut self.start_y);
                    self.status = Status::Accumulate;
                }
                Status::Accumulate => {
                    if is_stop(self.last_cmd) {
                        return PathCmd::Stop as u32;
                    }
                    self.generator.remove_all();
                    self.generator.add_vertex(
                        self.start_x,
                        self.start_y,
                        PathCmd::MoveTo as u32,
                    );
                    self.markers.add_vertex(
                        self.start_x,
                        self.start_y,
                        PathCmd::MoveTo as u32,
                    );
                    loop {
                        cmd = self.source.vertex(x, y);
                        if is_vertex(cmd) {
                            self.last_cmd = cmd;
                            if is_move_to(cmd) {
                                self.start_x = *x;
                                self.start_y = *y;
                                break;
                            }
                            self.generator.add_vertex(*x, *y, cmd);
                            self.markers.add_vertex(*x, *y, PathCmd::LineTo as u32);
                        } else {
                            if is_stop(cmd) {
                                self.last_cmd = PathCmd::Stop as u32;
                                break;
                            }
                            if is_end_poly(cmd) {
                                self.generator.add_vertex(*x, *y, cmd);
                                break;
                            }
                        }
                    }
                    self.generator.rewind(0);
                    self.status = Status::Generate;
                }
                Status::Generate => {
                    cmd = self.generator.vertex(x, y);
                    if is_stop(cmd) {
                        self.status = Status::Accumulate;
                    } else {
                        done = true;
                    }
                }
            }
        }
        cmd
    }
}

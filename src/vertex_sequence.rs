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
// VecSequence container and VertexDist struct
//
//----------------------------------------------------------------------------

//----------------------------------------------------------VecSequence
// Modified agg::pod_bvector. The data is interpreted as a sequence
// of vertices. It means that the type T must expose:
//
// bool T::operator() (const T& val)
//
// that is called every time new vertex is being added. The main purpose
// of this operator is the possibility to calculate some values during
// adding and to return true if the vertex fits some criteria or false if
// it doesn't. In the last case the new vertex is not added.
//
// The simple example is filtering coinciding vertices with calculation
// of the distance between the current and previous ones:
//
//    struct VertexDist
//    {
//        double   x;
//        double   y;
//        double   dist;
//
//        VertexDist() {}
//        VertexDist(double x_, double y_) :
//            x(x_),
//            y(y_),
//            dist(0.0)
//        {
//        }
//
//        bool operator () (const VertexDist& val)
//        {
//            return (dist = calc_distance(x, y, val.x, val.y)) > EPSILON;
//        }
//    };
//
// Function close() calls this operator and removes the last vertex if
// necessary.
//------------------------------------------------------------------------

pub use crate::array::PodBVector;
use crate::math::{calc_distance, VERTEX_DIST_EPSILON};
use crate::{VertexDistance, VertexSequence};

pub type VecSequence<T> = Vec<T>;

impl<T: VertexDistance + Copy + Clone> VertexSequence for VecSequence<T> {
    type ValueType = T;

    fn size(&self) -> usize {
        self.len()
    }

    fn remove_all(&mut self) {
        self.clear();
    }

    fn remove_last(&mut self) {
        self.remove(self.len() - 1);
    }

    fn get(&self, i: usize) -> &T {
        &self[i]
    }

    fn get_mut(&mut self, i: usize) -> &mut T {
        &mut self[i]
    }

    fn get_mut_slice(&mut self, s: usize, e: usize) -> &mut [T] {
        &mut self[s..=e]
    }

    fn close(&mut self, closed: bool) {
        while self.len() > 1 {
            let len = self.len();
            let v = self[len - 1];
            if self[len - 2].calc_distance(&v) {
                break;
            }
            let t = self.pop().unwrap();
            self.modify_last(t);
        }

        if closed {
            while self.len() > 1 {
                let len = self.len();
                let v = self[0];
                if self[len - 1].calc_distance(&v) {
                    break;
                }
                self.pop();
            }
        }
    }

    fn add(&mut self, val: T) {
        let len = self.len();
        if len > 1 {
            let v = self[len - 1];
            if !self[len - 2].calc_distance(&v) {
                self.pop();
            }
        }
        self.push(val);
    }
}

//-------------------------------------------------------------VertexDist
// VertexDistance (x, y) with the distance to the next one. The last vertex has
// distance between the last and the first points if the polygon is closed
// and 0.0 if it's a polyline.
#[derive(Clone, Copy, PartialEq)]
pub struct VertexDist {
    pub x: f64,
    pub y: f64,
    pub dist: f64,
    pub cmd: u32,
}

impl VertexDist {
    pub fn new(x: f64, y: f64) -> VertexDist {
        VertexDist {
            x: x,
            y: y,
            dist: 0.0,
            cmd: 0,
        }
    }
    pub fn new_with_cmd(x: f64, y: f64, cmd: u32) -> VertexDist {
        VertexDist {
            x: x,
            y: y,
            dist: 0.0,
            cmd: cmd,
        }
    }
}

impl VertexDistance for VertexDist {
    fn calc_distance(&mut self, val: &VertexDist) -> bool {
        let dist = calc_distance(self.x, self.y, val.x, val.y);
        let ret = dist > VERTEX_DIST_EPSILON;
        if !ret {
            self.dist = 1.0 / VERTEX_DIST_EPSILON;
        } else {
            self.dist = dist;
        }
        ret
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct VertexDistCmd {
    pub x: f64,
    pub y: f64,
    pub dist: f64,
    pub cmd: u32,
}

impl VertexDistCmd {
    pub fn new(x: f64, y: f64, cmd: u32) -> VertexDistCmd {
        VertexDistCmd {
            x: x,
            y: y,
            dist: 0.0,
            cmd: cmd,
        }
    }
}

impl VertexDistance for VertexDistCmd {
    fn calc_distance(&mut self, val: &VertexDistCmd) -> bool {
        let dist = calc_distance(self.x, self.y, val.x, val.y);
        let ret = dist > VERTEX_DIST_EPSILON;
        if !ret {
            self.dist = 1.0 / VERTEX_DIST_EPSILON;
        } else {
            self.dist = dist;
        }
        ret
    }
}

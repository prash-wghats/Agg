
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
// Contact: mcseem@ antigrain.com
//          mcseemagg@yahoo.com
//          http://www.antigrain.com
//----------------------------------------------------------------------------

use crate::{VertexConsumer, Color, ColorFn};

pub trait PodBVector<T> {
    fn prev(&self, i: usize) -> &T;
    fn curr(&self, i: usize) -> &T;
    fn next(&self, i: usize) -> &T;
    fn modify_last(&mut self, v: T);
    fn last(&self) -> &T;
    //fn size(&self) -> usize;
}

////
pub type PodArray<T> = Vec<T>;
impl<C: Color> ColorFn<C> for  PodArray<C> {
	fn size(&self) -> u32 {
		self.len() as u32
	}
    fn get(&mut self, v: u32) -> C {
		self[v as usize]
	}
}
////
pub type VecPodB<T> = Vec<T>;

/////
impl<T> PodBVector<T> for Vec<T> {
    fn prev(&self, i: usize) -> &T {
        &self[(self.len() + i - 1) % self.len()]
    }

    fn next(&self, i: usize) -> &T {
        &self[(i + 1) % self.len()]
    }

    fn curr(&self, i: usize) -> &T {
        &self[i]
    }

    fn modify_last(&mut self, v: T) {
        self.pop();
        self.push(v);
    }

    fn last(&self) -> &T {
        &self[self.len() - 1]
    }
    /*fn size(&self) -> usize {
        self.len()
    }*/
}

////
impl<T> VertexConsumer for Vec<T> {
    type ValueType = T;
    fn remove_all(&mut self) {
        self.clear()
    }
    fn add(&mut self, val: Self::ValueType) {
        self.push(val)
    }
}

////
pub struct PodAutoArray<T: Copy, const SIZE: usize> {
    m_array: [T; SIZE],
    m_size: usize,
}

impl<T: Copy, const SIZE: usize> PodAutoArray<T, SIZE> {
    pub fn new(t: T) -> Self {
        PodAutoArray {
            m_array: [t; SIZE],
            m_size: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.m_size
    }

    pub fn at(&self, i: usize) -> &T {
        &self.m_array[i]
    }

    pub fn add(&mut self, v: T) {
        self.m_array[self.m_size] = v;
        self.m_size += 1;
    }

    pub fn inc_size(&mut self, size: usize) {
        self.m_size += size;
    }

    pub fn value_at(&self, i: usize) -> T {
        self.m_array[i]
    }
}

impl<T: Copy, const SIZE: usize> std::ops::Index<usize> for PodAutoArray<T, SIZE> {
    type Output = T;

    fn index(&self, i: usize) -> &T {
        &self.m_array[i]
    }
}

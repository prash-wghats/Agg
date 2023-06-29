use crate::{basics::uround, Color, ColorInterpolator, ColorFn};

pub struct ColorIp<T: Color> {
    c1: T,
    c2: T,
    len: u32,
    count: u32,
}

impl<T: Color> ColorIp<T> {}

impl<T: Color> ColorInterpolator for ColorIp<T> {
    type C = T;
    fn new(c1: T, c2: T, len: u32) -> Self {
        Self {
            c1,
            c2,
            len,
            count: 0,
        }
    }

    fn next(&mut self) {
        self.count += 1;
    }

    fn color(&self) -> T {
        self.c1
            .gradient(&self.c2, f64::from(self.count) / f64::from(self.len))
    }
}

struct ColorPoint<T: Color> {
    offset: f64,
    color: T,
}

impl<T: Color> ColorPoint<T> {
    fn new(off: f64, c: T) -> Self {
        let mut off = off;
        if off < 0. {
            off = 0.
        }
        if off > 1. {
            off = 1.
        }
        Self {
            offset: off,
            color: c,
        }
    }
}

// Build Gradient Lut
// First, call remove_all(), then add_color() at least twice,
// then build_lut(). Argument "offset" in add_color must be
// in range [0...1] and defines a color stop as it is described
// in SVG specification, section Gradients and Patterns.
// The simplest linear gradient is:
//    gradient_lut.add_color(0.0, start_color);
//    gradient_lut.add_color(1.0, end_color);

pub struct GradientLut<T: ColorInterpolator, const COLOR_LUT_SIZE: usize> {
    color_lut: [T::C; COLOR_LUT_SIZE],
    color_profile: Vec<ColorPoint<T::C>>,
}

impl<T: ColorInterpolator, const COLOR_LUT_SIZE: usize> GradientLut<T, COLOR_LUT_SIZE> {
    pub fn new() -> Self {
        GradientLut {
            color_lut: [T::C::new(); COLOR_LUT_SIZE],
            color_profile: Vec::new(),
        }
    }

    pub fn remove_all(&mut self) {
        self.color_profile.clear();
    }

    pub fn add_color(&mut self, offset: f64, color: T::C) {
        self.color_profile.push(ColorPoint::new(offset, color));
    }

    pub fn build_lut(&mut self) {
        self.color_profile
            .sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap());
        self.color_profile.dedup_by(|a, b| a.offset == b.offset);

        if self.color_profile.len() >= 2 {
            let mut start = uround(self.color_profile[0].offset * COLOR_LUT_SIZE as f64) as usize;
            let mut end: usize = 0;
            let mut c = self.color_profile[0].color;

            for i in 0..start {
                self.color_lut[i] = c;
            }

            for i in 1..self.color_profile.len() {
                end = uround(self.color_profile[i].offset * COLOR_LUT_SIZE as f64) as usize;
                let mut ci = T::new(
                    self.color_profile[i - 1].color,
                    self.color_profile[i].color,
                    (end - start + 1) as u32,
                );

                while start < end {
                    self.color_lut[start] = ci.color();
                    ci.next();
                    start += 1;
                }
            }

            c = self.color_profile.last().unwrap().color;
            for i in end..COLOR_LUT_SIZE {
                self.color_lut[i] = c;
            }
        }
    }
}

impl<T: ColorInterpolator, const COLOR_LUT_SIZE: usize> ColorFn<T::C> for GradientLut<T, COLOR_LUT_SIZE> {
	fn size(&self) -> u32 {
		COLOR_LUT_SIZE as u32
	}
    fn get(&mut self, v: u32) -> T::C {
		self.color_lut[v as usize]
	}
}
use crate::basics::iround;
use crate::FilterF;
use std::f64::consts::PI;

#[repr(i32)]
pub enum ImageFilterScale {
    Shift = 14,                      //----image_filter_shift
    Scale = 1 << Self::Shift as i32, //----ImageFilterScale::Scale as i32
    Mask = Self::Scale as i32 - 1,   //----image_filter_mask
}

#[repr(i32)]
pub enum ImageSubpixelScale {
    Shift = 8,                       //----ImageSubpixelScale::Shift as i32
    Scale = 1 << Self::Shift as i32, //----ImageSubpixelScale::Scale as i32
    Mask = Self::Scale as i32 - 1,   //----image_subpixel_mask
}

//-----------------------------------------------------ImageFilterLut
pub struct ImageFilterLut {
    radius: f64,
    diameter: u32,
    start: i32,
    weight_array: Vec<i16>,
}

impl ImageFilterLut {
    pub fn new() -> ImageFilterLut {
        ImageFilterLut {
            radius: 0.0,
            diameter: 0,
            start: 0,
            weight_array: Vec::new(),
        }
    }

    pub fn new_filter<F: FilterF>(f: &F, normalization: bool) -> ImageFilterLut {
        Self::new_filter_dyn(f as &dyn FilterF, normalization)
    }

    pub fn new_filter_dyn(f: &dyn FilterF, normalization: bool) -> ImageFilterLut {
        let mut m = ImageFilterLut {
            radius: 0.0,
            diameter: 0,
            start: 0,
            weight_array: Vec::new(),
        };
        m.calculate(f, normalization);
        m
    }

    pub fn calculate(&mut self, filter: &dyn FilterF, normalization: bool) {
        let r = filter.radius();
        self.realloc_lut(r);

        let pivot = self.diameter() << (ImageSubpixelScale::Shift as i32 - 1);
        for i in 0..pivot {
            let x = (i as f64) / (ImageSubpixelScale::Scale as i32 as f64);
            let y = filter.calc_weight(x);
            self.weight_array[(pivot + i) as usize] =
                iround(y * ImageFilterScale::Scale as i32 as f64) as i16;
            self.weight_array[(pivot - i) as usize] =
                iround(y * ImageFilterScale::Scale as i32 as f64) as i16;
        }
        let end = (self.diameter() << ImageSubpixelScale::Shift as i32) - 1;
        self.weight_array[0] = self.weight_array[end as usize];
        if normalization {
            self.normalize();
        }
    }

    pub fn radius(&self) -> f64 {
        self.radius
    }

    pub fn diameter(&self) -> u32 {
        self.diameter
    }

    pub fn start(&self) -> i32 {
        self.start
    }

    pub fn weight_array(&self) -> &[i16] {
        &self.weight_array
    }

    pub fn realloc_lut(&mut self, radius: f64) {
        self.radius = radius;
        self.diameter = (radius.ceil() * 2.0) as u32;
        self.start = -((self.diameter / 2 - 1) as i32);
        let size = self.diameter << ImageSubpixelScale::Shift as i32;
        if size > self.weight_array.len() as u32 {
            self.weight_array.resize((size) as usize, 0);
        }
    }

    // This function normalizes integer values and corrects the rounding
    // errors. It doesn't do anything with the source floating point values
    // (m_weight_array_dbl), it corrects only integers according to the rule
    // of 1.0 which means that any sum of pixel weights must be equal to 1.0.
    // So, the filter function must produce a graph of the proper shape.
    pub fn normalize(&mut self) {
        let mut flip = 1;

        for i in 0..ImageSubpixelScale::Scale as u32 {
            loop {
                let mut sum: i32 = 0;

                for j in 0..self.diameter {
                    sum += (self.weight_array[(j * ImageSubpixelScale::Scale as u32 + i) as usize])
                        as i32;
                }

                if sum == ImageFilterScale::Scale as i32 {
                    break;
                }

                let k = (ImageFilterScale::Scale as i32 as f64) / (sum as f64);
                sum = 0;
                for j in 0..self.diameter {
                    let tmp = iround(
                        self.weight_array[(j * ImageSubpixelScale::Scale as u32 + i) as usize]
                            as f64
                            * k,
                    );
                    sum += tmp;
                    self.weight_array[(j * ImageSubpixelScale::Scale as u32 + i) as usize] =
                        tmp as i16;
                }

                sum -= ImageFilterScale::Scale as i32;
                let inc: i32 = if sum > 0 { -1 } else { 1 };

                for j in 0..self.diameter {
                    if sum == 0 {
                        break;
                    }
                    flip ^= 1;
                    let idx = if flip != 0 {
                        self.diameter / 2 + j / 2
                    } else {
                        self.diameter / 2 - j / 2
                    };
                    let v =
                        self.weight_array[(idx * ImageSubpixelScale::Scale as u32 + i) as usize];
                    if v < ImageFilterScale::Scale as i32 as i16 {
                        self.weight_array[(idx * ImageSubpixelScale::Scale as u32 + i) as usize] +=
                            inc as i16;
                        sum += inc;
                    }
                }
            }
        }

        let pivot = self.diameter << (ImageSubpixelScale::Shift as i32 - 1);

        for i in 0..pivot {
            self.weight_array[(pivot + i) as usize] = self.weight_array[(pivot - i) as usize];
        }
        let end = (self.diameter() << ImageSubpixelScale::Shift as i32) - 1;
        self.weight_array[0] = self.weight_array[end as usize];
    }
}

//--------------------------------------------------------ImageFilter
pub struct ImageFilter<F: FilterF> {
    pub filter_function: F,
    pub filter_function_lut: ImageFilterLut,
}

impl<F: FilterF> ImageFilter<F> {
    pub fn new(filter: F) -> Self {
        let mut filter = Self {
            filter_function: filter,
            filter_function_lut: ImageFilterLut::new(),
        };
        filter
            .filter_function_lut
            .calculate(&filter.filter_function, true);
        filter
    }
}

//-----------------------------------------------ImageFilterBilinear
pub struct ImageFilterBilinear;

impl ImageFilterBilinear {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterBilinear {
    fn radius(&self) -> f64 {
        1.0
    }
    fn calc_weight(&self, x: f64) -> f64 {
        1.0 - x
    }
}

//-----------------------------------------------ImageFilterHanning
pub struct ImageFilterHanning;

impl ImageFilterHanning {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterHanning {
    fn radius(&self) -> f64 {
        1.0
    }
    fn calc_weight(&self, x: f64) -> f64 {
        0.5 + 0.5 * (PI * x).cos()
    }
}

//-----------------------------------------------ImageFilterHamming
pub struct ImageFilterHamming;

impl ImageFilterHamming {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterHamming {
    fn radius(&self) -> f64 {
        1.0
    }
    fn calc_weight(&self, x: f64) -> f64 {
        0.54 + 0.46 * (PI * x).cos()
    }
}

//-----------------------------------------------ImageFilterHermite
pub struct ImageFilterHermite;

impl ImageFilterHermite {
    pub fn new() -> Self {
        Self {}
    }
}
impl FilterF for ImageFilterHermite {
    fn radius(&self) -> f64 {
        1.0
    }
    fn calc_weight(&self, x: f64) -> f64 {
        (2.0 * x - 3.0) * x * x + 1.0
    }
}

//------------------------------------------------ImageFilterQuadric
pub struct ImageFilterQuadric;

impl ImageFilterQuadric {
    pub fn new() -> Self {
        Self {}
    }
}
impl FilterF for ImageFilterQuadric {
    fn radius(&self) -> f64 {
        1.5
    }
    fn calc_weight(&self, x: f64) -> f64 {
        if x < 0.5 {
            0.75 - x * x
        } else if x < 1.5 {
            let t = x - 1.5;
            0.5 * t * t
        } else {
            0.0
        }
    }
}

//------------------------------------------------ImageFilterBicubic
pub struct ImageFilterBicubic;

impl ImageFilterBicubic {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterBicubic {
    fn radius(&self) -> f64 {
        2.0
    }
    fn calc_weight(&self, x: f64) -> f64 {
        fn pow3(x: f64) -> f64 {
            if x <= 0.0 {
                0.0
            } else {
                x * x * x
            }
        }
        (1.0 / 6.0) * (pow3(x + 2.) - 4. * pow3(x + 1.) + 6. * pow3(x) - 4. * pow3(x - 1.))
    }
}

//-------------------------------------------------ImageFilterKaiser
pub struct ImageFilterKaiser {
    a: f64,
    i0a: f64,
}

impl ImageFilterKaiser {
    pub fn new_parms(b: f64) -> ImageFilterKaiser {
        ImageFilterKaiser {
            a: b,

            i0a: 1.0 / Self::bessel_i0(b),
        }
    }
    pub fn new() -> Self {
        Self::new_parms(6.33)
    }
    const EPSILON: f64 = 1e-12;
    fn bessel_i0(x: f64) -> f64 {
        let mut i = 2;
        let mut sum = 1.;
        let y = x * x / 4.;
        let mut t = y;

        while t > Self::EPSILON {
            sum += t;
            t *= y / (i * i) as f64;
            i += 1;
        }
        sum
    }
}

impl FilterF for ImageFilterKaiser {
    fn radius(&self) -> f64 {
        1.0
    }

    fn calc_weight(&self, x: f64) -> f64 {
        Self::bessel_i0(self.a * (1. - x * x).sqrt()) * self.i0a
    }
}

//----------------------------------------------ImageFilterCatrom
pub struct ImageFilterCatrom;

impl ImageFilterCatrom {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterCatrom {
    fn radius(&self) -> f64 {
        2.0
    }

    fn calc_weight(&self, x: f64) -> f64 {
        if x < 1.0 {
            0.5 * (2.0 + x * x * (-5.0 + x * 3.0))
        } else if x < 2.0 {
            0.5 * (4.0 + x * (-8.0 + x * (5.0 - x)))
        } else {
            0.
        }
    }
}

//---------------------------------------------ImageFilterMitchell
pub struct ImageFilterMitchell {
    p0: f64,
    p2: f64,
    p3: f64,
    q0: f64,
    q1: f64,
    q2: f64,
    q3: f64,
}

impl ImageFilterMitchell {
    pub fn new_parms(b: f64, c: f64) -> ImageFilterMitchell {
        ImageFilterMitchell {
            p0: (6.0 - 2.0 * b) / 6.0,
            p2: (-18.0 + 12.0 * b + 6.0 * c) / 6.0,
            p3: (12.0 - 9.0 * b - 6.0 * c) / 6.0,
            q0: (8.0 * b + 24.0 * c) / 6.0,
            q1: (-12.0 * b - 48.0 * c) / 6.0,
            q2: (6.0 * b + 30.0 * c) / 6.0,
            q3: (-b - 6.0 * c) / 6.0,
        }
    }
    pub fn new() -> Self {
        Self::new_parms(1.0 / 3.0, 1.0 / 3.0)
    }
}

impl FilterF for ImageFilterMitchell {
    fn radius(&self) -> f64 {
        2.0
    }

    fn calc_weight(&self, x: f64) -> f64 {
        if x < 1.0 {
            self.p0 + x * x * (self.p2 + x * self.p3)
        } else if x < 2.0 {
            self.q0 + x * (self.q1 + x * (self.q2 + x * self.q3))
        } else {
            0.0
        }
    }
}

//----------------------------------------------ImageFilterSpline16
pub struct ImageFilterSpline16;

impl ImageFilterSpline16 {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterSpline16 {
    fn radius(&self) -> f64 {
        2.0
    }

    fn calc_weight(&self, x: f64) -> f64 {
        if x < 1.0 {
            ((x - 9.0 / 5.0) * x - 1.0 / 5.0) * x + 1.0
        } else {
            ((-1.0 / 3.0 * (x - 1.) + 4.0 / 5.0) * (x - 1.) - 7.0 / 15.0) * (x - 1.)
        }
    }
}

//---------------------------------------------ImageFilterSpline36
pub struct ImageFilterSpline36;

impl ImageFilterSpline36 {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterSpline36 {
    fn radius(&self) -> f64 {
        3.0
    }

    fn calc_weight(&self, x: f64) -> f64 {
        if x < 1.0 {
            ((13.0 / 11.0 * x - 453.0 / 209.0) * x - 3.0 / 209.0) * x + 1.0
        } else if x < 2.0 {
            ((-6.0 / 11.0 * (x - 1.) + 270.0 / 209.0) * (x - 1.) - 156.0 / 209.0) * (x - 1.)
        } else {
            ((1.0 / 11.0 * (x - 2.) - 45.0 / 209.0) * (x - 2.) + 26.0 / 209.0) * (x - 2.)
        }
    }
}

//----------------------------------------------ImageFilterGaussian
pub struct ImageFilterGaussian;

impl ImageFilterGaussian {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterGaussian {
    fn radius(&self) -> f64 {
        2.0
    }

    fn calc_weight(&self, x: f64) -> f64 {
        (-2.0 * x * x).exp() * (2.0 / std::f64::consts::PI).sqrt()
    }
}

//------------------------------------------------ImageFilterBessel
pub struct ImageFilterBessel;

impl ImageFilterBessel {
    pub fn new() -> Self {
        Self {}
    }
}

impl FilterF for ImageFilterBessel {
    fn radius(&self) -> f64 {
        3.2383
    }

    fn calc_weight(&self, x: f64) -> f64 {
        if x == 0.0 {
            std::f64::consts::PI / 4.0
        } else {
            crate::math::besj(std::f64::consts::PI * x, 1) / (2.0 * x)
        }
    }
}

//-------------------------------------------------ImageFilterSinc
pub struct ImageFilterSinc {
    radius: f64,
}

impl ImageFilterSinc {
    pub fn new_parms(r: f64) -> ImageFilterSinc {
        ImageFilterSinc {
            radius: if r < 2.0 { 2.0 } else { r },
        }
    }
    pub fn new() -> Self {
        Self::new_parms(4.0)
    }
}

impl FilterF for ImageFilterSinc {
    fn radius(&self) -> f64 {
        self.radius
    }

    fn set_radius(&mut self, r: f64) {
        self.radius = if r < 2.0 { 2.0 } else { r };
    }

    fn calc_weight(&self, x: f64) -> f64 {
        let mut x = x;
        if x == 0.0 {
            1.0
        } else {
            x = PI * x;
            x.sin() / x
        }
    }
}

//-----------------------------------------------ImageFilterLanczos
pub struct ImageFilterLanczos {
    pub radius: f64,
}

impl ImageFilterLanczos {
    pub fn new_parms(r: f64) -> ImageFilterLanczos {
        ImageFilterLanczos {
            radius: if r < 2.0 { 2.0 } else { r },
        }
    }
    pub fn new() -> Self {
        Self::new_parms(4.0)
    }
}

impl FilterF for ImageFilterLanczos {
    fn radius(&self) -> f64 {
        self.radius
    }

    fn set_radius(&mut self, r: f64) {
        self.radius = if r < 2.0 { 2.0 } else { r };
    }

    fn calc_weight(&self, x: f64) -> f64 {
        let mut x = x;
        if x == 0.0 {
            return 1.0;
        }
        if x > self.radius {
            return 0.0;
        }
        x *= PI;
        let xr = x / self.radius;
        (x.sin() / x) * (xr.sin() / xr)
    }
}

//----------------------------------------------ImageFilterBlackman
pub struct ImageFilterBlackman {
    pub radius: f64,
}

impl ImageFilterBlackman {
    pub fn new_parms(r: f64) -> ImageFilterBlackman {
        ImageFilterBlackman {
            radius: if r < 2.0 { 2.0 } else { r },
        }
    }

    pub fn new() -> Self {
        Self::new_parms(4.0)
    }
}

impl FilterF for ImageFilterBlackman {
    fn radius(&self) -> f64 {
        self.radius
    }

    fn set_radius(&mut self, r: f64) {
        self.radius = if r < 2.0 { 2.0 } else { r };
    }

    fn calc_weight(&self, x: f64) -> f64 {
        let mut x = x;
        if x == 0.0 {
            return 1.0;
        }
        if x > self.radius {
            return 0.0;
        }
        x *= PI;
        let xr = x / self.radius;
        (x.sin() / x) * (0.42 + 0.5 * xr.cos() + 0.08 * (2.0 * xr).cos())
    }
}

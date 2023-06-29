//----------------------------------------------------------------bspline
// A very simple class of Bi-cubic Spline interpolation.
// First call init(num, x[], y[]) where num - number of source points,
// x, y - arrays of X and Y values respectively. Here Y must be a function
// of X. It means that all the X-coordinates must be arranged in the ascending
// order.
// Then call get(x) that calculates a value Y for the respective X.
// The class supports extrapolation, i.e. you can call get(x) where x is
// outside the given with init() X-range. Extrapolation is a simple linear
// function.
//
//------------------------------------------------------------------------
use crate::array::PodArray;

#[derive(Debug, Clone)]
pub struct Bspline {
    max: usize,
    num: usize,
    x: usize,
    y: usize,
    am: PodArray<f64>,
    last_idx: isize,
}

impl Bspline {
    pub fn new() -> Self {
        Bspline {
            max: 0,
            num: 0,
            x: 0,
            y: 0,
            am: PodArray::new(),
            last_idx: -1,
        }
    }

    pub fn new_with_max(max: usize) -> Self {
        let mut b = Bspline {
            max: 0,
            num: 0,
            x: 0,
            y: 0,
            am: PodArray::new(),
            last_idx: -1,
        };
        b.init(max);
        b
    }

    pub fn new_with_points(num: usize, x: &[f64], y: &[f64]) -> Self {
        let mut b = Bspline {
            max: num,
            num,
            x: 0,
            y: 0,
            am: PodArray::new(),
            last_idx: -1,
        };
        b.init_with_points(num, x, y);
        b
    }

    pub fn init(&mut self, max: usize) {
        if max > 2 && max > self.max {
            self.am.resize(max * 3, 0.0);
            self.x = max;
            self.y = max * 2;
            self.max = max;
        }
        self.num = 0;
        self.last_idx = -1;
    }

    pub fn add_point(&mut self, x: f64, y: f64) {
        if self.num < self.max {
            self.am[self.num + self.x] = x;
            self.am[self.num + self.y] = y;
            self.num += 1;
        }
    }

    pub fn prepare(&mut self) {
        if self.num > 2 {
            for k in 0..self.num {
                self.am[k] = 0.0;
            }

            let mut n1 = 3 * self.num;
            let mut al = Vec::new();
            al.resize(n1, 0.0);

            let r = self.num;
            let s = self.num * 2;
            n1 = self.num - 1;
            let mut d = self.am[self.x + 1] - self.am[self.x + 0];
            let mut e = (self.am[self.y + 1] - self.am[self.y + 0]) / d;
            let mut h;
            let mut f;
            for k in 1..n1 {
                h = d;
                d = self.am[self.x + k + 1] - self.am[self.x + k];
                f = e;
                e = (self.am[self.y + k + 1] - self.am[self.y + k]) / d;
                al[k] = d / (d + h);
                al[r + k] = 1.0 - al[k];
                al[s + k] = 6.0 * (e - f) / (h + d);
            }
            let mut p;
            for k in 1..n1 {
                p = 1.0 / (al[r + k] * al[k - 1] + 2.0);
                al[k] *= -p;
                al[s + k] = (al[s + k] - al[r + k] * al[s + k - 1]) * p;
            }
            self.am[n1] = 0.0;
            al[n1 - 1] = al[s + n1 - 1];
            self.am[n1 - 1] = al[n1 - 1];

            for i in 0..self.num - 2 {
                let k = n1 - 2 - i;
                al[k] = al[k] * al[k + 1] + al[s + k];
                self.am[k] = al[k];
                //k -= 1;
            }
        }
        self.last_idx = -1;
    }

    pub fn init_with_points(&mut self, num: usize, x: &[f64], y: &[f64]) {
        if num > 2 {
            self.init(num);
            for i in 0..num {
                self.add_point(x[i], y[i]);
            }
            self.prepare();
        }
        self.last_idx = -1;
    }

    fn bsearch(&self, n: usize, x: &[f64], x0: f64, i: &mut usize) {
        let mut j = n - 1;
        let mut k;

        *i = 0;
        while (j - *i) > 1 {
            k = (*i + j) >> 1;
            if x0 < x[k] {
                j = k;
            } else {
                *i = k;
            }
        }
    }

    fn interpolation(&self, x: f64, i: usize) -> f64 {
        let j = i + 1;
        let d = self.am[self.x + i] - self.am[self.x + j];
        let h = x - self.am[self.x + j];
        let r = self.am[self.x + i] - x;
        let p = d * d / 6.0;
        return (self.am[j] * r * r * r + self.am[i] * h * h * h) / 6.0 / d
            + ((self.am[self.y + j] - self.am[j] * p) * r
                + (self.am[self.y + i] - self.am[i] * p) * h)
                / d;
    }

    fn extrapolation_left(&self, x: f64) -> f64 {
        let d = self.am[self.x + 1] - self.am[self.x + 0];
        return (-d * self.am[1] / 6.0 + (self.am[self.y + 1] - self.am[self.y + 0]) / d)
            * (x - self.am[self.x + 0])
            + self.am[self.y + 0];
    }

    fn extrapolation_right(&self, x: f64) -> f64 {
        let d = self.am[self.x + self.num - 1] - self.am[self.x + self.num - 2];
        return (d * self.am[self.num - 2] / 6.0
            + (self.am[self.y + self.num - 1] - self.am[self.y + self.num - 2]) / d)
            * (x - self.am[self.x + self.num - 1])
            + self.am[self.y + self.num - 1];
    }

    pub fn get(&self, x: f64) -> f64 {
        if self.num > 2 {
            let mut i: usize = 0;

            // Extrapolation on the left
            if x < self.am[self.x + 0] {
                return self.extrapolation_left(x);
            }

            // Extrapolation on the right
            if x >= self.am[self.x + self.num - 1] {
                return self.extrapolation_right(x);
            }

            // Interpolation
            self.bsearch(self.num, &self.am[self.x..], x, &mut i);
            return self.interpolation(x, i);
        }
        return 0.0;
    }

    pub fn get_stateful(&mut self, x: f64) -> f64 {
        if self.num > 2 {
            // Extrapolation on the left
            if x < self.am[self.x + 0] {
                return self.extrapolation_left(x);
            }

            // Extrapolation on the right
            if x >= self.am[self.x + self.num - 1] {
                return self.extrapolation_right(x);
            }

            let mut last_idx = self.last_idx as usize;
            if self.last_idx >= 0 {
                // Check if x is not in current range
                if x < self.am[self.x + last_idx] || x > self.am[self.x + last_idx + 1] {
                    // Check if x between next points (most probably)
                    if last_idx < self.num - 2
                        && x >= self.am[self.x + last_idx + 1]
                        && x <= self.am[self.x + last_idx + 2]
                    {
                        self.last_idx += 1;
                    } else if last_idx > 0
                        && x >= self.am[self.x + last_idx - 1]
                        && x <= self.am[self.x + last_idx]
                    {
                        // x is between pevious points
                        self.last_idx -= 1;
                    } else {
                        // Else perform full search
                        self.bsearch(self.num, &self.am[self.x..], x, &mut last_idx);
                        self.last_idx = last_idx as isize;
                    }
                }
                return self.interpolation(x, self.last_idx as usize);
            } else {
                // Interpolation
                self.bsearch(self.num, &self.am[self.x..], x, &mut last_idx);
                self.last_idx = last_idx as isize;
                return self.interpolation(x, last_idx);
            }
        }
        return 0.0;
    }
}

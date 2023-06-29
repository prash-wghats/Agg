//=============================================================swap_arrays
fn swap_arrays<T: Copy>(a1: *mut T, a2: *mut T, n: u32) {
    for i in 0..n as isize {
        unsafe {
            let tmp = *a1.offset(i);
            *a1.offset(i) = *a2.offset(i);
            *a2.offset(i) = tmp;
        }
    }
}

struct MatrixPivot<const ROWS: usize, const COLS: usize>;

impl<const ROWS: usize, const COLS: usize> MatrixPivot<ROWS, COLS> {
    //============================================================MatrixPivot
    fn pivot(m: &mut [[f64; COLS]; ROWS], row: usize) -> isize {
        let mut k = row as isize;
        let mut max_val: f64;
        let mut tmp: f64;

        max_val = -1.0;
        for i in row..ROWS as usize {
            tmp = m[i][row].abs();
            if tmp > max_val && tmp != 0.0 {
                max_val = tmp;
                k = i as isize;
            }
        }

        if m[k as usize][row] == 0.0 {
            return -1;
        }

        if k != row as isize {
            swap_arrays(m[k as usize].as_mut_ptr(), m[row].as_mut_ptr(), COLS as u32);
            return k;
        }
        return 0;
    }
}

//===============================================================SimulEq
pub struct SimulEq<const SIZE: usize, const RIGHT_COLS: usize, const HACK: usize>;
impl<const SIZE: usize, const RIGHT_COLS: usize, const HACK: usize> SimulEq<SIZE, RIGHT_COLS, HACK> {
    pub fn solve(
        left: &[[f64; SIZE]; SIZE], right: &[[f64; RIGHT_COLS]; SIZE],
        result: &mut [[f64; RIGHT_COLS]; SIZE],
    ) -> bool {
        let mut a1: f64;
        let size = SIZE;
        let rcols = RIGHT_COLS;
        let mut tmp :[[f64; HACK/*RIGHT_COLS+SIZE*/]; SIZE] = [[0.; HACK/*RIGHT_COLS+SIZE*/]; SIZE];

        for i in 0..size {
            for j in 0..size {
                tmp[i][j] = left[i][j];
            }
            for j in 0..rcols {
                tmp[i][size + j] = right[i][j];
            }
        }

        for k in 0..size {
            if MatrixPivot::<SIZE, HACK>::pivot(&mut tmp, k) < 0 {
                return false; // Singularity....
            }

            a1 = tmp[k][k];

            for j in k..SIZE + RIGHT_COLS {
                tmp[k][j] /= a1;
            }

            for i in k + 1..SIZE {
                a1 = tmp[i][k];
                for j in k..SIZE + RIGHT_COLS {
                    tmp[i][j] -= a1 * tmp[k][j];
                }
            }
        }

        for k in 0..RIGHT_COLS {

            for m in (0..SIZE as isize).rev() {
                result[m as usize][k] = tmp[m as usize][SIZE + k];
                for j in m as usize + 1..SIZE {
                    result[m as usize][k] -= tmp[m as usize][j] * result[j][k];
                }
            }
        }
        return true;
    }
}

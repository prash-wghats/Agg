use crate::basics::{is_close, is_move_to, is_stop, is_vertex};
use crate::VertexSource;

pub fn path_length<VS: VertexSource>(vs: &mut VS, path_id: u32) -> f64 {
    let mut len = 0.0;
    let mut start_x = 0.0;
    let mut start_y = 0.0;
    let mut x1 = 0.0;
    let mut y1 = 0.0;
    let mut x2 = 0.0;
    let mut y2 = 0.0;
    let mut first = true;
    let mut cmd;
    vs.rewind(path_id);
    loop {
        cmd = vs.vertex(&mut x2, &mut y2);
        if is_stop(cmd) {
            break;
        }
        if is_vertex(cmd) {
            if first || is_move_to(cmd) {
                start_x = x2;
                start_y = y2;
            } else {
                len += crate::math::calc_distance(x1, y1, x2, y2);
            }
            x1 = x2;
            y1 = y2;
            first = false;
        } else {
            if is_close(cmd) && !first {
                len += crate::math::calc_distance(x1, y1, start_x, start_y);
            }
        }
    }
    len
}

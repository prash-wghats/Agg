pub const GPC_OP_GPC_DIFF: GpcOpType = 0;
pub const GPC_OP_GPC_INT: GpcOpType = 1;
pub const GPC_OP_GPC_XOR: GpcOpType = 2;
pub const GPC_OP_GPC_UNION: GpcOpType = 3;
pub type GpcOpType = ::std::os::raw::c_int;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpcVertex {
    pub x: f64,
    pub y: f64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpcVertexList {
    pub num_vertices: ::std::os::raw::c_int,
    pub vertex: *mut GpcVertex,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpcPolygon {
    pub num_contours: ::std::os::raw::c_int,
    pub hole: *mut ::std::os::raw::c_int,
    pub contour: *mut GpcVertexList,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct GpcTristrip {
    pub num_strips: ::std::os::raw::c_int,
    pub strip: *mut GpcVertexList,
}

/*extern "C" {
    pub fn gpc_read_polygon(
        infile_ptr: *mut FILE, read_hole_flags: ::std::os::raw::c_int, polygon: *mut GpcPolygon,
    );
}
extern "C" {
    pub fn gpc_write_polygon(
        outfile_ptr: *mut FILE, write_hole_flags: ::std::os::raw::c_int, polygon: *mut GpcPolygon,
    );
}*/
extern "C" {
    pub fn gpc_add_contour(
        polygon: *mut GpcPolygon, contour: *mut GpcVertexList, hole: ::std::os::raw::c_int,
    );
}
extern "C" {
    pub fn gpc_polygon_clip(
        set_operation: GpcOpType, subject_polygon: *mut GpcPolygon, clip_polygon: *mut GpcPolygon,
        result_polygon: *mut GpcPolygon,
    );
}
extern "C" {
    pub fn gpc_tristrip_clip(
        set_operation: GpcOpType, subject_polygon: *mut GpcPolygon, clip_polygon: *mut GpcPolygon,
        result_tristrip: *mut GpcTristrip,
    );
}
extern "C" {
    pub fn gpc_polygon_to_tristrip(polygon: *mut GpcPolygon, tristrip: *mut GpcTristrip);
}
extern "C" {
    pub fn gpc_free_polygon(polygon: *mut GpcPolygon);
}
extern "C" {
    pub fn gpc_free_tristrip(tristrip: *mut GpcTristrip);
}


fn main() {
    if std::env::var("CARGO_FEATURE_LIBGPC").is_ok() {
        // Tell Cargo that if the given file changes, to rerun this build script.
        println!("cargo:rerun-if-changed=gpc/gpc.c");
        // Use the `cc` crate to build a C file and statically link it.
        cc::Build::new().file("gpc/gpc.c").compile("gpc");
    }
	if std::env::var("CARGO_FEATURE_X11").is_ok() {
	println!("cargo:rustc-link-lib=X11");
	}
}

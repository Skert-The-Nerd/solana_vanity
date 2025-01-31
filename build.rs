fn main() {
    if cfg!(feature = "gpu") {
        println!("cargo:rustc-link-lib=OpenCL");
    }
}

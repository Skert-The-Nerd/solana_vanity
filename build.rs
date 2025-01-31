// build.rs

fn main() {
    if cfg!(feature = "gpu") {
        // Example: Compiling an OpenCL kernel
        cc::Build::new()
            .file("src/gpu_kernel.cl") // Replace with your kernel file
            .compile("gpu_kernel");

        // Link against OpenCL
        println!("cargo:rustc-link-lib=OpenCL");
    }
}

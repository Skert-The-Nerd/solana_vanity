// build.rs

fn main() {
    // If the 'gpu' feature is enabled, perform GPU-related build steps
    if cfg!(feature = "gpu") {
        // Link against OpenCL library
        println!("cargo:rustc-link-lib=OpenCL");
        
        // If you have custom OpenCL or CUDA code to compile, use the 'cc' crate
        // Example: Compiling an OpenCL kernel written in C
        // Uncomment and modify the following lines if you have C/C++ GPU kernels

        /*
        cc::Build::new()
            .file("src/gpu_kernel.c") // Path to your GPU kernel C file
            .include("src/") // Include path if necessary
            .compile("gpu_kernel");
        */
    }
}

// build.rs

fn main() {
    // If the 'gpu' feature is enabled, perform GPU-related build steps
    if cfg!(feature = "gpu") {
        // Example: Linking against OpenCL libraries
        println!("cargo:rustc-link-lib=OpenCL");
        
        // Specify include paths or other configurations if necessary
        // println!("cargo:include=/path/to/opencl/include");
        
        // If you have custom OpenCL or CUDA code to compile, invoke the compiler here
        // This might involve using the 'cc' crate to compile C/C++ kernels
    }
}

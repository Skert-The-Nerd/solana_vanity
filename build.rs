fn main() {
    if cfg!(feature = "cuda") {
        cc::Build::new()
            .cuda(true)
            .flag("-arch=sm_86")
            .file("cuda/kernel.cu")
            .compile("kernel");
        
        println!("cargo:rerun-if-changed=cuda/kernel.cu");
    }
    println!("cargo:rerun-if-changed=build.rs");
}

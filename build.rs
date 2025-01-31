fn main() {
    #[cfg(feature = "cuda")]
    {
        cc::Build::new()
            .cuda(true)
            .flag("-arch=sm_86") // RTX 3090 specific
            .file("cuda/kernel.cu")
            .compile("kernel");
        
        println!("cargo:rerun-if-changed=cuda/kernel.cu");
    }
    println!("cargo:rerun-if-changed=build.rs");
}

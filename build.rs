fn main() {
    cc::Build::new()
        .cuda(true)
        .flag("-arch=sm_86")  // For RTX 3090
        .file("cuda/kernel.cu")
        .compile("kernel");
    
    println!("cargo:rerun-if-changed=cuda/kernel.cu");
}

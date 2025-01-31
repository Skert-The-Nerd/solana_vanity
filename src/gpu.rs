use ocl::{Platform, Device, Context, Queue, Program, Buffer, Kernel};

pub fn grind(target: String, case_insensitive: bool, num_threads: u32) {
    let platform = Platform::default();
    let device = Device::first(platform).expect("No GPU device found!");
    let context = Context::builder().platform(platform).devices(device.clone()).build().unwrap();
    let queue = Queue::new(&context, device, None).unwrap();

    let kernel_source = include_str!("gpu_kernel.cl");

    let program = Program::builder().src(kernel_source).build(&context).unwrap();
    let buffer = Buffer::<u8>::builder().queue(queue.clone()).len(num_threads).build().unwrap();

    let kernel = Kernel::builder()
        .program(&program)
        .name("vanity_search")
        .queue(queue.clone())
        .global_work_size(num_threads)
        .arg(&buffer)
        .build().unwrap();

    println!("Launching GPU kernel...");
    unsafe {
        kernel.enq().unwrap();
    }

    println!("GPU acceleration is running!");
}

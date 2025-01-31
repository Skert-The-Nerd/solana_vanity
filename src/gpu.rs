use ocl::{Platform, Device, Context, Queue, Program, Buffer, Kernel};

pub fn grind(target: String, case_insensitive: bool, num_threads: u32) {
    let platform = Platform::default();
    let device = Device::first(platform).expect("No GPU device found!");
    let context = Context::builder().platform(platform).devices(device.clone()).build().unwrap();
    let queue = Queue::new(&context, device, None).unwrap();

    let kernel_source = include_str!("gpu_kernel.cl");

    let program = Program::builder().src(kernel_source).build(&context).unwrap();
    let seeds = Buffer::<u8>::builder().queue(queue.clone()).len(32 * num_threads).build().unwrap();
    let results = Buffer::<u8>::builder().queue(queue.clone()).len(32).build().unwrap();
    let match_found = Buffer::<i32>::builder().queue(queue.clone()).len(1).build().unwrap();

    let kernel = Kernel::builder()
        .program(&program)
        .name("vanity_search")
        .queue(queue.clone())
        .global_work_size(num_threads)
        .arg(&seeds)
        .arg(&results)
        .arg(&match_found)
        .arg(&target)
        .arg(target.len() as i32)
        .build().unwrap();

    println!("Launching GPU kernel...");
    unsafe {
        kernel.enq().unwrap();
    }

    println!("GPU processing complete!");
}

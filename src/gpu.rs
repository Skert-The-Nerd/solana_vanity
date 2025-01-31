use ocl::{Platform, Device, Context, Queue, Program, Buffer, Kernel};

pub fn grind(target: String, case_insensitive: bool, num_threads: u32) {
    let platform = Platform::default();
    let device = Device::first(platform).expect("No GPU device found!");
    let context = Context::builder().platform(platform).devices(device.clone()).build().unwrap();
    let queue = Queue::new(&context, device, None).unwrap();

    let kernel_source = include_str!("gpu_kernel.cl");

    let program = Program::builder().src(kernel_source).build(&context).unwrap();

    // Buffers
    let seeds = Buffer::<u8>::builder().queue(queue.clone()).len(32 * num_threads).build().unwrap();
    let results = Buffer::<u8>::builder().queue(queue.clone()).len(32).build().unwrap();
    let match_found = Buffer::<i32>::builder().queue(queue.clone()).len(1).build().unwrap();
    let match_index = Buffer::<i32>::builder().queue(queue.clone()).len(1).build().unwrap();

    // Write the target to a fixed-size buffer
    let target_buffer = Buffer::<u8>::builder()
        .queue(queue.clone())
        .len(target.len())
        .build()
        .unwrap();

    queue.write(&target_buffer, 0, target.as_bytes()).enq().unwrap();

    // Kernel
    let kernel = Kernel::builder()
        .program(&program)
        .name("vanity_search")
        .queue(queue.clone())
        .global_work_size(num_threads)
        .arg(&seeds)          // First argument: seeds
        .arg(&results)        // Second argument: results
        .arg(&match_found)    // Third argument: match_found
        .arg(&match_index)    // Fourth argument: match_index
        .arg(&target_buffer)  // Fifth argument: target (prefix)
        .arg(target.len() as i32) // Sixth argument: target length
        .build()
        .unwrap();

    println!("Launching GPU kernel...");
    unsafe {
        kernel.enq().unwrap();
    }

    // Read the results back
    let mut found = vec![0; 1];
    let mut matched_key = vec![0; 32];
    queue.read(&match_found, &mut found).enq().unwrap();
    queue.read(&results, &mut matched_key).enq().unwrap();

    if found[0] == 1 {
        println!("Match Found: {:?}", matched_key);
    } else {
        println!("No match found.");
    }
}

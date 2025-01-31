use ocl::{Platform, Device, Context, Queue, Program, Buffer, Kernel};

pub fn grind(_target: String, _case_insensitive: bool, num_threads: u32) {
    let platform = Platform::default();
    let device = Device::first(platform).expect("No GPU device found!");
    let context = Context::builder().platform(platform).devices(device.clone()).build().unwrap();
    let queue = Queue::new(&context, device, None).unwrap();

    let kernel_source = include_str!("gpu_kernel.cl");

    let program = Program::builder().src(kernel_source).build(&context).unwrap();

    // Buffers (Correctly passing all 3 required arguments)
    let seeds = Buffer::<u8>::builder().queue(queue.clone()).len(32 * num_threads).build().unwrap();
    let results = Buffer::<u8>::builder().queue(queue.clone()).len(32).build().unwrap();
    let match_found = Buffer::<i32>::builder().queue(queue.clone()).len(1).build().unwrap();

    // Initialize seeds buffer with random data
    let mut seed_data = vec![0u8; 32 * num_threads];
    for byte in seed_data.iter_mut() {
        *byte = rand::random::<u8>();
    }
    queue.write(&seeds, 0, &seed_data).enq().unwrap();

    // Initialize match_found buffer
    queue.write(&match_found, 0, &[0]).enq().unwrap();

    // Correct Kernel Call
    let kernel = Kernel::builder()
        .program(&program)
        .name("vanity_search")
        .queue(queue.clone())
        .global_work_size(num_threads)
        .arg(&seeds)         // ✅ First argument (seeds)
        .arg(&results)       // ✅ Second argument (results)
        .arg(&match_found)   // ✅ Third argument (match_found)
        .build()
        .unwrap();

    println!("Launching GPU kernel...");
    unsafe {
        kernel.enq().unwrap();
    }

    // Read match result
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

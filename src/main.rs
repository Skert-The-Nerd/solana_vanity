use clap::Parser;

mod cpu;
mod gpu;

#[derive(Debug, Parser)]
#[command(name = "Solana Vanity Address Generator")]
struct Args {
    /// Target prefix to search for (case-sensitive)
    #[arg(short, long)]
    target: String,

    /// Use case-insensitive search
    #[arg(short, long, default_value_t = false)]
    case_insensitive: bool,

    /// Number of CPU threads to use
    #[arg(short, long, default_value_t = 0)]
    threads: u32,
}

fn main() {
    let args = Args::parse();

    #[cfg(feature = "gpu")]
    {
        gpu::grind(args.target, args.case_insensitive, args.threads);
    }

    #[cfg(not(feature = "gpu"))]
    {
        cpu::grind(args.target, args.case_insensitive, args.threads);
    }
}

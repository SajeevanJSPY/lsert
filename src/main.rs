mod args;
mod file_types;
mod io_control;
mod lexical_analysis;
mod serve;

use args::Args;

fn main() -> redis::RedisResult<()> {
    let args = Args::new();
    args.implication();

    Ok(())
}

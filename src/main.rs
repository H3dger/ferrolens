use ferrolens::error::Result;

fn main() -> Result<()> {
    ferrolens::run_with_args(std::env::args_os())
}

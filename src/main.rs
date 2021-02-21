use structopt::StructOpt;

fn main() -> ! {
    let exit_code = {
        let args = StructOpt::from_args();
        match archman::run(args) {
            Ok(_) => 0,
            Err(err) => {
                println!("Error: {}", err);
                1
            }
        }
    };
    std::process::exit(exit_code)
}

use ansi_term::Colour;
use structopt::StructOpt;

fn main() -> ! {
    let exit_code = {
        let args = StructOpt::from_args();
        match archman::run(args) {
            Ok(()) => 0,
            Err(err) => {
                println!("\n{} {:?}", Colour::Red.bold().paint("Error:"), err);
                1
            }
        }
    };
    std::process::exit(exit_code)
}

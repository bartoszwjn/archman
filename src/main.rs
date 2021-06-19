use ansi_term::Colour;
use structopt::StructOpt;

fn main() -> ! {
    let args = StructOpt::from_args();
    let exit_code = match archman::run(args) {
        Ok(()) => 0,
        Err(err) => {
            let is_tty = atty::is(atty::Stream::Stderr);
            let style = if is_tty {
                Colour::Red.bold()
            } else {
                Default::default()
            };
            eprintln!("\n{} {:?}", style.paint("error:"), err);
            1
        }
    };
    std::process::exit(exit_code)
}

use anstyle::AnsiColor;
use clap::Parser;
use is_terminal::IsTerminal;

fn main() -> ! {
    let args = Parser::parse();
    let exit_code = match archman::run(args) {
        Ok(()) => 0,
        Err(err) => {
            let is_tty = std::io::stderr().is_terminal();
            let style = if is_tty {
                AnsiColor::Red.on_default().bold()
            } else {
                Default::default()
            };
            eprintln!(
                "\n{}{}{} {:?}",
                style.render(),
                "error:",
                style.render_reset(),
                err,
            );
            1
        }
    };
    std::process::exit(exit_code)
}

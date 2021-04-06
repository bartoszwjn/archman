//! Utilities.

/// Prints a warning to `stderr` using colours if `stderr` is connected to a terminal.
macro_rules! warn {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::atty::is(::atty::Stream::Stderr);
            let style = if use_style {
                ::ansi_term::Colour::Yellow.bold()
            } else {
                ::ansi_term::Style::new()
            };
            eprint!("{}warning:{} ", style.prefix(), style.suffix());
            eprintln!($($fmt),+);
        }
    }
}

/// Print an info string to `stdout` using colours if `stdout` is connected to a terminal.
macro_rules! info {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::atty::is(::atty::Stream::Stdout);
            let style = if use_style {
                ::ansi_term::Colour::Blue.bold()
            } else {
                ::ansi_term::Style::new()
            };
            print!("{}", style.prefix());
            print!($($fmt),+);
            println!("{}", style.suffix());
        }
    }
}

/// Prints a string `stdout` using a bold style if `stdout` is connected to a terminal.
macro_rules! bold {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::atty::is(::atty::Stream::Stdout);
            let style = if use_style {
                ::ansi_term::Style::new().bold()
            } else {
                ::ansi_term::Style::new()
            };
            print!("{}", style.prefix());
            print!($($fmt),+);
            println!("{}", style.suffix());
        }
    }
}

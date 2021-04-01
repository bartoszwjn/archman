//! Utilities.

/// Prints `s` to `stdout` adding the given [`Style`] or [`Colour`] if `stdout` is connected to a
/// terminal.
macro_rules! println_styled {
    ($style:expr, $($rest:expr),+ $(,)?) => {
        {
            let style = $style;
            let style = if ::atty::is(::atty::Stream::Stdout) {
                style.into()
            } else {
                ::ansi_term::Style::default()
            };
            print!("{}", style.prefix());
            print!($($rest),+);
            println!("{}", style.suffix());
        }
    }
}

/// Prints `s` to `stderr` adding the given [`Style`] or [`Colour`] if `stderr` is connected to a
/// terminal.
macro_rules! eprintln_styled {
    ($style:expr, $($rest:expr),+ $(,)?) => {
        {
            let style = $style;
            let style = if ::atty::is(::atty::Stream::Stdout) {
                style.into()
            } else {
                ::ansi_term::Style::default()
            };
            eprint!("{}", style.prefix());
            eprint!($($rest),+);
            eprintln!("{}", style.suffix());
        }
    }
}

//! Utilities.

/// Prints an error to `stderr` using colours if `stderr` is connected to a terminal.
macro_rules! error {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::is_terminal::IsTerminal::is_terminal(&::std::io::stderr());
            let style = if use_style {
                ::anstyle::AnsiColor::Red.on_default().bold()
            } else {
                ::anstyle::Style::new()
            };
            eprintln!(
                "{}{}{} {}",
                style.render(),
                "error:",
                style.render_reset(),
                ::core::format_args!($($fmt),+),
            );
        }
    }
}

/// Prints a warning to `stderr` using colours if `stderr` is connected to a terminal.
macro_rules! warn {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::is_terminal::IsTerminal::is_terminal(&::std::io::stderr());
            let style = if use_style {
                ::anstyle::AnsiColor::Yellow.on_default().bold()
            } else {
                ::anstyle::Style::new()
            };
            eprintln!(
                "{}{}{} {}",
                style.render(),
                "warning:",
                style.render_reset(),
                ::core::format_args!($($fmt),+),
            );
        }
    }
}

/// Prints an info string to `stdout` using a bold style if `stdout` is connected to a terminal.
macro_rules! info {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::is_terminal::IsTerminal::is_terminal(&::std::io::stdout());
            let style = if use_style {
                ::anstyle::Style::new().bold()
            } else {
                ::anstyle::Style::new()
            };
            eprintln!(
                "{}{}{} {}",
                style.render(),
                "info:",
                style.render_reset(),
                ::core::format_args!($($fmt),+),
            );
        }
    }
}

/// Prints a coloured string to `stdout` using colours if `stdout` is connected to a terminal.
macro_rules! colour {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::is_terminal::IsTerminal::is_terminal(&::std::io::stdout());
            let style = if use_style {
                ::anstyle::AnsiColor::Blue.on_default().bold()
            } else {
                ::anstyle::Style::new()
            };
            println!(
                "{}{}{}",
                style.render(),
                ::core::format_args!($($fmt),+),
                style.render_reset(),
            );
        }
    }
}

/// Prints a string `stdout` using a bold style if `stdout` is connected to a terminal.
macro_rules! bold {
    ($($fmt:expr),+ $(,)?) => {
        {
            let use_style = ::is_terminal::IsTerminal::is_terminal(&::std::io::stdout());
            let style = if use_style {
                ::anstyle::Style::new().bold()
            } else {
                ::anstyle::Style::new()
            };
            println!(
                "{}{}{}",
                style.render(),
                ::core::format_args!($($fmt),+),
                style.render_reset(),
            );
        }
    }
}

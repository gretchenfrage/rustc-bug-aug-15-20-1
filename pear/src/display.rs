
use crate::Error;
use std::fmt::{self, Display, Debug, Formatter};
use unicode_width::UnicodeWidthChar;
use ansi_parser::AnsiParser;
use textwrap::wrap;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt_error(f, self, Spaces(0), true, false)?;
        Ok(())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

const INDENT_SPACES: usize = 4;

fn display_width(s: &str) -> usize {
    let mut max = 0;
    let mut curr = 0;
    for c in s.ansi_parse()
        .filter_map(|output| match output {
            ansi_parser::Output::TextBlock(s) => Some(s),
            _ => None,
        })
        .flat_map(str::chars)
    {
        if c == '\n' {
            curr = 0;
        } else {
            curr += c.width().unwrap_or(0);
            max = max.max(curr);
        }
    }
    max
}

fn fmt_error(
    f: &mut Formatter, 
    error: &Error, 
    indent: Spaces,
    braced: bool,
    end_of_error_newline: bool,
) -> fmt::Result {
    const DEBUG_ALT_TRIGGER: usize = 50;
    
    let indent2 = Spaces(indent.0 + INDENT_SPACES);

    if braced {
        write!(f, "{}[ error ]\n", indent)?;
    }

    if error.0.wrap_enabled {
        for line in wrap(&error.0.message, 60) {
            write!(f, "{}{}\n", indent2, line)?;
        }
    } else {
        for line in error.0.message.lines() {
            write!(f, "{}{}\n", indent2, line)?;
        }
    }

    for (key, val) in error.0.fields.iter() {
        let key_width = display_width(key) + 5;
        write!(f, "{}- {} = ", indent2, key)?;

        let val_str = {
            if display_width(&val.debug) <= DEBUG_ALT_TRIGGER {
                &val.debug
            } else {
                &val.debug_alt
            }
        };

        for (i, line) in val_str.lines().enumerate() {
            if i > 0 {
                write!(f, "{}{}", indent2, Spaces(key_width))?;
            }
            write!(f, "{}\n", line)?;
        }
    }

    if let Some(backtrace) = error.0.backtrace.as_ref() {
        let backtrace_str = {
            if f.alternate() {
                format!("{:#?}", backtrace)
            } else {
                format!("{:?}", backtrace)
            }
        };
        for line in backtrace_str.lines() {
            write!(f, "{}{}\n", indent2, line)?;
        }
    }

    if error.0.causes.len() == 1 {
        write!(f, "{}[ caused by ]\n", indent)?;
        fmt_error(f, &error.0.causes[0], indent, false, true)?;
    } else if error.0.causes.len() > 1 {
        for (i, cause) in error.0.causes.iter().enumerate() {
            write!(
                f, 
                "{}[ caused by ({}/{}) ]\n", 
                indent, 
                i + 1, 
                error.0.causes.len(),
            )?;
            fmt_error(f, cause, indent2, true, true)?;
        }
    }

    if braced {
        write!(f, "{}[ end of error ]", indent)?;
        if end_of_error_newline {
            f.write_str("\n")?;
        }
    }

    Ok(())
}

#[derive(Copy, Clone)]
struct Spaces(usize);

impl Display for Spaces {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for _ in 0..self.0 {
            f.write_str(" ")?;
        }
        Ok({})
    }
}
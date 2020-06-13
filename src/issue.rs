#![allow(non_camel_case_types)]
#![allow(clippy::pub_enum_variant_names)]

use crate::site::{Site, Location};

use std::fs;
use std::fmt;
use std::io::{BufRead, BufReader};

use colored;
use colored::*;

use unindent::unindent;

#[derive(Clone, Copy)]
pub enum Kind {
      LexError,   LexWarn,
    ParseError, ParseWarn,
     TypeError,  TypeWarn,
     CompError,  CompWarn
}

#[derive(Clone)]
pub struct Issue {
    pub kind : Kind,
    pub site : Site,
    pub message : String,
    note_message : Option<String>,
    pub is_fatal : bool,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Kind::LexError   => "Lexicographical Error".red(),
            Kind::ParseError =>         "Grammar Error".red(),
            Kind::TypeError  =>          "Typing Error".red(),
            Kind::CompError  =>     "Compilation Error".red(),
            Kind::LexWarn    => "Lexicographical Warning".yellow(),
            Kind::ParseWarn  =>         "Grammar Warning".yellow(),
            Kind::TypeWarn   =>          "Typing Warning".yellow(),
            Kind::CompWarn   =>     "Compilation Warning".yellow(),
        };
        write!(f, "{}", printable)
    }
}

impl Issue {
    #[must_use = "Issue must be displayed"]
    pub fn new(kind : Kind, site : Site, fmt_msg : String) -> Self {
        Self {
            kind,
            site: site.clone(),
            note_message: None,
            message: unindent(&fmt_msg)
                .split('\n')
                .collect::<Vec<&str>>()
                .join("\n  "),
            is_fatal: false
        }
    }

    #[must_use = "Issue must be displayed"]
    pub fn fatal(mut self) -> Self {
        self.is_fatal = true;
        self
    }

    #[must_use = "Issue must be displayed"]
    pub fn note(mut self, msg : &str) -> Self {
        self.note_message = Some(msg.to_owned());
        self
    }

    pub fn print(self) -> Self {
        unsafe {
            crate::PANIC_MESSAGE = "Compilation could not continue.";
        };

        eprintln!("\n{}", self);
        if self.is_fatal {
            std::process::exit(1);
        }
        self
    }

    pub fn crash_and_burn(self) -> ! {
        self.print();
        std::process::exit(1)
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{}{} {}",
            "issue".bold().red(),
            ":".white(),
            self.message.bold())?;
        write!(f, "{}", "".clear())?;

        write!(f, " ==> {kind}",
            kind = self.kind.to_string().bold())?;

        let mut got_file = false;

        if let Some(path) = &self.site.path {
            got_file = true;
            write!(f, " in ({}", path.to_string_lossy())?;
        } else if self.site.repl {
            got_file = true;
            write!(f, " in (<REPL>")?;
        }

        if let Some(line) = self.site.location.line {
            if got_file {
                write!(f, ":{}", line)?;
            } else {
                got_file = true;
                write!(f, " in ({}", line)?;
            }
        }

        if let Some(column) = self.site.location.column {
            if got_file {
                write!(f, ":{}", column)?;
            } else {
                got_file = true;
                write!(f, " at (column: {}", column)?;
            }
        }

        if got_file {
            write!(f, ")")?;
        }

        let indent = 5;

        let mut opened_file = false;
        let mut multi_line = false;
        if let Some(path) = &self.site.path {
            let file_result = fs::File::open(path);
            if let Ok(file) = file_result {
                opened_file = true;
                if let Location {
                    line: Some(line),
                    lines: Some(lines),
                    ..
                } = self.site.location {
                    if lines == 1 {
                        let mut line_content = if let Some(Ok(line_str)) =
                            BufReader::new(file)
                                .lines().nth(line - 1) {
                            line_str
                        } else {
                            "[**] THIS LINE DOES NOT EXIST! \
                                  Either the file was deleted, \
                                  or this is a bug!".to_string()
                        };
                        if let Location {
                            column: Some(column),
                            last_column: Some(last_column),
                            ..
                        } = self.site.location {
                            let (i, j) = (column - 1, last_column - 1);
                            line_content = format!("{}{}{}",
                                &line_content[..i],
                                &line_content[i..j]
                                    .white().bold(),
                                &line_content[j..]);
                        }
                        writeln!(f, ":\n{space}|\n{line_num}| {line_str}",
                            space = " ".repeat(indent),
                            line_num = format!("{: >4} ", line).bold(),
                            line_str = line_content)?;
                    } else {  // Error spans multiple lines
                        multi_line = true;
                        // TODO: Display the lines.
                    }
                }
            }
        }

        let note_ascii = self.note_message.as_ref().map(|some|
            format!("{} {}",
                "|\n+-".yellow(), some.bold()));

        if let Some(column) = self.site.location.column {
            if opened_file {
                if multi_line {
                    // TODO: Show the arrows for the longest line in the
                    //       multiple lines.
                } else {
                    let columns = self.site.location.columns
                        .unwrap_or(1);
                     writeln!(f, "{space}|{: >offset$}",
                        "^".repeat(columns).yellow().bold(),
                        space=" ".repeat(indent),
                        offset=(column + columns))?;
                    if let Some(note_fmt) = note_ascii {
                        let indented = note_fmt
                            .split("\n")
                            .map(|l| format!("{space} {line}",
                                line=l,
                                space=" ".repeat(indent + column)))
                            .collect::<Vec<String>>()
                            .join("\n");
                        writeln!(f, "{}", indented)?;
                    }
                }
            }
        } else if let Some(note_fmt) = note_ascii {
            writeln!(f, "{}", note_fmt)?;
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! issue {
    ($type:ident, $site:expr, $message:expr) => {
        #[must_use = "Issue must be displayed"] {
            issue::Issue::new(issue::Kind::$type, $site.clone(),
                String::from($message))
        }
    };

    ($type:ident, $site:expr, $message:expr, $($form:expr),*) => {
        #[must_use = "Issue must be displayed"] {
            issue::Issue::new(issue::Kind::$type, $site.clone(),
                format!($message, $($form),*))
        }
    };
}

#[macro_export]
macro_rules! fatal {
    ($type:ident, $($args:tt)*) => {
        #[must_use = "Issue must be displayed"] {
            let mut value = issue!($type, $($args)*);
            value.is_fatal = true;
            value
        }
    };
}

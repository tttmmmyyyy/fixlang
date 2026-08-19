use crate::misc::{insert_to_map_vec, Map, Set};
use crate::parse::sourcefile::Span;
use colored::{Color, Colorize};
use serde_json::Value;
use std::{
    any::Any,
    fmt::{self, Display, Formatter},
    mem, panic,
    path::{Path, PathBuf},
};

/// Diagnostic code for "use of a deprecated item".
pub const WARN_DEPRECATED: &'static str = "deprecated";

/// Diagnostic code for "import of a module whose project is not a declared dependency".
pub const WARN_UNDECLARED_DEPENDENCY: &'static str = "undeclared-dependency";

/// Severity of a diagnostic.
///
/// Errors are fatal and cause compilation to fail. Warnings are non-fatal:
/// they are reported to the user but do not by themselves block compilation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Fatal diagnostic; compilation fails when any error is present.
    Error,
    /// Non-fatal diagnostic; reported but does not block compilation.
    Warning,
}

impl Severity {
    /// Returns the colored CLI label (e.g. red bold "error") and the color
    /// of the `^^^` underline used when rendering associated source spans.
    pub fn label_and_underline_color(&self) -> (String, Color) {
        match self {
            Severity::Error => ("error".red().bold().to_string(), Color::Red),
            Severity::Warning => ("warning".yellow().bold().to_string(), Color::Yellow),
        }
    }
}

/// The diagnostics gathered over a piece of compilation, errors and warnings together.
pub struct Errors {
    /// The diagnostics, in the order they were reported.
    errs: Vec<Error>,
}

impl Display for Errors {
    /// Writes each diagnostic as the compiler prints it, passing over a message already written.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Errors {
    /// A collection holding no diagnostic.
    pub fn empty() -> Errors {
        Errors { errs: vec![] }
    }

    /// Whether this collection contains any item with `Severity::Error`.
    ///
    /// Items with `Severity::Warning` are not counted; a collection that
    /// holds only warnings is treated as a successful compilation.
    pub fn has_error(&self) -> bool {
        self.errs.iter().any(|e| e.severity == Severity::Error)
    }

    /// Whether this collection contains any diagnostic at all (errors or warnings).
    pub fn has_diagnostics(&self) -> bool {
        !self.errs.is_empty()
    }

    /// Takes every diagnostic out of `self` as an error, leaving `self` empty, once one of them
    /// has error severity. A collection of warnings alone is a success and stays where it is, for
    /// a later `take_warnings` to print.
    pub fn to_result(&mut self) -> Result<(), Errors> {
        if self.has_error() {
            Err(mem::replace(self, Errors::empty()))
        } else {
            Ok(())
        }
    }

    /// Drain all warning-severity items into a fresh `Errors`, leaving only
    /// error-severity items in `self`. Useful for printing warnings before
    /// checking `to_result()`.
    pub fn take_warnings(&mut self) -> Errors {
        let (warnings, errors) = mem::take(&mut self.errs)
            .into_iter()
            .partition(|err| err.severity == Severity::Warning);
        self.errs = errors;
        Errors { errs: warnings }
    }

    /// Moves every diagnostic of `other` to the end of this collection, keeping their order.
    pub fn append(&mut self, mut other: Errors) {
        self.errs.append(&mut other.errs);
    }

    /// Appends the diagnostics of a failed `res` to this collection, so that the caller carries on
    /// and reports them together with whatever it finds afterwards.
    pub fn eat_err(&mut self, res: Result<(), Errors>) {
        match res {
            Ok(_v) => {}
            Err(errs) => {
                self.append(errs);
            }
        }
    }

    /// Hands the value of a successful `res` to `act_if_ok`, and appends the diagnostics of a
    /// failed one to this collection.
    pub fn eat_err_or<T>(&mut self, res: Result<T, Errors>, act_if_ok: impl FnOnce(T)) {
        match res {
            Ok(v) => act_if_ok(v),
            Err(errs) => {
                self.append(errs);
            }
        }
    }

    /// A collection holding one error of the given message, with no source location attached.
    pub fn from_msg(msg: String) -> Errors {
        Errors {
            errs: vec![Error::from_msg(msg)],
        }
    }

    /// A collection holding one error of the given message, attached to each of the given
    /// locations that is present.
    pub fn from_msg_srcs(msg: String, srcs: &[&Option<Span>]) -> Errors {
        Errors {
            errs: vec![Error::from_msg_srcs(msg, srcs)],
        }
    }

    /// A collection holding the given diagnostic alone.
    pub fn from_err(err: Error) -> Errors {
        Errors { errs: vec![err] }
    }

    /// A collection holding one error whose message is `msg`, a colon, and the display form of
    /// `err`: `msg` says what the compiler was doing, and `err` is the failure that arose in it.
    pub fn from_msg_err<E>(msg: &str, err: E) -> Errors
    where
        E: Display,
    {
        Errors::from_msg(format!("{}: {}", msg, err))
    }

    /// Renders every diagnostic as the compiler prints it, each on its own lines. A diagnostic
    /// whose rendering repeats one already written is left out.
    pub fn to_string(&self) -> String {
        let mut msg_set = Set::default();
        let mut str = String::default();
        for err in &self.errs {
            let msg = err.to_string();
            if msg_set.contains(&msg) {
                continue;
            }
            msg_set.insert(msg.clone());
            str += &msg;
            str += "\n";
        }
        str
    }

    /// Groups the diagnostics by the file of their first source location, ordered by path.
    ///
    /// # Arguments
    /// * `spanless_fallback` — the file a diagnostic carrying no source location is grouped
    ///   under, so that every diagnostic belongs to some file even where the compiler could not
    ///   point at one.
    pub fn organize_by_path(&self, spanless_fallback: &Path) -> Vec<(PathBuf, Vec<Error>)> {
        // Organize errors into a hashmap.
        let mut map: Map<PathBuf, Vec<Error>> = Map::default();
        for err in &self.errs {
            let path = match err.srcs.first() {
                None => spanless_fallback.to_path_buf(),
                Some((_, span)) => span.input.file_path.clone(),
            };
            insert_to_map_vec(&mut map, &path, err.clone());
        }

        // Convert the hashmap into a vector.
        let mut res = map.into_iter().collect::<Vec<_>>();

        // Sort the vector by the path.
        res.sort_by(|a, b| a.0.cmp(&b.0));

        res
    }
}

/// One diagnostic: what went wrong, where in the source it is, and how severe it is.
#[derive(Clone)]
pub struct Error {
    /// The text shown after the severity label.
    pub msg: String,
    /// The source locations this diagnostic concerns, each paired with the line printed above the
    /// quoted source, such as "The error occurs at:" or "The value is defined at:". The first
    /// location decides which file the diagnostic belongs to.
    pub srcs: Vec<(String, Span)>,
    /// The code naming the kind of this diagnostic, such as `WARN_DEPRECATED`. It reaches an
    /// editor through the language server, which offers a fix for the kinds it recognizes.
    pub code: Option<&'static str>,
    /// Data about this diagnostic beyond its text, such as the name a fix has to insert. Its
    /// shape is decided by `code`.
    pub data: Option<Value>,
    /// Severity of this diagnostic. Construct warnings via
    /// `Error::warning_from_msg_srcs`; the other constructors produce
    /// `Severity::Error`.
    pub severity: Severity,
}

impl Error {
    /// A diagnostic of error severity carrying the given message, with no source location.
    pub fn from_msg(msg: String) -> Error {
        Error {
            msg,
            srcs: vec![],
            code: None,
            data: None,
            severity: Severity::Error,
        }
    }

    /// A diagnostic of error severity carrying the given message, attached to each of the given
    /// locations that is present, with no description above the quoted source.
    pub fn from_msg_srcs(msg: String, srcs: &[&Option<Span>]) -> Error {
        Error {
            msg,
            srcs: srcs
                .iter()
                .filter_map(|x| x.as_ref().map(|x| (String::default(), (*x).clone())))
                .collect(),
            code: None,
            data: None,
            severity: Severity::Error,
        }
    }

    /// Build a warning-severity diagnostic.
    pub fn warning_from_msg_srcs(msg: String, srcs: &[&Option<Span>]) -> Error {
        let mut err = Error::from_msg_srcs(msg, srcs);
        err.severity = Severity::Warning;
        err
    }

    /// Attaches one more source location to this diagnostic, shown after the ones already
    /// attached.
    ///
    /// # Arguments
    /// * `src_desc` — the line printed above the quoted source, telling the reader what the
    ///   location is to the diagnostic, such as "The value is defined at:".
    pub fn add_src(&mut self, src_desc: String, src: Span) {
        self.srcs.push((src_desc, src));
    }

    /// Attaches several described source locations at once, in the given order, after the ones
    /// already attached.
    pub fn add_srcs(&mut self, mut desc_srcs: Vec<(String, Span)>) {
        self.srcs.append(&mut desc_srcs);
    }

    /// Renders this diagnostic as the compiler prints it: the severity label, the message, and
    /// then each attached location as its description followed by the quoted source with the
    /// span underlined in the severity's color.
    pub fn to_string(&self) -> String {
        let mut str = String::default();
        let (label, underline_color) = self.severity.label_and_underline_color();
        str += &label;
        str += ": ";
        str += &self.msg;
        str += "\n";
        for (src_desc, src) in &self.srcs {
            if src_desc.len() > 0 {
                str += "\n";
                str += src_desc;
                str += "\n";
            }
            str += "\n";
            str += &src.to_string(underline_color);
        }
        str
    }
}

/// Panics with `msg`, having installed a panic hook that prints the message alone, so that the
/// thread name, the panic location and the backtrace note stay out of the compiler's output.
fn panic_notrace(msg: &str) -> ! {
    // Default panic hook shows message such as "thread 'main' panicked at " or "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace".
    // We replace it to empty.
    panic::set_hook(Box::new(move |info| {
        let msg = any_to_string(info.payload());
        eprintln!("{}", msg);
    }));
    panic!("{}", msg);
}

/// The message a panic payload carries. A payload of a type other than `String` or `&str` gives
/// "(unknown error)", since the panic's own message is out of reach then.
pub fn any_to_string(any: &dyn Any) -> String {
    if let Some(s) = any.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = any.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "(unknown error)".to_string()
    }
}

/// Ends the process, printing `msg` in the form a reported error takes.
pub fn panic_with_msg(msg: &str) -> ! {
    let errs = Errors::from_msg(msg.to_string());
    panic_notrace(&errs.to_string())
}

/// The value of a successful `res`. A failed one ends the process with its diagnostics printed.
pub fn panic_if_err<T>(res: Result<T, Errors>) -> T {
    res.unwrap_or_else(|errs| panic_notrace(&errs.to_string()))
}

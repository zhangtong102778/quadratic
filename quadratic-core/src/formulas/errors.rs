//! Error reporting functionality for compilation and runtime.

use std::borrow::Cow;
use std::error::Error;
use std::fmt;

use super::Span;

/// Error message and accompanying span.
#[derive(Debug, Clone)]
pub struct FormulaError {
    /// Location of the source code where the error occurred (if any).
    pub span: Option<Span>,
    /// Type of error.
    pub msg: FormulaErrorMsg,
}
impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(f, "column {} to {}: {}", span.start, span.end, self.msg),
            None => write!(f, "{}", self.msg),
        }
    }
}
impl Error for FormulaError {}
impl FormulaError {
    /// Attaches a span to this FormulaError, if it does not already have one.
    pub fn with_span(mut self, span: impl Into<Span>) -> Self {
        if self.span.is_none() {
            self.span = Some(span.into());
        }
        self
    }
}

/// Information about the type of error that occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormulaErrorMsg {
    // Miscellaneous errors
    Unimplemented,
    UnknownError,
    InternalError(Cow<'static, str>),

    // Compile errors
    Unterminated(&'static str),
    Expected {
        expected: Cow<'static, str>,
        got: Option<Cow<'static, str>>,
    },
    ArraySizeMismatch {
        expected: (usize, usize),
        got: (usize, usize),
    },
    NonRectangularArray,
    BadArgumentCount,
    BadFunctionName,
    BadCellReference,
    BadNumber,

    // Runtime errors
    CircularReference,
    Overflow,
    DivideByZero,
    NegativeExponent,
    IndexOutOfBounds,
}
impl fmt::Display for FormulaErrorMsg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unimplemented => {
                write!(f, "This feature is unimplemented")
            }
            Self::UnknownError => {
                write!(f, "(unknown error)")
            }
            Self::InternalError(s) => {
                write!(f, "Internal error: {s}\nThis is a bug in Quadratic, not your formula. Please report this to us!")
            }

            Self::Unterminated(s) => {
                write!(f, "This {s} never ends")
            }
            Self::Expected { expected, got } => match got {
                Some(got) => write!(f, "Expected {expected}, got {got}"),
                None => write!(f, "Expected {expected}"),
            },
            Self::ArraySizeMismatch { expected, got } => {
                write!(f, "Array size mismatch: expected {expected:?}, got {got:?}")
            }
            Self::NonRectangularArray => {
                write!(f, "Array must be rectangular")
            }
            Self::BadArgumentCount => {
                // TODO: give a nicer error message that says what the arguments
                // should be
                write!(f, "Bad argument count")
            }
            Self::BadFunctionName => {
                write!(f, "There is no function with this name")
            }
            Self::BadCellReference => {
                write!(f, "Bad cell reference")
            }
            Self::BadNumber => {
                write!(f, "Bad numeric literal")
            }

            Self::CircularReference => {
                write!(f, "Circular reference")
            }
            Self::Overflow => {
                write!(f, "Numeric overflow")
            }
            Self::DivideByZero => {
                write!(f, "Divide by zero")
            }
            Self::NegativeExponent => {
                write!(f, "Negative exponent")
            }
            Self::IndexOutOfBounds => {
                write!(f, "Index out of bounds")
            }
        }
    }
}
impl FormulaErrorMsg {
    /// Attaches a span to this error message, returning a FormulaError.
    pub fn with_span(self, span: impl Into<Span>) -> FormulaError {
        FormulaError {
            span: Some(span.into()),
            msg: self,
        }
    }
    /// Returns a FormulaError from this error message, without a span.
    pub const fn without_span(self) -> FormulaError {
        FormulaError {
            span: None,
            msg: self,
        }
    }
}

impl<T: Into<FormulaErrorMsg>> From<T> for FormulaError {
    fn from(msg: T) -> Self {
        msg.into().without_span()
    }
}

/// Handles internal errors. Panics in debug mode for the stack trace, but
/// returns a nice error message in release mode or on web.
///
/// Prefer internal_error!(); be careful not to call this and then throw away
/// the error it returns, because in debug mode in Rust code it will still
/// panic. For example, use `.ok_or_else(|| internal_error_value!(...))` rather
/// than `.ok_or(internal_error_value!(...))`.
macro_rules! internal_error_value {
    // Don't allocate a new String for &'static str literals.
    ( $msg:expr ) => {{
        // Panic in a debug build (for stack trace).
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        #[allow(unused)]
        let ret: crate::formulas::FormulaError = panic!("{}", $msg);
        // Give nice error message for user in release build.
        #[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
        #[allow(unused)]
        let ret: crate::formulas::FormulaError = crate::formulas::FormulaErrorMsg::InternalError(
            std::borrow::Cow::Borrowed($msg),
        )
        .without_span();
        #[allow(unreachable_code)]
        ret
    }};
    // Automatically format!() arguments.
    ( $( $args:expr ),+ $(,)? ) => {{
        // Panic in a debug build (for stack trace).
        #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
        #[allow(unused)]
        let ret: crate::formulas::FormulaError = panic!($( $args ),+);
        // Give nice error message for user in release build.
        #[cfg(not(all(debug_assertions, not(target_arch = "wasm32"))))]
        #[allow(unused)]
        let ret: crate::formulas::FormulaError =
            crate::formulas::FormulaErrorMsg::InternalError(format!($( $args ),+).into()).without_span();
        #[allow(unreachable_code)]
        ret
    }};
}

/// Emits an internal error. Panics in debug mode for the stack trace, but
/// returns a nice error message in release mode or on web.
///
/// Note that this macro actually returns the error from the caller; it does not
/// just provide the value.
macro_rules! internal_error {
    ( $( $args:expr ),+ $(,)? ) => {
        return Err(internal_error_value!($( $args ),+))
    };
}

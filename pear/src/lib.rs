//! An error-handling library.
//!
//! The primary type of this library is `Error`. It contains no boxed objects, 
//! it's just simple old data. It contains three main sections of information: 
//!
//! 1. The _message_, a string summary of the error. If this is longer than 60 
//!    visible characters, `Error` will automatically word-wrap it. 
//! 2. The _fields_, which are (name, value) tuples of contextual information.
//! 3. The _causes_, which is a `Vec<Error>`. Unlike most other error-handling 
//!    libraries, this library can handle errors with more than one cause. 
//!
//! When a `Field` is constructed, or an underlying error is converted into a 
//! `crate::Error`, the `std::fmt` API is used to convert it into a `String`.
//!
//! Set `RUST_BACKTRACE` to a non-`"0"` value to enable backtrace capturing. 

extern crate backtrace;
extern crate map_vec;
extern crate textwrap;
extern crate unicode_width;
extern crate ansi_parser;

/// `std::fmt::Display` implementation.
mod display;

/// Backtrace utilities.
mod backtrace_util;

/// Eager conversion from `impl Debug` into `String`. 
mod pre_debug;

use crate::{
    pre_debug::PreDebug,
    backtrace_util::capture_backtrace_if_enabled,
};
use std::{
    borrow::Borrow,
    iter,
    fmt::Debug,
};
use backtrace::Backtrace;
use map_vec::Map as VecMap;

/// An error. 
///
/// This is the primary type of this library. It contains no boxed objects, 
/// it's just simple old data. It contains three main sections of information: 
///
/// 1. The _message_, a string summary of the error. If this is longer than 60 
///    visible characters, `Error` will automatically word-wrap it. 
/// 2. The _fields_, which are (name, value) tuples of contextual information.
/// 3. The _causes_, which is a `Vec<Error>`. Unlike most other error-handling 
///    libraries, this library can handle errors with more than one cause. 
///
/// When a `Field` is constructed, or an underlying error is converted into a 
/// `crate::Error`, the `std::fmt` API is used to convert it into a `String`.
///
/// Set `RUST_BACKTRACE` to a non-`"0"` value to enable backtrace capturing. 
/// If `self` is formatted in alternative display mode (`{:#}`) then the 
/// `Backtrace` will format with more verbose information.  
#[derive(Clone)]
pub struct Error(Box<ErrorInner>);

/// Alias for `Result<I, crate::Error>`.
pub type Result<I> = std::result::Result<I, Error>;

#[derive(Debug, Clone)]
struct ErrorInner {
    message: String,
    fields: VecMap<String, PreDebug>,
    backtrace: Option<Backtrace>,
    causes: Vec<Error>,
    wrap_enabled: bool,
}

/// A captured (name, value) tuple of `Error` contextual information. 
#[derive(Debug, Clone)]
pub struct Field(String, PreDebug);

impl Error {
    /// Construct a new `Error` from message, fields, and causes. 
    pub fn new<M, F, C>(message: M, fields: F, causes: C) -> Self
    where
        M: Into<String>,
        F: IntoIterator<Item=Field>,
        C: IntoCauses,
    {
        Error(Box::new(ErrorInner {
            message: message.into(),
            fields: fields.into_iter().map(|Field(k, v)| (k, v)).collect(),
            backtrace: capture_backtrace_if_enabled(),
            causes: causes.into_causes(),
            wrap_enabled: true,
        }))
    }

    /// Convert a `std::error::Error` into a `crate::Error`.
    ///
    /// This will crawl the error's `.source()` chain and convert it 
    /// into a chain of single-cause `Error` by using `.to_string()` for
    /// the messages. 
    pub fn from_std(error: &dyn std::error::Error) -> Self
    {
        let mut head = Error::new(error.to_string(), iter::empty(), ());
        *head.wrap_enabled_mut() = false;
        let mut tail = &mut head;
        let mut curr_std = error.source();

        while let Some(error) = curr_std {
            let mut cause = Error::new(error.to_string(), iter::empty(), ());
            *cause.wrap_enabled_mut() = false;
            *tail.causes_mut() = vec![cause];
            tail = &mut tail.causes_mut()[0];
            curr_std = error.source();
        }

        head
    }

    /// Get `self`'s message. 
    pub fn message(&self) -> &str { &self.0.message }

    /// Get `self`'s message, mutably.
    pub fn message_mut(&mut self) -> &mut String { &mut self.0.message }

    /// Get a field of `self` by name. 
    pub fn get_field<'s, K>(&'s self, key: &K) -> Option<impl Debug + 's>
    where
        String: Borrow<K>,
        K: Eq,
    {
        self.0.fields.get(key)
    }

    /// Insert a field into `self`, overriding if already present. 
    pub fn put_field(&mut self, field: Field) {
        let Field(k, v) = field;
        self.0.fields.insert(k, v);
    }

    /// Remove a field of `self` by name. 
    pub fn remove_field<K>(&mut self, key: &K)
    where
        String: Borrow<K>,
        K: Eq,
    {
        self.0.fields.remove(key);
    }

    /// Remove all fields of `self`.
    pub fn clear_fields(&mut self) {
        self.0.fields.clear();
    }

    /// Iterate over all fields of `self`.
    pub fn fields<'s>(&'s self) -> impl Iterator<Item = (&'s str, impl Debug + 's)> + 's {
        self.0.fields.iter()
            .map(|(k, v)| (k.as_str(), v))
    }

    /// Get the number of fields in `self`. 
    pub fn num_fields(&self) -> usize {
        self.0.fields.len()
    }

    /// Get the captured `Backtrace` of `self`, if one was captured. 
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace.as_ref()
    }

    /// Mutably access the optional captured `Backtrace` of `self`.
    pub fn backtrace_mut(&mut self) -> &mut Option<Backtrace> {
        &mut self.0.backtrace
    }

    /// Get the list of causes of `self`.
    pub fn causes(&self) -> &[Error] {
        self.0.causes.as_slice()
    }

    /// Mutably access the list of causes of `self`.
    pub fn causes_mut(&mut self) -> &mut Vec<Error> {
        &mut self.0.causes
    }

    /// Mutably access whether message word-wrapping is enabled.
    pub fn wrap_enabled_mut(&mut self) -> &mut bool {
        &mut self.0.wrap_enabled
    }
}

impl Field {
    /// Construct a new `Field` from key and value. 
    pub fn new<K, V>(key: K, val: V) -> Self
    where
        K: Into<String>,
        V: Debug,
    {
        Field(key.into(), PreDebug::new(val))
    }
}

/// Types which can represent an `Error` cause or list of causes. 
///
/// This is notably implemented for:
/// - `()` (representing no cause)
/// - `Option<Error>`
/// - `Error`
/// - `Vec<Error>` 
pub trait IntoCauses: Sized {
    fn into_causes(self) -> Vec<Error>;

    /// Wrap `self` in `parent`, so that `self` becomes the causes of `parent`. 
    fn wrap(self, mut parent: Error) -> Error {
        parent.causes_mut().extend(self.into_causes());
        parent
    }
}

impl IntoCauses for () {
    fn into_causes(self) -> Vec<Error> {
        vec![]
    }
}

impl IntoCauses for Option<Error> {
    fn into_causes(self) -> Vec<Error> {
        match self {
            Some(error) => vec![error],
            None => vec![],
        }
    }
}

impl IntoCauses for Error {
    fn into_causes(self) -> Vec<Error> {
        vec![self]
    }
}

impl IntoCauses for Vec<Error> {
    fn into_causes(self) -> Vec<Error> {
        self
    }
}

/// Extension methods to `Result<_, impl IntoCauses>`. 
pub trait ResultExt: Sized {
    type Item;

    /// If `self` is the `Err` variant, wrap the underlying error in `parent`, 
    /// so that the underlying error becomes the causes of `parent`.
    ///
    /// `parent` is actually a function which is used to construct the parent 
    /// `Error`, if `self` is the `Err` variant. 
    fn wrap_err<F>(self, parent: F) -> std::result::Result<Self::Item, Error>
    where
        F: FnOnce() -> Error;

    /// If `self` is the `Err` variant, extend a `Vec<Error>` with the 
    /// underlying error list. 
    fn push_err(self, vec: &mut Vec<Error>);

    /// If `self` is the `Err` variant, extend the underlying error list by 
    /// taking the contents of a `Vec<Error>`. 
    ///
    /// In the case that `self` is the `Err` variant, `vec` will be emptied 
    /// by the time this methods returns. 
    fn pull_err(self, vec: &mut Vec<Error>) -> std::result::Result<Self::Item, Vec<Error>>;
}

impl<I, E> ResultExt for std::result::Result<I, E>
where
    E: IntoCauses,
{
    type Item = I;

    fn wrap_err<F>(self, parent: F) -> std::result::Result<Self::Item, Error>
    where
        F: FnOnce() -> Error
    {
        self.map_err(move |causes| {
            causes.wrap(parent())
        })
    }

    fn push_err(self, vec: &mut Vec<Error>) {
        if let Err(causes) = self {
            vec.extend(causes.into_causes());
        }
    }

    fn pull_err(self, vec: &mut Vec<Error>) -> std::result::Result<Self::Item, Vec<Error>> {
        self.map_err(move |causes| {
            let mut causes = causes.into_causes();
            causes.extend(vec.drain(..));
            causes
        })
    }
}

impl<E: std::error::Error> From<E> for Error {
    fn from(e: E) -> Self {
        Error::from_std(&e)
    }
}

/// Construct an `Error` from a field list, followed by the standard string 
/// interpolation syntax for the message. 
///
/// In addition to the provided fields, this also automatically populates the
/// `"rust_module"` and `"rust_line"` fields using the `module_path!()` and 
/// `line!()` macros. 
///
/// ```
/// use pear::pear;
/// 
/// let hello = "hello world";
/// let error = pear!({
///     hello = hello,
///     one_fourty_four = 12 * 12,
/// }, "I am an error, {:?} is a tuple", (1, 2));
///
/// println!("{}", error);
///
/// // output looks like: 
/// //
/// // [ error ]
/// //     I am an error, (1, 2) is a tuple
/// //     - hello = "hello world"
/// //     - one_fourty_four = 144
/// //     - rust_module = "example"
/// //     - rust_line = 20
/// // [ end of error ]
/// ```
#[macro_export]
macro_rules! pear {
    (
        { $($key:ident = $val:expr),* $(,)? },
        $($fmt:tt)*
    )=>{{
        let mut error = $crate::Error::new(
            ::std::format!($($fmt)*),
            ::std::iter::empty(),
            (),
        );
        $(
            error.put_field($crate::Field::new(
                stringify!($key),
                &$val,
            ));
        )*
        error.put_field($crate::Field::new(
            "rust_module", ::std::module_path!()
        ));
        error.put_field($crate::Field::new(
            "rust_line", ::std::line!()
        ));
        error
    }};
}

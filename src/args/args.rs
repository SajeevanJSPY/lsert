// Main Source: https://github.com/clap-rs/clap/tree/master/clap_lex
use crate::args::osstr_ext;
pub use osstr_ext::OsStrExt;
use std::ffi::{OsStr, OsString};
pub use std::io::SeekFrom;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct RawArgs {
    items: Vec<OsString>,
}

impl RawArgs {
    pub fn new(iter: impl IntoIterator<Item = impl Into<std::ffi::OsString>>) -> Self {
        let iter = iter.into_iter();
        Self::from(iter)
    }

    pub fn cursor(&self) -> ArgCursor {
        ArgCursor::new()
    }

    pub fn next(&self, cursor: &mut ArgCursor) -> Option<ParsedArg<'_>> {
        self.next_os(cursor).map(ParsedArg::new)
    }

    fn next_os(&self, cursor: &mut ArgCursor) -> Option<&OsStr> {
        let next = self.items.get(cursor.cursor).map(|s| s.as_os_str());
        cursor.cursor = cursor.cursor.saturating_add(1);
        next
    }
}

impl<I, T> From<I> for RawArgs
where
    I: Iterator<Item = T>,
    T: Into<OsString>,
{
    fn from(val: I) -> Self {
        Self {
            items: val.map(|x| x.into()).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArgCursor {
    cursor: usize,
}

impl ArgCursor {
    fn new() -> Self {
        Self { cursor: 0 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParsedArg<'s> {
    inner: &'s OsStr,
}

impl<'s> ParsedArg<'s> {
    fn new(inner: &'s OsStr) -> Self {
        Self { inner }
    }

    pub fn is_stdio(&self) -> bool {
        self.inner == "-"
    }

    pub fn is_escape(&self) -> bool {
        self.inner == "--"
    }

    pub fn to_long(&self) -> Option<(Result<&str, &OsStr>, Option<&OsStr>)> {
        let raw = self.inner;
        let remainder = raw.strip_prefix("--")?;
        if remainder.is_empty() {
            debug_assert!(self.is_escape());
            return None;
        }

        let (flag, value) = if let Some((p0, p1)) = remainder.split_once("=") {
            (p0, Some(p1))
        } else {
            (remainder, None)
        };
        let flag = flag.to_str().ok_or(flag);
        Some((flag, value))
    }

    pub fn to_short(&self) -> Option<ShortFlags<'_>> {
        if let Some(remainder_os) = self.inner.strip_prefix("-") {
            if remainder_os.starts_with("-") {
                None
            } else if remainder_os.is_empty() {
                debug_assert!(self.is_stdio());
                None
            } else {
                Some(ShortFlags::new(remainder_os))
            }
        } else {
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct ShortFlags<'s> {
    inner: &'s OsStr,
    utf8_prefix: std::str::CharIndices<'s>,
    invalid_suffix: Option<&'s OsStr>,
}

impl<'s> ShortFlags<'s> {
    fn new(inner: &'s OsStr) -> Self {
        let (utf8_prefix, invalid_suffix) = split_nonutf8_once(inner);
        let utf8_prefix = utf8_prefix.char_indices();
        Self {
            inner,
            utf8_prefix,
            invalid_suffix,
        }
    }

    pub fn next_flag(&mut self) -> Option<Result<char, &'s OsStr>> {
        if let Some((_, flag)) = self.utf8_prefix.next() {
            return Some(Ok(flag));
        }

        if let Some(suffix) = self.invalid_suffix {
            self.invalid_suffix = None;
            return Some(Err(suffix));
        }

        None
    }
}

impl<'s> Iterator for ShortFlags<'s> {
    type Item = Result<char, &'s OsStr>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_flag()
    }
}

fn split_nonutf8_once(b: &OsStr) -> (&str, Option<&OsStr>) {
    match b.try_str() {
        Ok(s) => (s, None),
        Err(err) => {
            // SAFETY: `char_indices` ensures `index` is at a valid UTF-8 boundary
            let (valid, after_valid) = unsafe { osstr_ext::split_at(b, err.valid_up_to()) };
            let valid = valid.try_str().unwrap();
            (valid, Some(after_valid))
        }
    }
}

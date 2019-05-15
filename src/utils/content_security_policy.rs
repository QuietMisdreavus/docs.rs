use std::borrow::Cow;
use std::fmt;
use std::str::from_utf8;

use iron::error::HttpError;
use iron::headers::{Header, HeaderFormat};

/// Defines the Content Security Policy used for all pages on docs.rs.
// FIXME: integrate a crate like `content-security-policy` if/when it gets the ability to serialize
// policies back out
#[derive(Clone, Debug)]
pub struct CspHeader(pub Vec<Cow<'static, str>>);

impl Header for CspHeader {
    fn header_name() -> &'static str {
        "Content-Security-Policy"
    }

    fn parse_header(raw: &[Vec<u8>]) -> Result<Self, HttpError> {
        Ok(CspHeader(try!(raw.iter().map(|s| from_utf8(s).map(|s| Cow::Owned(s.to_string())))
                                    .collect::<Result<Vec<Cow<'static, str>>, _>>())))
    }
}

impl HeaderFormat for CspHeader {
    fn fmt_header(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;

        for h in &self.0 {
            if first {
                first = false;
            } else {
                try!(f.write_str("; "));
            }

            try!(fmt::Display::fmt(h, f));
        }

        Ok(())
    }
}

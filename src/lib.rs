//! Parsing and formatting of C universal character names (UCNs).
//!
//! A UCN is a `\u` or `\U` escape followed by hexadecimal digits,
//! e.g. `\u00e9` or `\U0001F600`. This crate converts between such
//! escape sequences and Unicode scalar values.

use std::fmt;

/// The error type returned when parsing a UCN fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// The input did not start with `\u` or `\U`.
    MissingPrefix,
    /// The number of hexadecimal digits was not exactly 4 for `\u` or 8 for `\U`.
    InvalidLength,
    /// A character other than a hexadecimal digit was found in the digits.
    InvalidHexDigit,
    /// The parsed code point is not a valid Unicode scalar value.
    InvalidScalarValue,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingPrefix => write!(f, "missing \\u or \\U prefix"),
            ParseError::InvalidLength => write!(f, "invalid number of hexadecimal digits"),
            ParseError::InvalidHexDigit => write!(f, "invalid hexadecimal digit"),
            ParseError::InvalidScalarValue => write!(f, "not a valid Unicode scalar value"),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parses a complete string consisting of a single C universal character name.
///
/// The accepted forms are:
///
/// * `\u` followed by exactly four hexadecimal digits.
/// * `\U` followed by exactly eight hexadecimal digits.
///
/// The digits are interpreted as a Unicode code point. Code points that are
/// not valid Unicode scalar values (i.e. surrogates `D800..=DFFF` or values
/// greater than `10FFFF`) are rejected with [`ParseError::InvalidScalarValue`].
///
/// # Examples
///
/// ```
/// use ucn_escape_parser::parse;
///
/// assert_eq!(parse("\\u00e9").unwrap(), 'é');
/// assert_eq!(parse("\\U0001F600").unwrap(), '😀');
/// assert!(parse("\\uD800").is_err());
/// ```
pub fn parse(s: &str) -> Result<char, ParseError> {
    let mut chars = s.chars();
    let prefix = chars.next().ok_or(ParseError::MissingPrefix)?;
    let digits_count = match prefix {
        '\\' => {}
        _ => return Err(ParseError::MissingPrefix),
    };
    let kind = chars.next().ok_or(ParseError::MissingPrefix)?;
    let expected_len = match kind {
        'u' => 4,
        'U' => 8,
        _ => return Err(ParseError::MissingPrefix),
    };
    let rest: String = chars.collect();
    if rest.len() != expected_len {
        return Err(ParseError::InvalidLength);
    }
    let mut value: u32 = 0;
    for c in rest.chars() {
        let digit = c.to_digit(16).ok_or(ParseError::InvalidHexDigit)?;
        value = value * 16 + digit;
    }
    char::from_u32(value).ok_or(ParseError::InvalidScalarValue)
}

/// Formats a Unicode scalar value as a C universal character name.
///
/// The output is `\u` followed by four lowercase hexadecimal digits if the
/// scalar value is in the Basic Multilingual Plane (U+0000..=U+FFFF), and
/// `\U` followed by eight lowercase hexadecimal digits otherwise.
///
/// # Examples
///
/// ```
/// use ucn_escape_parser::format;
///
/// assert_eq!(format('é'), "\\u00e9");
/// assert_eq!(format('😀'), "\\U0001f600");
/// ```
pub fn format(c: char) -> String {
    let cp = c as u32;
    if cp <= 0xFFFF {
        format!("\\u{:04x}", cp)
    } else {
        format!("\\U{:08x}", cp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bmp() {
        assert_eq!(parse("\\u00e9").unwrap(), 'é');
        assert_eq!(parse("\\u0041").unwrap(), 'A');
        assert_eq!(parse("\\u0000").unwrap(), '\0');
    }

    #[test]
    fn parse_supplementary() {
        assert_eq!(parse("\\U0001F600").unwrap(), '😀');
        assert_eq!(parse("\\U0001F4A9").unwrap(), '💩');
        assert_eq!(parse("\\U0010FFFF").unwrap(), '\u{10FFFF}');
    }

    #[test]
    fn parse_missing_prefix() {
        assert_eq!(parse(""), Err(ParseError::MissingPrefix));
        assert_eq!(parse("u00e9"), Err(ParseError::MissingPrefix));
        assert_eq!(parse("\\x00e9"), Err(ParseError::MissingPrefix));
    }

    #[test]
    fn parse_invalid_length() {
        assert_eq!(parse("\\u00e"), Err(ParseError::InvalidLength));
        assert_eq!(parse("\\u00e99"), Err(ParseError::InvalidLength));
        assert_eq!(parse("\\U0001F60"), Err(ParseError::InvalidLength));
        assert_eq!(parse("\\U0001F6000"), Err(ParseError::InvalidLength));
    }

    #[test]
    fn parse_invalid_hex_digit() {
        assert_eq!(parse("\\u00g9"), Err(ParseError::InvalidHexDigit));
        assert_eq!(parse("\\U0001F60Z"), Err(ParseError::InvalidHexDigit));
    }

    #[test]
    fn parse_invalid_scalar_value() {
        assert_eq!(parse("\\uD800"), Err(ParseError::InvalidScalarValue));
        assert_eq!(parse("\\uDFFF"), Err(ParseError::InvalidScalarValue));
        assert_eq!(parse("\\U00110000"), Err(ParseError::InvalidScalarValue));
    }

    #[test]
    fn format_bmp() {
        assert_eq!(format('é'), "\\u00e9");
        assert_eq!(format('A'), "\\u0041");
        assert_eq!(format('\0'), "\\u0000");
        assert_eq!(format('\u{FFFF}'), "\\uffff");
    }

    #[test]
    fn format_supplementary() {
        assert_eq!(format('😀'), "\\U0001f600");
        assert_eq!(format('💩'), "\\U0001f4a9");
        assert_eq!(format('\u{10FFFF}'), "\\U0010ffff");
    }

    #[test]
    fn roundtrip() {
        for &c in &['A', 'é', '😀', '\u{10FFFF}', '\0'] {
            let s = format(c);
            assert_eq!(parse(&s).unwrap(), c);
        }
    }
}

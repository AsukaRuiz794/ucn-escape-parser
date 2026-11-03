# ucn-escape-parser

Parses C universal character names like `\u00e9` and converts them to and from Unicode scalar values.

```rust
use ucn_escape_parser::{parse, format};

assert_eq!(parse("\\u00e9").unwrap(), 'é');
assert_eq!(format('é'), "\\u00e9");
assert_eq!(parse("\\U0001F600").unwrap(), '😀');
assert_eq!(format('😀'), "\\U0001f600");
```

## Why this exists

C source code can spell a character with a universal character name (UCN): `\u` followed by exactly four hex digits, or `\U` followed by exactly eight. This crate provides a tiny, dependency-free way to turn those escape strings into `char` values and back again, which is useful when generating or consuming C literals from Rust.

The main trade-off is strictness. The parser accepts only a complete UCN, not a bare hex number and not a UCN embedded in a larger string. That keeps the API small and the error cases unambiguous. If you need to scan a larger buffer, split it yourself and call `parse` on each candidate.

## The awkward edge

`\uD800` through `\uDFFF` are syntactically valid UCNs but do not denote Unicode scalar values. This crate rejects them with `ParseError::InvalidScalarValue` rather than silently producing a replacement character. Code points above `U+10FFFF` are rejected the same way.

## API

- `parse(s: &str) -> Result<char, ParseError>`
- `format(c: char) -> String`
- `enum ParseError { MissingPrefix, InvalidLength, InvalidHexDigit, InvalidScalarValue }`

The `format` function uses lowercase hexadecimal digits and chooses `\u` for code points in the Basic Multilingual Plane and `\U` otherwise.

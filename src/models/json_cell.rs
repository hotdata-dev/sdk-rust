//! One cell of a query result row, held as JSON text.
//!
//! Hand-written and regen-immune: `scripts/normalize-openapi.py` gives the
//! anonymous row-cell schema the name `JsonCell` in the throwaway spec fed to
//! the generator, and `regenerate.yml` passes that name in `--import-mappings`
//! so the generator emits `Vec<Vec<models::JsonCell>>` for the `rows` fields
//! and never writes this file. See `.openapi-generator-ignore`.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

/// One cell of a query result row, carried as the JSON text the server sent.
///
/// `serde_json::Value` has no arbitrary-precision number: a JSON number that
/// does not fit an `i64`/`u64` is parsed through `f64`, so a `DECIMAL(38,2)`
/// carrying 22 significant digits loses everything past the 17th before a
/// caller ever sees it. Holding the token text decides nothing at parse time —
/// the digits the server wrote are the digits that come back out.
///
/// Serializing re-emits that text verbatim, so a cell round-trips to the
/// server's own bytes: a number stays an unquoted JSON number rather than
/// turning into a string.
///
/// `serde_json` is the only supported format. The underlying `RawValue`
/// captures and re-emits text through a private newtype that other serde
/// formats do not recognise, so deserializing a cell from YAML fails and
/// serializing one to YAML writes the JSON text as a string. Go through
/// [`to_value`](Self::to_value) when a cell has to reach a non-JSON format,
/// accepting the rounding that costs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsonCell(Box<RawValue>);

/// Which of the six JSON types a [`JsonCell`] holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JsonCellKind {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}

impl JsonCell {
    /// Adopt `text` — one complete JSON value — as a cell.
    ///
    /// Errors when `text` is not a single well-formed JSON value. The text is
    /// stored as given and re-emitted byte for byte on serialize.
    pub fn from_json_text(text: String) -> Result<Self, serde_json::Error> {
        RawValue::from_string(text).map(Self)
    }

    /// The JSON null cell.
    pub fn null() -> Self {
        Self::from_json_text("null".to_owned()).expect("`null` is valid JSON")
    }

    /// The cell's JSON text, exactly as it arrived.
    ///
    /// This is the lossless view: for a number it is every digit the server
    /// wrote, whatever its width.
    pub fn as_json_str(&self) -> &str {
        self.0.get()
    }

    /// Which JSON type this cell holds.
    ///
    /// Read off the first byte, which a well-formed JSON value always has and
    /// which is enough to tell the six types apart.
    pub fn kind(&self) -> JsonCellKind {
        match self.as_json_str().trim_start().as_bytes().first() {
            Some(b'n') => JsonCellKind::Null,
            Some(b't') | Some(b'f') => JsonCellKind::Bool,
            Some(b'"') => JsonCellKind::String,
            Some(b'[') => JsonCellKind::Array,
            Some(b'{') => JsonCellKind::Object,
            // A leading digit, '-', or (from a non-conforming producer) nothing
            // at all. Numbers are the only remaining JSON type.
            _ => JsonCellKind::Number,
        }
    }

    /// True when the cell is JSON null.
    pub fn is_null(&self) -> bool {
        self.kind() == JsonCellKind::Null
    }

    /// The contents of a JSON string cell, unescaped. `None` for every other
    /// kind — including a number, whose text is available from
    /// [`as_json_str`](Self::as_json_str).
    ///
    /// Borrows out of the stored text for a string with no escape sequence,
    /// which is the common case; only an escaped string is decoded into an
    /// owned `String`. Reading one string column down a large result
    /// therefore allocates per escaped value rather than per row.
    pub fn as_str(&self) -> Option<Cow<'_, str>> {
        if self.kind() != JsonCellKind::String {
            return None;
        }
        let text = self.as_json_str().trim();
        // `kind` said String, so the text is a quoted JSON string.
        let inner = text
            .strip_prefix('"')
            .and_then(|t| t.strip_suffix('"'))
            .unwrap_or_default();
        if inner.contains('\\') {
            serde_json::from_str::<String>(text).ok().map(Cow::Owned)
        } else {
            Some(Cow::Borrowed(inner))
        }
    }

    /// The elements of a JSON array cell, each still carried as text. `None`
    /// for every other kind.
    ///
    /// Elements keep their own digits: descending into an array does not go
    /// through a value tree.
    pub fn as_array(&self) -> Option<Vec<JsonCell>> {
        match self.kind() {
            JsonCellKind::Array => serde_json::from_str(self.as_json_str()).ok(),
            _ => None,
        }
    }

    /// Parse the cell into a value tree.
    ///
    /// Rounds a number too wide for an `f64` — the loss this type exists to
    /// avoid — so prefer [`as_json_str`](Self::as_json_str) wherever the text
    /// will do, and reach for this only when a caller genuinely needs
    /// `serde_json::Value`.
    ///
    /// # Panics
    ///
    /// If the stored text is not valid JSON, which a cell cannot hold: every
    /// constructor goes through `RawValue`, which validates. A failure here is
    /// a broken invariant, not a bad cell. It panics rather than substituting
    /// `Value::Null`, which would be indistinguishable from a cell that really
    /// is null.
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::from_str(self.as_json_str())
            .expect("a JsonCell holds valid JSON: every constructor validates through RawValue")
    }
}

/// Cells are equal when their JSON text is. Text equality is stricter than
/// value equality — `1.0` and `1.00` are different cells — which is the point:
/// this type's contract is the bytes, and two different texts are two
/// different things to anything reading them.
impl PartialEq for JsonCell {
    fn eq(&self, other: &Self) -> bool {
        self.as_json_str() == other.as_json_str()
    }
}

impl Default for JsonCell {
    fn default() -> Self {
        Self::null()
    }
}

impl std::fmt::Display for JsonCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_json_str())
    }
}

impl From<serde_json::Value> for JsonCell {
    /// Build a cell from a value tree. Whatever precision the tree already
    /// lost stays lost; this is for constructing cells from literals, not for
    /// recovering them.
    fn from(value: serde_json::Value) -> Self {
        Self::from_json_text(value.to_string()).expect("a Value always renders as valid JSON")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// The whole point: a decimal wider than an `f64` keeps every digit
    /// through a deserialize/serialize round-trip.
    #[test]
    fn a_wide_decimal_keeps_every_digit() {
        let body = r#"{"rows":[[99999999999999999999.99]]}"#;
        #[derive(Deserialize, Serialize)]
        struct Body {
            rows: Vec<Vec<JsonCell>>,
        }
        let parsed: Body = serde_json::from_str(body).expect("parses");
        assert_eq!(parsed.rows[0][0].as_json_str(), "99999999999999999999.99");
        assert_eq!(serde_json::to_string(&parsed).unwrap(), body);
    }

    /// A number stays an unquoted JSON number on the way out — the cell is not
    /// a string carrying digits.
    #[test]
    fn a_number_serializes_unquoted() {
        let cell = JsonCell::from_json_text("1234567890123456789012.5".to_owned()).unwrap();
        assert_eq!(
            serde_json::to_string(&cell).unwrap(),
            "1234567890123456789012.5"
        );
    }

    #[test]
    fn kind_covers_every_json_type() {
        let cases = [
            ("null", JsonCellKind::Null),
            ("true", JsonCellKind::Bool),
            ("false", JsonCellKind::Bool),
            ("-1.5e10", JsonCellKind::Number),
            ("0", JsonCellKind::Number),
            (r#""hi""#, JsonCellKind::String),
            ("[1,2]", JsonCellKind::Array),
            (r#"{"a":1}"#, JsonCellKind::Object),
        ];
        for (text, want) in cases {
            let cell = JsonCell::from_json_text(text.to_owned()).unwrap();
            assert_eq!(cell.kind(), want, "kind of {text}");
        }
    }

    #[test]
    fn as_str_unescapes_a_string_and_declines_everything_else() {
        let s = JsonCell::from_json_text(r#""a\"b\nc""#.to_owned()).unwrap();
        assert_eq!(s.as_str().as_deref(), Some("a\"b\nc"));
        let n = JsonCell::from_json_text("12".to_owned()).unwrap();
        assert_eq!(n.as_str(), None);
    }

    /// Only an escaped string allocates; the common case borrows.
    ///
    /// `serde_json::Value::as_str` handed back a borrowed `&str`, so returning
    /// an owned `String` here would add one allocation per row to any caller
    /// reading a string column down a large result.
    #[test]
    fn as_str_borrows_unless_the_text_is_escaped() {
        let plain = JsonCell::from_json_text(r#""hello""#.to_owned()).unwrap();
        assert!(matches!(plain.as_str(), Some(Cow::Borrowed("hello"))));

        let empty = JsonCell::from_json_text(r#""""#.to_owned()).unwrap();
        assert!(matches!(empty.as_str(), Some(Cow::Borrowed(""))));

        let escaped = JsonCell::from_json_text(r#""a\"b""#.to_owned()).unwrap();
        assert!(matches!(escaped.as_str(), Some(Cow::Owned(_))));
        assert_eq!(escaped.as_str().as_deref(), Some("a\"b"));
    }

    /// Descending into an array keeps each element's text, so a wide decimal
    /// nested in a list survives too.
    #[test]
    fn as_array_keeps_element_precision() {
        let cell = JsonCell::from_json_text("[99999999999999999999.99,\"x\"]".to_owned()).unwrap();
        let items = cell.as_array().expect("array");
        assert_eq!(items[0].as_json_str(), "99999999999999999999.99");
        assert_eq!(items[1].as_str().as_deref(), Some("x"));
        assert!(JsonCell::from_json_text("1".to_owned())
            .unwrap()
            .as_array()
            .is_none());
    }

    #[test]
    fn from_value_and_equality_compare_text() {
        assert_eq!(JsonCell::from(json!(1)), JsonCell::from(json!(1)));
        assert_ne!(
            JsonCell::from_json_text("1.0".to_owned()).unwrap(),
            JsonCell::from_json_text("1.00".to_owned()).unwrap()
        );
        assert!(JsonCell::default().is_null());
    }

    #[test]
    fn from_json_text_rejects_text_that_is_not_one_json_value() {
        assert!(JsonCell::from_json_text("1 2".to_owned()).is_err());
        assert!(JsonCell::from_json_text("[".to_owned()).is_err());
    }
}

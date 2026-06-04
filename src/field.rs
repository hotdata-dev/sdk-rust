//! Ergonomic constructors for nullable, optional request fields.
//!
//! Several generated request models (e.g.
//! [`UpdateDatasetRequest`](crate::models::UpdateDatasetRequest)) represent a
//! field that is *both* optional (may be omitted from the request) *and*
//! nullable (may be sent as JSON `null`) as a double option
//! `Option<Option<T>>`, serialized with serde's `double_option`:
//!
//! | Value             | On the wire        | Meaning                          |
//! |-------------------|--------------------|----------------------------------|
//! | `None`            | field omitted      | leave the field unchanged        |
//! | `Some(None)`      | `"field": null`    | explicitly clear / unset / unpin |
//! | `Some(Some(v))`   | `"field": v`       | set the field to `v`             |
//!
//! Writing `Some(Some(x))` / `Some(None)` by hand is easy to get backwards.
//! These helpers name the three intents so call sites read clearly and the
//! double option stays in the generated model unchanged (regeneration-safe):
//!
//! ```
//! use hotdata::field;
//! use hotdata::models::UpdateDatasetRequest;
//!
//! let mut req = UpdateDatasetRequest::new();
//! req.label = field::set("renamed");   // set label to "renamed"
//! req.pinned_version = field::clear();  // unpin (send null)
//! // req.table_name stays `None` -> omitted -> unchanged
//! ```
//!
//! This module is hand-written and regeneration-immune (it never touches the
//! generated models, only constructs their double-option fields).

/// Set a nullable, optional field to a concrete `value` (`Some(Some(value))`).
///
/// Accepts anything that converts into `T`, so `field::set("x")` works for a
/// `String` field.
pub fn set<T, V: Into<T>>(value: V) -> Option<Option<T>> {
    Some(Some(value.into()))
}

/// Explicitly clear a nullable field, sending JSON `null` (`Some(None)`).
///
/// Use this to unset / unpin a value, as opposed to [`unchanged`] which leaves
/// the field untouched.
pub fn clear<T>() -> Option<Option<T>> {
    Some(None)
}

/// Leave a nullable, optional field unchanged by omitting it (`None`).
///
/// Equivalent to the field's default; provided for symmetry and readability at
/// call sites that set some fields and deliberately skip others.
pub fn unchanged<T>() -> Option<Option<T>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_wraps_in_double_some() {
        let f: Option<Option<String>> = set("x");
        assert_eq!(f, Some(Some("x".to_owned())));
    }

    #[test]
    fn set_accepts_into() {
        // &str -> String via Into<T>.
        let f: Option<Option<String>> = set(String::from("y"));
        assert_eq!(f, Some(Some("y".to_owned())));
        let n: Option<Option<i32>> = set(7i32);
        assert_eq!(n, Some(Some(7)));
    }

    #[test]
    fn clear_is_some_none() {
        let f: Option<Option<i32>> = clear();
        assert_eq!(f, Some(None));
    }

    #[test]
    fn unchanged_is_none() {
        let f: Option<Option<String>> = unchanged();
        assert_eq!(f, None);
    }
}

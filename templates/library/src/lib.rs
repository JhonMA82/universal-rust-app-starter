//! Example public API. Replace the demonstration domain with your own.

/// Normalize a display label by trimming whitespace.
///
/// # Examples
/// ```
/// assert_eq!("  hello  ".trim(), "hello");
/// ```
pub fn normalized_label(input: &str) -> String {
    input.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_spaces() {
        assert_eq!(normalized_label("  Rust  "), "Rust");
    }

    #[test]
    fn preserves_unicode() {
        assert_eq!(normalized_label("  ñ  "), "ñ");
    }
}

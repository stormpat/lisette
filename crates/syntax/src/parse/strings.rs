/// Normalise CRLF to LF; pass other bytes through unchanged.
pub(crate) fn cook_string_contents(content: &str) -> String {
    if !content.contains('\r') {
        return content.to_string();
    }

    let bytes = content.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(content.len());
    let mut i = 0;
    let mut copy_start = 0;

    while i < bytes.len() {
        if bytes[i] == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
            out.extend_from_slice(&bytes[copy_start..i]);
            out.push(b'\n');
            i += 2;
            copy_start = i;
            continue;
        }
        i += 1;
    }

    out.extend_from_slice(&bytes[copy_start..]);

    // SAFETY: input was valid UTF-8 and we only inserted ASCII LF or skipped
    // ASCII bytes, so the remaining content is still valid UTF-8.
    unsafe { String::from_utf8_unchecked(out) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_single_line() {
        assert_eq!(cook_string_contents("hello"), "hello");
    }

    #[test]
    fn preserves_embedded_newline() {
        assert_eq!(cook_string_contents("a\nb"), "a\nb");
    }

    #[test]
    fn normalises_crlf_to_lf() {
        assert_eq!(cook_string_contents("a\r\nb"), "a\nb");
    }

    #[test]
    fn preserves_lone_cr() {
        assert_eq!(cook_string_contents("a\rb"), "a\rb");
    }

    #[test]
    fn passes_other_escapes_through() {
        assert_eq!(cook_string_contents("a\\nb"), "a\\nb");
        assert_eq!(cook_string_contents("a\\\\b"), "a\\\\b");
    }

    #[test]
    fn preserves_multibyte_utf8() {
        assert_eq!(cook_string_contents("héllo"), "héllo");
        assert_eq!(cook_string_contents("a\r\né"), "a\né");
    }
}

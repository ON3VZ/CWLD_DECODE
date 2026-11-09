//! Tiny reader for the flat XML that QRZ.com sends. No general XML parser is needed: every field is a
//! simple `<tag>text</tag>` element.

/// First text content of `<tag>...</tag>`, with the five standard entities decoded.
pub fn tag(xml: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(unescape(xml[start..end].trim()))
}

pub fn unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// Percent-encodes everything outside the unreserved set, so passwords with `;` or `&` survive.
pub fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_tags_and_entities() {
        let x = "<a><fname>Jan &amp; Piet</fname><name>Peeters</name></a>";
        assert_eq!(tag(x, "fname").as_deref(), Some("Jan & Piet"));
        assert_eq!(tag(x, "name").as_deref(), Some("Peeters"));
        assert_eq!(tag(x, "grid"), None);
    }

    #[test]
    fn encodes_reserved_characters() {
        assert_eq!(encode("a;b&c d"), "a%3Bb%26c%20d");
        assert_eq!(encode("Ab-1._~"), "Ab-1._~");
    }
}

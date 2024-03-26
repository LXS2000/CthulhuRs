pub(crate) fn is_char_ascii_whitespace(c: char) -> bool {
    c == '\u{09}' || c == '\u{0A}' || c == '\u{0C}' || c == '\u{0D}' || c == '\u{20}'
}

pub(crate) fn strip_leading_and_trailing_ascii_whitespace(string: &str) -> &str {
    string.trim_matches(is_char_ascii_whitespace)
}

pub(crate) fn collect_a_sequence_of_non_ascii_white_space_code_points(string: &str)
                                                           -> (&str, &str) {
    match string.find(is_char_ascii_whitespace) {
        Some(i) => string.split_at(i),
        None => (string, "")
    }
}

pub(crate) struct SplitAsciiWhitespace<'a>(&'a str);

impl<'a> Iterator for SplitAsciiWhitespace<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        self.0 = self.0.trim_start_matches(is_char_ascii_whitespace);
        let mut s = self.0.splitn(2, is_char_ascii_whitespace);
        let next = s.next().unwrap_or("");
        self.0 = s.next().unwrap_or("");
        match (next.is_empty(), self.0.is_empty()) {
            (true, true) => None,
            (false, _) => Some(next),
            _ => self.next(),
        }
    }
}

pub(crate) fn split_ascii_whitespace(string: &str) -> SplitAsciiWhitespace {
    SplitAsciiWhitespace(string)
}

pub(crate) struct SplitCommas<'a>(&'a str);

impl<'a> Iterator for SplitCommas<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let mut s = self.0.splitn(2, ',');
        let next = s.next().unwrap_or("");
        self.0 = s.next().unwrap_or("");
        match (next.is_empty(), self.0.is_empty()) {
            (true, true) => None,
            (true, false) => Some(""),
            (false, _) => Some(next),
        }
    }
}

pub(crate) fn split_commas(string: &str) -> SplitCommas {
    SplitCommas(string)
}

pub(crate) fn ascii_case_insensitive_match(a: &str, b: &str) -> bool {
    a.to_ascii_lowercase() == b.to_ascii_lowercase()
}

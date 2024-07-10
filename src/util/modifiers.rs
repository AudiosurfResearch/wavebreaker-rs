use regex::Regex;

/// For song titles with modifiers, this function returns a vector of the modifiers, or `None` if no modifiers are found.
///
/// **Example:** "death comes from above \[as-steep]" -> \["steep"]
pub fn parse_from_title(title: &str) -> Option<Vec<&str>> {
    let full_tag_string_regex =
        Regex::new(r"(?:\[as-[a-zA-Z0-9]+\]\s*)+$").expect("Regex should always be valid!");
    let tag_separation_regex =
        Regex::new(r"\[as-([a-zA-Z0-9]+)\]").expect("Regex should always be valid!");

    full_tag_string_regex.find(title).map_or_else(
        || None,
        |tag_string| {
            Some(
                tag_separation_regex
                    .captures_iter(tag_string.as_str())
                    .map(|cap| cap.get(1).map_or("", |m| m.as_str()))
                    .collect(),
            )
        },
    )
}

/// If the song title has modifiers, this function returns a new ``String`` without them.
///
/// **Example:** "death comes from above \[as-steep]" -> "death comes from above"
pub fn remove_from_title(title: &str) -> String {
    let full_tag_string_regex =
        Regex::new(r"(?:\[as-[a-zA-Z0-9]+\]\s*)+$").expect("Regex should always be valid!");

    full_tag_string_regex.find(title).map_or_else(
        || title.to_string(),
        |mat| title[0..mat.start()].trim_end().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mods_from_title() {
        let title = "SUMMER [as-tag1] [as-tag2] [as-steep]";
        let expected_tags = Some(vec!["tag1", "tag2", "steep"]);
        assert_eq!(parse_from_title(title), expected_tags);
    }

    #[test]
    fn mods_from_title_empty() {
        let title = "On Down";
        assert_eq!(parse_from_title(title), None);
    }

    #[test]
    fn remove_mods_from_title() {
        let title = "Future Rewind [as-tag1] [as-tag2] [as-tag3]";
        let expected_result = "Future Rewind";
        assert_eq!(remove_from_title(title), expected_result);
    }

    #[test]
    fn remove_mods_from_title_empty() {
        let title = "マボロシ";
        let expected_result = "マボロシ";
        assert_eq!(remove_from_title(title), expected_result);
    }
}

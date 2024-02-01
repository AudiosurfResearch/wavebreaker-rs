use regex::Regex;

/// For song titles with modifiers, this function returns a vector of the modifiers.
/// 
/// **Example:** "death from above [as-steep]" -> ["steep"]
/// 
/// Returns an empty vector if no modifiers are found.
pub fn modifiers_from_title(title: &str) -> Vec<&str> {
    let full_tag_string_regex = Regex::new(r"(?:\[as-[a-zA-Z0-9]+\]\s*)+$").expect("Regex should always be valid!");
    let tag_separation_regex = Regex::new(r"\[as-([a-zA-Z0-9]+)\]").expect("Regex should always be valid!");

    full_tag_string_regex.find(title).map_or_else(
        std::vec::Vec::new,
        |tag_string| {
            tag_separation_regex
                .captures_iter(tag_string.as_str())
                .map(|cap| cap.get(1).map_or("", |m| m.as_str()))
                .collect()
        },
    )
}

/// If the song title has modifiers, this function returns a new ``String`` without them.
/// 
/// **Example:** "death from above [as-steep]" -> "death from above"
pub fn remove_modifiers_from_title(title: &str) -> String {
    let full_tag_string_regex = Regex::new(r"(?:\[as-[a-zA-Z0-9]+\]\s*)+$").expect("Regex should always be valid!");

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
        let title = "My Title [as-tag1] [as-tag2] [as-steep]";
        let expected_tags = vec!["tag1", "tag2", "steep"];
        assert_eq!(modifiers_from_title(title), expected_tags);
    }

    #[test]
    fn mods_from_title_empty() {
        let title = "My Title";
        let expected_tags: Vec<&str> = vec![];
        assert_eq!(modifiers_from_title(title), expected_tags);
    }

    #[test]
    fn remove_mods_from_title() {
        let title = "My Title [as-tag1] [as-tag2] [as-tag3]";
        let expected_result = "My Title";
        assert_eq!(remove_modifiers_from_title(title), expected_result);
    }

    #[test]
    fn remove_mods_from_title_empty() {
        let title = "My Title";
        let expected_result = "My Title";
        assert_eq!(remove_modifiers_from_title(title), expected_result);
    }
}

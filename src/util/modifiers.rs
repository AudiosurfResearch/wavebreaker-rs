use regex::Regex;

pub fn tags_from_title(title: &str) -> Vec<&str> {
    let full_tag_string_regex = Regex::new(r"(?:\[as-[a-zA-Z0-9]+\]\s*)+$").unwrap();
    let tag_separation_regex = Regex::new(r"\[as-([a-zA-Z0-9]+)\]").unwrap();

    full_tag_string_regex.find(title).map_or_else(
        || vec![],
        |tag_string| {
            tag_separation_regex
                .captures_iter(tag_string.as_str())
                .map(|cap| cap.get(1).map_or("", |m| m.as_str()))
                .collect()
        },
    )
}

pub fn remove_tags_from_title(title: &str) -> String {
    let full_tag_string_regex = Regex::new(r"(?:\[as-[a-zA-Z0-9]+\]\s*)+$").unwrap();

    full_tag_string_regex.find(title).map_or_else(
        || title.to_string(),
        |mat| title[0..mat.start()].trim_end().to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tags_from_title_with_tags() {
        let title = "My Title [as-tag1] [as-tag2] [as-steep]";
        let expected_tags = vec!["tag1", "tag2", "steep"];
        assert_eq!(tags_from_title(title), expected_tags);
    }

    #[test]
    fn test_tags_from_title_without_tags() {
        let title = "My Title";
        let expected_tags: Vec<&str> = vec![];
        assert_eq!(tags_from_title(title), expected_tags);
    }

    #[test]
    fn test_remove_tags_from_title_with_tags() {
        let title = "My Title [as-tag1] [as-tag2] [as-tag3]";
        let expected_result = "My Title";
        assert_eq!(remove_tags_from_title(title), expected_result);
    }

    #[test]
    fn test_remove_tags_from_title_without_tags() {
        let title = "My Title";
        let expected_result = "My Title";
        assert_eq!(remove_tags_from_title(title), expected_result);
    }
}

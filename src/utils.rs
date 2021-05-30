pub fn text_to_lines<'a>(line: &'a str) -> Vec<&'a str> {
    let words: Vec<&'a str> = line.split("\n")
        .into_iter()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();
    words
}

pub fn line_to_words<'a>(line: &'a str, divider: &str) -> Vec<&'a str> {
    let words: Vec<&'a str> = line.split(divider)
        .into_iter()
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .collect();
    words
}
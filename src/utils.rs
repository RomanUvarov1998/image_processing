use std::{vec::IntoIter};


pub struct TextBlocksIter<'text> {
    iter: IntoIter<&'text str>
}

impl<'text> TextBlocksIter<'text> {
    pub fn new(text: &'text str, blocks_separator: &'text str) -> Self {
        let blocks: Vec<&str> = text.split(blocks_separator)
            .into_iter()
            .map(|w| w.trim())
            .filter(|w| !w.is_empty())
            .collect();

        let iter = blocks.into_iter();

        TextBlocksIter { iter }
    }

    pub fn iter(&'text mut self) -> &'text mut IntoIter<&'text str> { &mut self.iter }

    pub fn len(&self) -> usize { self.iter.len() }
}


pub struct LinesIter<'text> {
    iter: IntoIter<&'text str>
}

impl<'text> LinesIter<'text> {
    pub fn new(text: &'text str) -> Self {
        let lines: Vec<&str> = text.split("\n")
            .into_iter()
            .map(|w| w.trim())
            .filter(|w| !w.is_empty())
            .collect();
        let iter = lines.into_iter();
        LinesIter { iter }
    }

    pub fn next_or_empty(&mut self) -> &str {
        self.iter.next().unwrap_or("")
    }

    pub fn all_left(&'text mut self, separate_by_newline: bool) -> String {
        let mut left = String::new();

        if let Some(line) = self.iter.next() {
            left.push_str(line);
        }
        while let Some(line) = self.iter.next() {
            if separate_by_newline{
                left.push_str("\n");
            }
            left.push_str(line);
        }

        left
    }

    pub fn len(&self) -> usize { self.iter.len() }
}


pub struct WordsIter<'text> {
    iter: IntoIter<&'text str>
}

impl<'text> WordsIter<'text> {
    pub fn new(text: &'text str, divider: &str) -> Self {
        let lines: Vec<&str> = text.split(divider)
            .into_iter()
            .map(|w| w.trim())
            .filter(|w| !w.is_empty())
            .collect();
        let iter = lines.into_iter();
        WordsIter { iter }
    }

    pub fn next_or_empty(&mut self) -> &str {
        self.iter.next().unwrap_or("")
    }

    pub fn len(&self) -> usize { self.iter.len() }
}
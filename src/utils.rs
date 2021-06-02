use std::{vec::IntoIter};

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

    pub fn next(&mut self) -> &str {
        self.iter.next().unwrap_or("")
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

    pub fn next(&mut self) -> &str {
        self.iter.next().unwrap_or("")
    }

    pub fn len(&self) -> usize { self.iter.len() }
}
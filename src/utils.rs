use std::{ops::{Add, AddAssign, Neg, Sub, SubAssign}, vec::IntoIter};
use fltk::prelude::WidgetExt;

// ---------------------------------- Text ------------------------------------

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


// ---------------------------------- Geomerty ------------------------------------

#[derive(Clone, Copy, Debug)]
pub struct Pos { pub x: i32, pub y: i32 }

#[allow(unused)]
impl Pos {
	pub fn new(x: i32, y: i32) -> Self {
		Self { x, y }
	}

    pub fn mul_f(&self, val: f32) -> Self {
        Pos::new(
            (self.x as f32 * val) as i32,
            (self.y as f32 * val) as i32,
        )
    }

    pub fn center_of<W: WidgetExt>(wid: &W) -> Self {
        Pos {
            x: wid.x() + wid.w() / 2,
            y: wid.y() + wid.h() / 2
        }
    }
}

impl Sub for Pos {
    type Output = Pos;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Pos {
    fn sub_assign(&mut self, rhs: Self) {
		self.x -= rhs.x;
		self.y -= rhs.y;
    }
}

impl Add for Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
		self.x += rhs.x;
		self.y += rhs.y;
    }
}

impl Neg for Pos {
    type Output = Pos;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Size { pub w: i32, pub h: i32 }

#[allow(unused)]
impl Size {
	pub fn new(w: i32, h: i32) -> Self {
		Self { w, h }
	}

    pub fn of<W: WidgetExt>(wid: &W) -> Self {
		Self { w: wid.w(), h: wid.h() }
	}

    pub fn mul_f(&self, val: f32) -> Self {
        Size::new(
            (self.w as f32 * val) as i32,
            (self.h as f32 * val) as i32,
        )
    }
}

impl AddAssign for Size {
    fn add_assign(&mut self, rhs: Self) {
		self.w += rhs.w;
		self.h += rhs.h;
    }
}


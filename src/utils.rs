use std::{ops::{Add, AddAssign, Neg, Sub, SubAssign}, vec::IntoIter};
use fltk::prelude::WidgetExt;

use crate::img::{Img, PixelPos};

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
            if separate_by_newline {
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

    pub fn next(&mut self) -> Option<&str> {
        self.iter.next()
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

    pub fn size_of<W: WidgetExt>(wid: &W) -> Self {
		Self { x: wid.w(), y: wid.h() }
	}

    pub fn size_of_img(img: &Img) -> Self {
        Self { x: img.w() as i32, y: img.h() as i32 }
    }

    pub fn of<W: WidgetExt>(wid: &W) -> Self {
        Pos { x: wid.x(), y: wid.y() }
    }

    pub fn to_pixel_pos(&self) -> PixelPos {
        assert!(self.x >= 0);
        assert!(self.y >= 0);
        let col = self.x as usize;
        let row = self.y as usize;
        PixelPos::new(row, col)
    }

    pub fn decompose(&self) -> (i32, i32) /* x, y */ {
        (self.x, self.y)
    }

    pub fn mul_f(&self, val: f32) -> Self {
        Pos::new(
            (self.x as f32 * val) as i32,
            (self.y as f32 * val) as i32,
        )
    }

    pub fn div_f(&self, val: f32) -> Self {
        Pos::new(
            (self.x as f32 / val) as i32,
            (self.y as f32 / val) as i32,
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

impl Default for Pos {
    fn default() -> Self {
        Pos::new(0, 0)
    }
}

pub trait Clampable where Self: Copy + Clone + PartialOrd {
    fn clamp_min(&mut self, min_value: Self) {
        if *self < min_value {
            *self = min_value;
        }
    }

    fn clamp_max(&mut self, max_value: Self) {
        if *self > max_value {
            *self = max_value;
        }
    }
}

impl Clampable for f32 {}
impl Clampable for i32 {}


#[derive(Clone, Copy, Debug)]
pub struct DraggableRect { top_left: Pos, bottom_right: Pos }

#[allow(unused)]
impl DraggableRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut rect = DraggableRect { 
            top_left: Pos::new(x, y), 
            bottom_right : Pos::new(x + w, y + h)
        };
        rect.correct_anchors();
        rect
    }

    pub fn tl(&self) -> Pos { self.top_left }
    pub fn br(&self) -> Pos { self.bottom_right }

    pub fn x(&self) -> i32 { self.top_left.x }
    pub fn y(&self) -> i32 { self.top_left.y }
    pub fn w(&self) -> i32 { self.bottom_right.x - self.top_left.x }
    pub fn h(&self) -> i32 { self.bottom_right.y - self.top_left.y }
    pub fn center(&self) -> Pos { (self.top_left + self.bottom_right).div_f(2.0) }

    pub fn move_to_new_top_left(&mut self, new_top_left: Pos) {
        let delta = new_top_left - self.top_left;
        self.move_by_delta(delta);
    }

    pub fn move_by_delta(&mut self, delta: Pos) {
        self.top_left += delta;
        self.bottom_right += delta;
    }

    pub fn drag(&mut self, delta: Pos, drag_pos: DragPos) -> DragPos {
        match drag_pos.x {
            DragPosX::Left => { self.top_left.x += delta.x; },
            DragPosX::Right => { self.bottom_right.x += delta.x; },
            _ => {},
        }

        match drag_pos.y {
            DragPosY::Top => { self.top_left.y += delta.y; },
            DragPosY::Bottom => { self.bottom_right.y += delta.y; },
            _ => {},
        }

        if drag_pos == DragPos::new(DragPosX::Middle, DragPosY::Middle) {
            self.top_left += delta;
            self.bottom_right += delta;
        }

        let mut new_drag_pos = drag_pos;

        if self.top_left.x > self.bottom_right.x { new_drag_pos.mirror_x(); }
        if self.top_left.y > self.bottom_right.y { new_drag_pos.mirror_y(); }

        self.correct_anchors();

        new_drag_pos
    }

    pub fn fit_inside(&mut self, area: RectArea) {
        self.top_left.x.clamp_min(area.x);
        self.top_left.y.clamp_min(area.y);
        self.bottom_right.x.clamp_max(area.right());
        self.bottom_right.y.clamp_max(area.bottom());
        
        self.correct_anchors();
    }

    fn correct_anchors(&mut self) {
        if self.top_left.x > self.bottom_right.x {
            std::mem::swap(&mut self.top_left.x, &mut self.bottom_right.x);
        }
        
        if self.top_left.y > self.bottom_right.y {
            std::mem::swap(&mut self.top_left.y, &mut self.bottom_right.y);
        }
    }
}


#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum DragPosX { Left = 0, Middle = 1, Right = 2 }

#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum DragPosY { Top = 0, Middle = 1, Bottom = 2 }

#[derive(Clone, Copy, Debug)]
pub struct DragPos { pub x: DragPosX, pub y: DragPosY }

impl PartialEq for DragPos {
    fn eq(&self, other: &Self) -> bool {
        self.x as i32 == other.x as i32 && self.y as i32 == other.y as i32
    }
}

impl DragPos {
    pub fn from(x: i32, y: i32) -> Self {
        let px = match x {
            0 => DragPosX::Left,
            1 => DragPosX::Middle,
            2 => DragPosX::Right,
            _ => unreachable!()
        };
        
        let py = match y {
            0 => DragPosY::Top,
            1 => DragPosY::Middle,
            2 => DragPosY::Bottom,
            _ => unreachable!()
        };

        DragPos::new(px, py)
    }

    pub fn new(x: DragPosX, y: DragPosY) -> Self {
        DragPos { x, y }
    }

    pub fn mirror_x(&mut self) {
        self.x = match self.x {
            DragPosX::Left => DragPosX::Right,
            DragPosX::Middle => self.x,
            DragPosX::Right => DragPosX::Left,
        }
    }

    pub fn mirror_y(&mut self) {
        self.y = match self.y {
            DragPosY::Top => DragPosY::Bottom,
            DragPosY::Middle => self.y,
            DragPosY::Bottom => DragPosY::Top,
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct ScalableRect { top_left: Pos, size: Pos, scale: f32 }

impl ScalableRect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        assert!(w >= 0);
        assert!(h >= 0);

        ScalableRect { 
            top_left: Pos::new(x, y),
            size: Pos::new(w, h),
            scale: 1_f32 
        }
    }

    pub fn tl(&self) -> Pos { self.top_left }

    pub fn actual_w(&self) -> i32 { self.size.x }
    pub fn actual_h(&self) -> i32 { self.size.y }

    pub fn scaled_w(&self) -> i32 { (self.size.x as f32 * self.scale) as i32 }
    pub fn scaled_h(&self) -> i32 { (self.size.y as f32 * self.scale) as i32 }
    pub fn scaled_br(&self) -> Pos { self.top_left + self.size.mul_f(self.scale) }

    pub fn area_scaled(&self) -> RectArea { 
        RectArea::new(
            self.top_left.x, self.top_left.y, 
            self.scaled_w(), self.scaled_h())
    }

    pub fn self_to_pixel(&self, pos: Pos) -> Pos {
        (pos - self.top_left).div_f(self.scale)
    }


    pub fn stretch_self_to_area(&mut self, area: RectArea) {
        // ------------------------ fit by scale -------------------------------
        self.scale = self.get_scale_to_fit(area.size());

        // ------------------------ fit by position -------------------------------
        self.top_left = {
            let new_top_left_x = 
                if self.scaled_w() < area.w() {
                    area.x() + area.w() / 2 - self.scaled_w() / 2
                } else {
                    area.x()
                };

            let new_top_left_y = 
                if self.scaled_h() < area.h() {
                    area.y() + area.h() / 2 - self.scaled_h() / 2
                } else {
                    area.y()
                };

            Pos::new(new_top_left_x, new_top_left_y)
        };
    }

    pub fn zoom_area(&mut self, area: RectArea, view_center: Pos) {
        assert!(area.x >= self.tl().x);
        assert!(area.y >= self.tl().y);
        assert!(area.right() <=self.scaled_br().x);
        assert!(area.bottom() <= self.scaled_br().y);

        // move area center to view_center
        self.translate(view_center - area.center());

        // scale and keep view_center position
        let delta = {
            let new_scale = {
                let ratio_w = self.scaled_w() as f32 / area.w() as f32;
                let ratio_h = self.scaled_h() as f32 / area.h() as f32;
                if ratio_w < ratio_h { ratio_w } else { ratio_h }
            };
            new_scale - self.scale
        };

        self.scale_keep_anchor_pos(delta, view_center);
    }

    pub fn translate(&mut self, delta: Pos) {
        self.top_left += delta;
    }

    pub fn scale_keep_anchor_pos(&mut self, delta: f32, anchor: Pos) {
        let relative = anchor - self.tl();

        self.top_left -= relative.mul_f(delta / self.scale);

        self.scale += delta;
    }

    pub fn fit_scale(&mut self, area_size: Pos) {
        let scale_to_fit = self.get_scale_to_fit(area_size);
        self.scale.clamp_min(scale_to_fit);
    }

    pub fn fit_pos(&mut self, area: RectArea) {
        let area_center = area.center();

        self.top_left.x = 
            if self.scaled_w() >= area.w() {
                self.top_left.x.clamp(area.right() - self.scaled_w(), area.x())
            } else {
                area_center.x - self.scaled_w() / 2
            };

        self.top_left.y = 
            if self.scaled_h() >= area.h() {
                self.top_left.y.clamp(area.bottom() - self.scaled_h(), area.y())
            } else {
                area_center.y - self.scaled_h() / 2
            };
    }


    fn get_scale_to_fit(&self, area_size: Pos) -> f32 {
        let ratio_w: f32 = area_size.x as f32 / self.actual_w() as f32;
        let ratio_h: f32 = area_size.y as f32 / self.actual_h() as f32;
        if ratio_w < ratio_h { ratio_w } else { ratio_h }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RectArea {
    pub x: i32, 
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[allow(unused)]
impl RectArea {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        assert!(w > 0);
        assert!(h > 0);

        RectArea { x, y, w, h }
    }

    pub fn of_widget<W: WidgetExt>(w: &W) -> Self { RectArea::new(w.x(), w.y(), w.w(), w.h() ) }

    pub fn of_draggable_rect(rect: &DraggableRect) -> Self { 
        RectArea::new(rect.x(), rect.y(), rect.w(), rect.h() ) 
    }

    pub fn with_zero_origin(mut self) -> Self {
        self.x = 0;
        self.y = 0;
        self
    }

    pub fn size(&self) -> Pos { Pos::new(self.w, self.h) }

    pub fn tl(&self) -> Pos { Pos::new(self.x, self.y) }

    pub fn center(&self) -> Pos { Pos::new(self.x + self.w / 2, self.y + self.h / 2) }

    pub fn x(&self) -> i32 { self.x }
    pub fn y(&self) -> i32 { self.y }
    pub fn w(&self) -> i32 { self.w }
    pub fn h(&self) -> i32 { self.h }

    pub fn right(&self) -> i32 { self.x + self.w }
    pub fn bottom(&self) -> i32 { self.y + self.h }
}
use fltk::{group::{self, PackType}, prelude::{GroupExt, WidgetExt}};

use super::{Alignable};

pub struct MyColumn {
    pack: group::Pack,
}

#[allow(unused)]
impl MyColumn {
    pub fn new(w: i32, h: i32) -> Self {
        let mut pack = group::Pack::default()
            .with_size(w, h);
        pack.set_type(PackType::Vertical);
        const PADDING: i32 = 3;
        pack.set_spacing(PADDING);

        MyColumn { pack }
    }

    pub fn end(&mut self) { self.pack.end(); }

    pub fn widget<'own>(&'own self) -> &'own group::Pack { 
        &self.pack 
    }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut group::Pack { 
        &mut self.pack 
    }
}

impl Alignable for MyColumn {
    fn resize(&mut self, w: i32, h: i32) { self.pack.set_size(w, h); }

    fn x(&self) -> i32 { self.pack.x() }

    fn y(&self) -> i32 { self.pack.y() }

    fn w(&self) -> i32 { self.pack.w() }

    fn h(&self) -> i32 { self.pack.h() }
}


pub struct MyRow {
    pack: group::Pack
}

#[allow(unused)]
impl MyRow {
    pub fn new(w: i32) -> Self {
        let mut pack = group::Pack::default()
            .with_size(w, 0);
        pack.set_type(PackType::Horizontal);
        const PADDING: i32 = 3;
        pack.set_spacing(PADDING);

        MyRow { pack }
    }

    pub fn with_pos(mut self, x: i32, y: i32) -> Self {
        self.pack.set_pos(x, y);
        self
    }

    pub fn end(&mut self) { 
        self.pack.end(); 

        let mut max_child_height = (0..self.pack.children())
            .map(|ch_num| self.pack.child(ch_num).unwrap().h())
            .max();

        self.pack.set_size(self.pack.w(), max_child_height.unwrap_or(0));
    }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut group::Pack { 
        &mut self.pack 
    }

    pub fn widget<'own>(&'own self) -> &'own group::Pack { 
        &self.pack 
    }
}

impl Alignable for MyRow {
    fn resize(&mut self, w: i32, h: i32) { self.pack.set_size(w, h); }

    fn x(&self) -> i32 { self.pack.x() }

    fn y(&self) -> i32 { self.pack.y() }
    
    fn w(&self) -> i32 { self.pack.w() }

    fn h(&self) -> i32 { self.pack.h() }
}
use fltk::{group::{self, PackType}, prelude::{GroupExt, WidgetExt}};

use super::{Alignable, Alignment};

#[allow(unused)]
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

    #[allow(unused)]
    pub fn with_pos(mut self, x: i32, y: i32) -> Self {
        self.pack.set_pos(x, y);
        self
    }

    pub fn end(&mut self) { self.pack.end(); }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut group::Pack { 
        &mut self.pack 
    }

    pub fn add_child<W: WidgetExt>(&mut self, child: &W, al: Alignment) {
        self.pack.begin();
        self.pack.add(child);
        self.pack.end();
    }
}

impl Alignable for MyColumn {
    fn alignment(&self) -> Alignment {
        todo!()
    }

    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.pack.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.pack.x() }

    fn y(&self) -> i32 { self.pack.y() }

    fn w(&self) -> i32 { self.pack.w() }

    fn h(&self) -> i32 { self.pack.h() }
}


#[allow(unused)]
pub struct MyRow {
    pack: group::Pack
}

#[allow(unused)]
impl MyRow {
    pub fn new(w: i32, h: i32) -> Self {
        let mut pack = group::Pack::default()
            .with_size(w, h);
        pack.set_type(PackType::Horizontal);
        const PADDING: i32 = 3;
        pack.set_spacing(PADDING);

        MyRow { pack }
    }

    pub fn with_pos(mut self, x: i32, y: i32) -> Self {
        self.pack.set_pos(x, y);
        self
    }

    pub fn end(&mut self) { self.pack.end(); }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut group::Pack { 
        &mut self.pack 
    }
}

impl Alignable for MyRow {
    fn alignment(&self) -> Alignment {
        todo!()
    }

    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32) { self.pack.resize(x, y, w, h); }

    fn x(&self) -> i32 { self.pack.x() }

    fn y(&self) -> i32 { self.pack.y() }
    
    fn w(&self) -> i32 { self.pack.w() }

    fn h(&self) -> i32 { self.pack.h() }
}
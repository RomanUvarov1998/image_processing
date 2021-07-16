
pub const TEXT_PADDING: i32 = 10;

const PADDING: i32 = 20;

pub mod container;
pub mod usual;
pub mod img_presenter;
pub mod step_editor;
pub mod line;
pub mod message;
pub mod small_dlg;
mod step;
mod embedded_images;

pub trait Alignable {
    fn resize(&mut self, w: i32, h: i32);
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn w(&self) -> i32;
    fn h(&self) -> i32;
}
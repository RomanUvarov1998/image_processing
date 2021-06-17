
pub const TEXT_PADDING: i32 = 10;

pub mod container;
pub mod usual;

pub trait Alignable {
    fn resize(&mut self, x: i32, y: i32, w: i32, h: i32);
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn w(&self) -> i32;
    fn h(&self) -> i32;
}
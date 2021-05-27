use fltk::prelude::WidgetExt;

pub fn err_msg<T: WidgetExt>(source: &T, msg: &str) {
    fltk::dialog::alert(source.w() / 2, source.h() / 2, &msg);
}
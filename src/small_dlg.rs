use fltk::{prelude::WidgetExt, window::Window};

pub fn err_msg(source: &Window, msg: &str) {
    fltk::dialog::alert(source.w() / 2, source.h() / 2, &msg);
}

pub fn confirm(source: &Window, question: &str) -> bool {
    let ans = fltk::dialog::choice(source.w() / 2, source.h() / 2, &question, "Да", "Нет","");
    ans == 0
}

pub fn info_msg(source: &Window, msg: &str) {
    fltk::dialog::message(source.w() / 2, source.h() / 2, &msg);
}
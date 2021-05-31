use fltk::prelude::WidgetExt;

pub fn err_msg<T: WidgetExt>(source: &T, msg: &str) {
    fltk::dialog::alert(source.w() / 2, source.h() / 2, &msg);
}

pub fn confirm<T: WidgetExt>(source: &T, question: &str) -> bool {
    let ans = fltk::dialog::choice(source.w() / 2, source.h() / 2, &question, "Да", "Нет","");
    ans == 0
}

pub fn info_msg<T: WidgetExt>(source: &T, msg: &str) {
    fltk::dialog::message(source.w() / 2, source.h() / 2, &msg);
}
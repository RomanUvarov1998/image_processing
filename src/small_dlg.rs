use fltk::{prelude::WidgetExt, window::Window};

pub fn show_err_msg(parent_window: &Window, msg: &str) {
    let (x, y) = count_box_pos(parent_window, msg);

    fltk::dialog::alert(x, y, &msg);
}

pub fn confirm_with_dlg(parent_window: &Window, question: &str) -> bool {
    let (x, y) = count_box_pos(parent_window, question);

    let ans = fltk::dialog::choice(x, y, &question, "Да", "Нет","");

    ans == 0
}

pub fn show_info_msg(parent_window: &Window, msg: &str) {
    let (x, y) = count_box_pos(parent_window, msg);

    fltk::dialog::message(x, y, &msg);
}

fn count_box_pos(parent_window: &Window, text: &str) -> (i32, i32) {
    let (text_w, text_h) = fltk::draw::measure(text, true);

    let x = parent_window.x() + parent_window.w() / 2 - text_w / 2;
    let y = parent_window.y() + parent_window.h() / 2 - text_h / 2;

    (x, y)
}
use crate::{my_err::MyError, utils::Pos};

pub fn show_err_msg(center_pos: Pos, err: MyError) {
    let msg = err.get_message();

    let (x, y) = count_box_pos(center_pos, &msg);

    fltk::dialog::alert(x, y, &msg);
}

pub fn confirm_with_dlg(center_pos: Pos, question: &str) -> bool {
    let (x, y) = count_box_pos(center_pos, question);

    let ans = fltk::dialog::choice(x, y, &question, "Да", "Нет", "");

    ans == 0
}

pub fn show_info_msg(center_pos: Pos, msg: &str) {
    let (x, y) = count_box_pos(center_pos, msg);

    fltk::dialog::message(x, y, &msg);
}

fn count_box_pos(center_pos: Pos, text: &str) -> (i32, i32) {
    let (text_w, text_h) = fltk::draw::measure(text, true);

    let x = center_pos.x - text_w / 2;
    let y = center_pos.y - text_h / 2;

    (x, y)
}

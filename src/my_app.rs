use std::result;
use crate::{filter, my_err::MyError};
use fltk::{ prelude::*, * };
use crate::img;
use ::image as ImgLib;

#[derive(Debug, Copy, Clone)]
enum Message {
    LoadInitialImage,
    ProcessLoadedImage
}

pub fn create_app() -> result::Result<(), MyError> {
    let app = app::App::default();
    let mut wind = window::Window::new(100, 100, 400, 400, "Main window");

    let mut pack_main = group::Pack::default()
        .size_of(&wind)
        .center_of(&wind);
    pack_main.set_type(group::PackType::Horizontal);
    pack_main.set_spacing(30);

    let (s, r) = app::channel::<Message>();

    let pack_left = group::Pack::default().with_size(wind.width() / 2, wind.height()); 
    let mut btn_load = button::Button::new(0, 5, 100, 30, "Load image");   
    btn_load.emit(s, Message::LoadInitialImage);
    let mut frame_left = frame::Frame::default().with_size(200, 300).with_label("Initial"); 
    let mut img_initial: Option<img::Img> = None;
    pack_left.end();

    let pack_right = group::Pack::default().with_size(wind.width() / 2, wind.height()).with_label("Processed");
    let mut btn_process = button::Button::new(100, 5, 100, 30, "Process image");
    btn_process.emit(s, Message::ProcessLoadedImage);
    let mut frame_right = frame::Frame::default().with_size(200, 300).with_label("Processed"); 
    let mut img_copy: Option<img::Img> = None;
    pack_right.end();

    let mut blur_filter = filter::LinearFilter::mean_filter_of_size(5);

    pack_main.end();

    wind.end();
    wind.make_resizable(true);
    wind.show();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::LoadInitialImage => {
                    img_initial = Some(load_img()?);
                    img_copy = img_initial.clone();
                    frame_left.set_image(Some(img_initial.unwrap().give_image()?));
                    frame_left.redraw();
                },
                Message::ProcessLoadedImage => {
                    match img_copy {
                        Some(ref mut img_ref) => {
                            let pixel_buf = img_ref.apply_filter(&mut blur_filter);
                            frame_right.set_image(Some(pixel_buf.give_image()?));
                            frame_right.redraw();
                        }
                        None => {
                            frame_right.set_label("You should choose image to process first");
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn load_img() -> result::Result<img::Img, MyError> {
    let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
    dlg.show();
    let path_buf = dlg.filename();

    let path_str = path_buf.to_str().unwrap();
    let img_dyn = ImgLib::io::Reader::open(path_str.to_string())?.decode()?;    

    Ok(img::Img::new(img_dyn))
}
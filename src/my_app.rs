use std::result;
use crate::{filter, my_err::MyError};
use fltk::{app, button, dialog, enums::{Align, FrameType}, frame, image::BmpImage, prelude::*, text, window};
use crate::img;
use ::image as ImgLib;

#[derive(Debug, Copy, Clone)]
enum Message {
    LoadInitialImage,
    ProcessLoadedImage,
}

const PADDING: i32 = 3;
const WIN_WIDTH: i32 = 640;
const WIN_HEIGHT: i32 = 480;
const BTN_WIDTH: i32 = 100;
const BTN_HEIGHT: i32 = 30;

pub fn create_app() -> result::Result<(), MyError> {
    let app = app::App::default();
    let mut wind = window::Window::default()
        .with_size(WIN_WIDTH, WIN_HEIGHT)
        .center_screen()
        .with_label("Main window");

    let (s, r) = app::channel::<Message>();

    let frame_resize_cbk = |f: &mut frame::Frame| { 
        match f.image() {
            Some(mut img) => {
                img.scale(f.width(), f.height(), true, true);
                img.draw(
                    f.x() + f.width() / 2 - img.width() / 2, 
                    f.y() + f.height() / 2 - img.height() / 2, 
                    f.width(), f.height());
                f.redraw();
            },
            None => { 
                f.set_label("");
            }
        }
    };

    //------------------------- frame_img_init -------------------------------------

    let mut frame_img_init = frame::Frame::default()
        .with_pos(0, BTN_HEIGHT * 2)
        .with_size(WIN_WIDTH / 2, WIN_HEIGHT - BTN_HEIGHT * 2);
    frame_img_init.set_frame(FrameType::EmbossedFrame);
    frame_img_init.set_align(Align::ImageMask | Align::TextNextToImage);
    frame_img_init.draw(frame_resize_cbk);

    //------------------------- btn_load -------------------------------------

    let mut btn_load = button::Button::default()
        .with_size(BTN_WIDTH, BTN_HEIGHT)
        .above_of(&frame_img_init, PADDING)
        .with_label("Load image");
    btn_load.emit(s, Message::LoadInitialImage);

    //------------------------- label init -------------------------------------

    let mut lbl_init = frame::Frame::default()
        .with_size(WIN_WIDTH / 2, BTN_HEIGHT)
        .above_of(&btn_load, PADDING)
        .with_label("init Img title");

    //------------------------- frame_img_proc -------------------------------------

    let mut frame_img_proc = frame::Frame::default()
        .with_pos(WIN_WIDTH / 2, BTN_HEIGHT * 2)
        .with_size(WIN_WIDTH / 2, WIN_HEIGHT - BTN_HEIGHT * 2);
    frame_img_proc.set_frame(FrameType::EmbossedFrame);
    frame_img_proc.set_align(Align::ImageMask | Align::TextNextToImage | Align::Bottom);    
    frame_img_proc.draw(frame_resize_cbk);

    //------------------------- btn_process -------------------------------------

    let mut btn_process = button::Button::default()
        .with_size(BTN_WIDTH, BTN_HEIGHT)
        .above_of(&frame_img_proc, PADDING)
        .with_label("Process image");
    btn_process.emit(s, Message::ProcessLoadedImage);

    //------------------------- label proc -------------------------------------

    let mut lbl_proc = frame::Frame::default()
        .with_size(WIN_WIDTH / 2, BTN_HEIGHT)
        .above_of(&btn_process, PADDING)
        .with_label("proc Img title");
    
    let mut img_initial: Option<(img::Img, BmpImage)> = None;
    let mut img_copy: Option<(img::Img, BmpImage)> = None;
    //let mut blur_filter = filter::LinearFilter::mean_filter_of_size(5);
    let mut blur_filter = filter::MedianFilter::new(3);

    wind.end();
    wind.make_resizable(true);
    wind.show();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match msg {
                Message::LoadInitialImage => {
                    let loaded_img = load_img()?;

                    lbl_init.set_label(&format!("Размер {}x{}", loaded_img.w(), loaded_img.h()));

                    let draw_data = loaded_img.clone().give_image()?;

                    let mut to_draw = draw_data.clone();
                    to_draw.scale(0, 0, true, true);
                    frame_img_init.set_image(Some(to_draw));

                    img_initial = Some((loaded_img, draw_data));
                    img_copy = img_initial.clone();
                    frame_img_init.redraw();
                },
                Message::ProcessLoadedImage => {
                    match img_copy {
                        Some(ref mut img_ref) => {
                            let mut img = img_ref.0.apply_filter(&mut blur_filter).give_image()?;

                            lbl_proc.set_label(&format!("Размер {}x{}", img.w(), img.h()));

                            img.scale(0, 0, true, true);
                            frame_img_proc.set_image(Some(img));
                            frame_img_proc.redraw();
                        }
                        None => {
                            frame_img_proc.set_label("You should choose image to process first");
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
use std::result;
use crate::{my_err::MyError, proc_steps::{self, StepType}};
use fltk::{app, group, prelude::*, window};

pub fn create_app() -> result::Result<(), MyError> {
    let app = app::App::default();
    let mut wind = window::Window::default()
        .with_size(proc_steps::WIN_WIDTH, proc_steps::WIN_HEIGHT)
        .center_screen()
        .with_label("Main window");

    let (sender, r) = app::channel::<proc_steps::Message>();

/*
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

    

    let mut tab_control = group::Tabs::default()
        .with_pos(0, 0)
        .with_size(WIN_WIDTH, WIN_HEIGHT);
    let tab1 = group::Group::default()
        .with_pos(PADDING,  PADDING + BTN_HEIGHT)
        .with_size(WIN_WIDTH - PADDING, WIN_HEIGHT - PADDING)
        .with_label("Tab 1");

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
    let mut blur_filter = filter::MedianFilter::new(5);

    tab1.end();
    tab_control.end();
    */
    
    let scroll_area = group::Scroll::default()
        .with_pos(0, 0)
        .with_size(proc_steps::WIN_WIDTH, proc_steps::WIN_HEIGHT);

    let mut steps_line = proc_steps::ProcessingLine::new(sender);
    steps_line.add(StepType::LinearFilter(5), sender);
    steps_line.add(StepType::MedianFilter(5), sender);

    scroll_area.end();

    wind.end();
    wind.make_resizable(true);
    wind.show();

    while app.wait() {
        if let Some(msg) = r.recv() {
            match steps_line.process_from_step(msg) {
                Err(err) => {
                    fltk::dialog::alert(
                        proc_steps::WIN_WIDTH / 2, 
                        proc_steps::WIN_HEIGHT / 2, 
                        &err.extract_msg());
                },
                Ok(_) => { }
            }
        }
    }

    Ok(())
}
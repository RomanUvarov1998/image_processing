use std::{cell::{RefCell}, rc::Rc};
use fltk::{frame, image::RgbImage, prelude::{ImageExt, WidgetBase, WidgetExt}};
use crate::{AssetItem, img::{PixelsArea}, my_component::{container::{MyColumn, MyRow}, usual::{MyButton, MyToggleButton}}, utils::{DragPos, DraggableRect, Pos, RectArea, ScalableRect}};
use super::Alignable;


pub struct MyImgPresenter {
    column: MyColumn,
    btns_row: MyRow,
    btn_fit: MyButton,
    btn_toggle_selection: MyToggleButton,
    frame_img: frame::Frame,
    img_pres_rect_rc: Option<Rc<RefCell<ImgPresRect>>>,
    tx_resized: Option<std::sync::mpsc::Sender<ImgPresMsg>>
}

impl MyImgPresenter {
    pub fn new(w: i32, h: i32) -> Self {
        let mut column = MyColumn::new(w, h);

        let mut btns_row = MyRow::new(w);

        let mut btn_fit = MyButton::with_img_and_tooltip(AssetItem::FitImage, "Уместить");
        btn_fit.set_active(false);
        
        let mut btn_toggle_selection = MyToggleButton::with_img_and_tooltip(AssetItem::CropImage, "Брать выделенное");
        btn_toggle_selection.set_active(false);

        btns_row.end();

        let mut frame_img = frame::Frame::default()
            .with_size(w, h - btns_row.h() - column.widget().spacing() * 3);
        use fltk::enums::{FrameType, Align};
        frame_img.set_frame(FrameType::EmbossedBox);
        frame_img.set_align(Align::Center); 
        
        column.end();

        MyImgPresenter { 
            column,
            btns_row, btn_fit, btn_toggle_selection, 
            frame_img, img_pres_rect_rc: None,
            tx_resized: None
        }
    }

    pub fn clear_image(&mut self) {
        self.tx_resized = None;

        self.btn_fit.set_active(false);
        self.btn_fit.widget_mut().set_callback(move |_| { });

        self.btn_toggle_selection.set_active(false);
        self.btn_toggle_selection.widget_mut().set_callback(move |_| { });
        self.btn_toggle_selection.set_toggle(false);

        self.frame_img.handle(|_, _| { false });
        self.frame_img.draw(|_| {});

        if let Some(ref rc) = self.img_pres_rect_rc {
            assert_eq!(Rc::strong_count(rc), 1);
            self.img_pres_rect_rc = None;
        }

        self.frame_img.redraw(); 
    }

    pub fn set_img(&mut self, img: RgbImage) {
        if let Some(_) = self.img_pres_rect_rc {
            self.clear_image();
        }

        let pres_rect = ImgPresRect::new(
            Pos::new(img.w(), img.h()), 
            RectArea::of_widget(&self.frame_img).with_zero_origin());
        let presenter_rc: Rc<RefCell<ImgPresRect>> = Rc::new(RefCell::new(pres_rect));

        let (tx, rx) = std::sync::mpsc::channel::<ImgPresMsg>();

        self.set_draw_cbk(img, Rc::clone(&presenter_rc), rx);

        self.img_pres_rect_rc = Some(presenter_rc);

        self.set_btn_toggle_cbk(tx.clone());
        self.set_btn_fit_cbk(tx.clone());
        self.tx_resized = Some(tx.clone());
        self.set_frame_handle_cbk(tx);
		
        self.frame_img.redraw(); 
    }

    fn set_draw_cbk(
        &mut self, 
        mut drawable: RgbImage, 
        presenter_rc: Rc<RefCell<ImgPresRect>>, 
        rx_draw: std::sync::mpsc::Receiver<ImgPresMsg>
    ) {
        self.frame_img.draw(move |frame| 
        {
            let view_area = RectArea::of_widget(frame);
            let view_area_size = view_area.size();
            let draw_position = Pos::of(frame);

            let mut presenter_rc_mut = presenter_rc.borrow_mut();

            while let Ok(msg) = rx_draw.try_recv() {
                presenter_rc_mut.consume_msg(msg, view_area_size);
            }

            drop(presenter_rc_mut);

            use fltk::draw;
            draw::push_clip(view_area.x(), view_area.y(), view_area.w(), view_area.h());
            
            presenter_rc.borrow_mut().draw_img(&mut drawable, draw_position);

            draw::pop_clip();
        });
    }

    fn set_btn_toggle_cbk(&mut self, tx: std::sync::mpsc::Sender<ImgPresMsg>) {
        let mut frame_copy = self.frame_img.clone();

        self.btn_toggle_selection.widget_mut().set_callback(move |btn| { 
            let msg = if btn.is_toggled() { ImgPresMsg::SeletionOn } else { ImgPresMsg::SelectionOff };
            tx.send(msg).unwrap();
            frame_copy.redraw();
        });
        self.btn_toggle_selection.set_active(true);
    }

    fn set_btn_fit_cbk(&mut self, tx: std::sync::mpsc::Sender<ImgPresMsg>) {
        let mut frame_copy = self.frame_img.clone();  
        let mut btn_toggle_selection_copy = self.btn_toggle_selection.clone();  

        self.btn_fit.widget_mut().set_callback(move |_| {
            tx.send(ImgPresMsg::Fit).unwrap();
            tx.send(ImgPresMsg::SelectionOff).unwrap();
            btn_toggle_selection_copy.toggle(false);
            frame_copy.redraw();
        });
        self.btn_fit.set_active(true);
    }

    fn set_frame_handle_cbk(&mut self, tx: std::sync::mpsc::Sender<ImgPresMsg>) {
        let mut was_mouse_down = false;
        self.frame_img.handle(move |f, ev| {
            let mouse_pos = Pos::new(fltk::app::event_x() - f.x(), fltk::app::event_y() - f.y());

            use fltk::app::MouseWheel;

            const SCROLL_DELTA: f32 = 0.2_f32;
            let factor_delta: f32 = match fltk::app::event_dy() {
                MouseWheel::None => 0_f32,
                MouseWheel::Down => SCROLL_DELTA,
                MouseWheel::Up => -SCROLL_DELTA,
                MouseWheel::Right | MouseWheel::Left => unreachable!("")
            };

            use fltk::enums::Event;
			let event_handled = match ev {
                Event::Push => {
                    was_mouse_down = true;
                    tx.send(ImgPresMsg::MouseDown (mouse_pos)).unwrap();
					true
                },
                Event::Released => {
                    was_mouse_down = false;
                    tx.send(ImgPresMsg::MouseUp).unwrap();
					true
                },
                Event::MouseWheel => {
                    if was_mouse_down {
                        tx.send(ImgPresMsg::MouseScroll { factor_delta, pos: mouse_pos }).unwrap();
						true
                    } else {
						false
                    }
                },
                Event::Drag => {
                    was_mouse_down = true;
                    tx.send(ImgPresMsg::MouseMove (mouse_pos)).unwrap();
                    true
                },
                _ => false
            };

			if event_handled {
            	f.redraw();
			}

            event_handled
        });
    }


    pub fn get_selection_rect(&self) -> Option<PixelsArea> { 
        if self.btn_toggle_selection.is_toggled() {
            let presenter_rc_mut = self.img_pres_rect_rc
                .as_ref()
                .expect("image_copy(): Presenter rect is None")
                .try_borrow()
                .expect("Couldn't get & to presenter from image_copy()");

            let scale_rect: &ScalableRect = &presenter_rc_mut.scale_rect;
            let sel_rect: &SelectionRect = presenter_rc_mut.selection_rect
                .as_ref()
                .expect("Selection mode btn is ON but there sel_rect is None");

            let tl: Pos = scale_rect.self_to_pixel(sel_rect.inner.tl());
            let br: Pos = scale_rect.self_to_pixel(sel_rect.inner.br());

            drop(presenter_rc_mut);

            Some(PixelsArea::new(tl.to_pixel_pos(), br.to_pixel_pos()))
        } else {
            None
        }
    }
}

impl Alignable for MyImgPresenter {
    fn resize(&mut self, w: i32, h: i32) { 
        self.frame_img.set_size(w, h - self.btns_row.h() - self.column.widget().spacing() * 3);
        
        if let Some(ref tx) = self.tx_resized {
            tx.send(ImgPresMsg::ComponentResized).unwrap();
            self.frame_img.redraw(); 
        }
    }

    fn x(&self) -> i32 { self.frame_img.x() }

    fn y(&self) -> i32 { self.frame_img.y() }

    fn w(&self) -> i32 { self.frame_img.w() }

    fn h(&self) -> i32 { self.frame_img.h() }
}


struct ImgPresRect {
    scale_rect: ScalableRect,
    prev_pos: Option<Pos>,
    selection_rect: Option<SelectionRect>
}

impl ImgPresRect {
    fn new(img_size: Pos, frame_area: RectArea) -> Self {
        let mut rect = ScalableRect::new(0, 0, img_size.x, img_size.y);
        rect.stretch_self_to_area(frame_area);
        
        ImgPresRect { 
            scale_rect: rect,
            prev_pos: None,
            selection_rect: None
        }
    }


    fn consume_msg(&mut self, msg: ImgPresMsg, current_view_area_size: Pos) {
        let view_area = RectArea::new(
            0, 0, 
            current_view_area_size.x, current_view_area_size.y);

        match msg {
            ImgPresMsg::MouseDown (pos) => self.start_drag(pos),
            ImgPresMsg::MouseMove (cur) => self.drag(cur),
            ImgPresMsg::MouseUp => self.stop_drag(view_area),
            ImgPresMsg::MouseScroll { factor_delta, pos } => self.scale(pos, factor_delta),
            ImgPresMsg::Fit => {
                if let Some(ref sel_rect) = self.selection_rect {
                    let center = current_view_area_size.div_f(2_f32);
                    self.scale_rect.zoom_area(
                        RectArea::of_draggable_rect(&sel_rect.inner),
                        center);
                } else {
                    self.scale_rect.stretch_self_to_area(view_area);
                }
            },
            ImgPresMsg::SeletionOn => {
                self.selection_rect = Some(SelectionRect::middle_third_of(current_view_area_size));
            },
            ImgPresMsg::SelectionOff => {
                self.selection_rect = None;
            }
            ImgPresMsg::ComponentResized => {
                let view_size = view_area.size();
                self.scale_rect.fit_scale(view_size);
                self.scale_rect.fit_pos(RectArea::new(0, 0, view_size.x, view_size.y));
            },
        }
    }


    fn start_drag(&mut self, pos: Pos) {
        self.prev_pos = Some(pos);

        if let Some(ref mut rect) = self.selection_rect {
            rect.start_drag(pos);
        }
    }

    fn drag(&mut self, to: Pos) {
        let prev = match self.prev_pos {
            Some(pos) => pos,
            None => { return; },
        };

        let delta = to - prev;
        self.prev_pos = Some(to);

        if let Some(ref mut sel_rect) = self.selection_rect {
            if !sel_rect.drag(delta)  {
                self.scale_rect.translate(delta);
            }
        } else {
            self.scale_rect.translate(delta);
        }
    }

    fn scale(&mut self, anchor: Pos, delta: f32) {
        self.scale_rect.scale_keep_anchor_pos(delta, anchor);
    }

    fn stop_drag(&mut self, wiew_area: RectArea) {
        self.prev_pos = None;

        self.scale_rect.fit_scale(wiew_area.size());
        self.scale_rect.fit_pos(wiew_area);

        if let Some(ref mut sel_rect) = self.selection_rect {
            sel_rect.stop_drag();
            sel_rect.inner.fit_inside(wiew_area);
            sel_rect.inner.fit_inside(self.scale_rect.area_scaled());
        }
    }


    fn draw_img(&mut self, img: &mut fltk::image::RgbImage, draw_position: Pos) {
		let (im_w, im_h) = (self.scale_rect.scaled_w(), self.scale_rect.scaled_h());
        img.scale(im_w, im_h, true, true);

        
        let im_pos = self.scale_rect.tl();
        img.draw(draw_position.x + im_pos.x, draw_position.y + im_pos.y, im_w, im_h);

        if let Some(ref rect) = self.selection_rect {
            rect.draw(draw_position.x, draw_position.y);
        }
    }
}


#[derive(Debug)]
struct SelectionRect {
    inner: DraggableRect,
    drag_pos: Option<DragPos>
}

impl SelectionRect {
    fn middle_third_of(area_size: Pos) -> Self {
        let w = area_size.x / 3;
        let h = area_size.y / 3;
        let x = w;
        let y = h;
        
        SelectionRect { 
            inner: DraggableRect::new(x, y, w, h) ,
            drag_pos: None 
        }
    }

    fn x(&self) -> i32 { self.inner.x() }
    fn y(&self) -> i32 { self.inner.y() }
    fn w(&self) -> i32 { self.inner.w() }
    fn h(&self) -> i32 { self.inner.h() }

    const RECT_SIDE: i32 = 10;

    fn draw(&self, ox: i32, oy: i32) {
        use fltk::{draw, enums::Color};

        draw::draw_rect_with_color(
            ox + self.x(), oy + self.y(), 
            self.w(), self.h(),
            Color::Blue);

        let draw_rect_around = |x: i32, y: i32, fill_color: Color| {
            let (rx, ry) = (x - Self::RECT_SIDE / 2, y - Self::RECT_SIDE / 2);

            draw::draw_rect_fill(
                rx, ry, 
                Self::RECT_SIDE, Self::RECT_SIDE, 
                fill_color);
            draw::draw_rect_with_color(
                rx, ry, 
                Self::RECT_SIDE, Self::RECT_SIDE, 
                Color::Blue);
        };

        let w_half = self.w() / 2;
        let h_half = self.h() / 2;

        for x_step in 0..3 {
            for y_step in 0..3 {
                let fill_color: Color = 
                    if let Some(dp) = self.drag_pos {
                        if dp == DragPos::from(x_step, y_step) {
                            Color::Green 
                        } else {
                            Color::Red
                        }
                    } else {
                        Color::Red
                    };

                draw_rect_around(
                    ox + self.x() + w_half * x_step, 
                    oy + self.y() + h_half * y_step,
                    fill_color);
            }
        }
    }

    fn start_drag(&mut self, pos: Pos) {
        let w_half = self.w() / 2;
        let h_half = self.h() / 2;

        let fits_rect = |rcx: i32, rcy: i32, p: Pos| -> bool {
            p.x >= rcx - Self::RECT_SIDE 
            && p.x <= rcx + Self::RECT_SIDE
            && p.y >= rcy - Self::RECT_SIDE 
            && p.y <= rcy + Self::RECT_SIDE
        };

        self.drag_pos = None;
        'out: for x_step in 0..3 {
            for y_step in 0..3 {
                if fits_rect(self.x() + w_half * x_step, self.y() + h_half * y_step, pos) {
                    self.drag_pos = Some(DragPos::from(x_step, y_step));
                    break 'out;
                }
            }
        }
    }

    fn stop_drag(&mut self) {
        self.drag_pos = None;
    }

    fn drag(&mut self, delta: Pos) -> bool  {
        if let Some(ref mut dt) = self.drag_pos {
            *dt = self.inner.drag(delta, *dt);
            return true;
        }
        return false;
    }
}


#[derive(Clone, Copy, Debug)]
enum ImgPresMsg {
    MouseDown (Pos),
    MouseMove (Pos),
    MouseUp,
    MouseScroll { factor_delta: f32, pos: Pos },
    Fit,
    SeletionOn, SelectionOff,
    ComponentResized
}


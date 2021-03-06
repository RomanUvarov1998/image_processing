use super::embedded_images::{Asset, AssetItem};
use super::{Alignable, TEXT_PADDING};
use crate::utils::{Pos, WordsIter};
use fltk::{
    app::Sender,
    button,
    enums::Shortcut,
    frame, menu, misc,
    prelude::{ImageExt, MenuExt, WidgetBase, WidgetExt},
};

const IMG_PADDING: i32 = 5;

enum ImgPadding {
    Sides {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    },
    All(i32),
}

trait MyComponentWithImage {
    fn set_image_from_asset(&mut self, item: AssetItem, padding: ImgPadding)
    where
        Self: WidgetExt + WidgetBase,
    {
        let path = item.to_path();
        let bytes = Asset::get(path).expect(&format!(
            "Couldn't load image from embedded asset by path '{}'",
            path
        ));
        let mut img = fltk::image::PngImage::from_data(&bytes[..]).expect(&format!(
            "Couldn't load image from embedded bytes by path '{}'",
            path
        ));

        let (pl, pt, pr, pb) = match padding {
            ImgPadding::Sides {
                left,
                top,
                right,
                bottom,
            } => (left, top, right, bottom),
            ImgPadding::All(p) => (p, p, p, p),
        };

        self.set_size(pl + img.w() + pr, pt + img.h() + pb);

        self.draw(move |wid| {
            img.draw(wid.x() + pl, wid.y() + pt, img.w(), img.h());
        });
    }
}

pub struct MyButton {
    btn: button::Button,
}

#[allow(unused)]
impl MyButton {
    pub fn with_label<'label>(label: &'label str) -> Self {
        let mut btn = button::Button::default();
        btn.set_label(label);

        let (w, h) = btn.measure_label();
        btn.set_size(w + TEXT_PADDING, h + TEXT_PADDING);

        MyButton { btn }
    }

    pub fn with_img_and_tooltip(item: AssetItem, tooltip: &str) -> Self {
        let mut btn = button::Button::default();
        btn.set_image_from_asset(item, ImgPadding::All(IMG_PADDING));
        btn.set_tooltip(tooltip);

        MyButton { btn }
    }

    pub fn set_emit<TMsg>(&mut self, tx: Sender<TMsg>, msg: TMsg)
    where
        TMsg: 'static + Clone + Copy + Send + Sync,
    {
        self.btn.emit(tx, msg);
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.btn.activate();
        } else {
            self.btn.deactivate();
        }
    }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut button::Button {
        &mut self.btn
    }
}

impl Alignable for MyButton {
    fn resize(&mut self, w: i32, h: i32) {
        self.btn.set_size(w, h);
    }

    fn x(&self) -> i32 {
        self.btn.x()
    }

    fn y(&self) -> i32 {
        self.btn.y()
    }

    fn w(&self) -> i32 {
        self.btn.w()
    }

    fn h(&self) -> i32 {
        self.btn.h()
    }
}

impl MyComponentWithImage for button::Button {}

#[derive(Clone)]
pub struct MyToggleButton {
    btn: button::ToggleButton,
}

#[allow(unused)]
impl MyToggleButton {
    pub fn with_label<'label>(label: &'label str) -> Self {
        let mut btn = button::ToggleButton::default();
        btn.set_label(label);

        let (w, h) = btn.measure_label();
        btn.set_size(w + TEXT_PADDING, h + TEXT_PADDING);

        MyToggleButton { btn }
    }

    pub fn with_img_and_tooltip(item: AssetItem, tooltip: &str) -> Self {
        let mut btn = button::ToggleButton::default();
        btn.set_image_from_asset(item, ImgPadding::All(IMG_PADDING));
        btn.set_tooltip(tooltip);

        MyToggleButton { btn }
    }

    pub fn toggle(&mut self, value: bool) {
        self.btn.toggle(value);
    }

    pub fn is_toggled(&self) -> bool {
        self.btn.is_toggled()
    }

    pub fn set_toggle(&mut self, value: bool) {
        self.btn.toggle(value);
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.btn.activate();
        } else {
            self.btn.deactivate();
        }
    }

    pub fn set_emit<TMsg>(&mut self, tx: Sender<TMsg>, msg: TMsg)
    where
        TMsg: 'static + Clone + Copy + Send + Sync,
    {
        self.btn.emit(tx, msg);
    }

    pub fn widget_mut<'own>(&'own mut self) -> &'own mut button::ToggleButton {
        &mut self.btn
    }

    pub fn widget<'own>(&'own self) -> &'own button::ToggleButton {
        &self.btn
    }
}

impl Alignable for MyToggleButton {
    fn resize(&mut self, w: i32, h: i32) {
        self.btn.set_size(w, h);
    }

    fn x(&self) -> i32 {
        self.btn.x()
    }

    fn y(&self) -> i32 {
        self.btn.y()
    }

    fn w(&self) -> i32 {
        self.btn.w()
    }

    fn h(&self) -> i32 {
        self.btn.h()
    }
}

impl MyComponentWithImage for button::ToggleButton {}

pub struct MyLabel {
    inner: frame::Frame,
    text: String,
}

#[allow(unused)]
impl MyLabel {
    pub fn new<'text>(text: &'text str, w: i32) -> Self {
        let mut inner = frame::Frame::default();

        let mut label = MyLabel {
            inner,
            text: text.to_string(),
        };

        let mut cbk = Self::create_draw_callback(text);
        label.inner.set_size(w, 0);
        cbk(&mut label.inner);

        label.inner.draw(cbk);

        label
    }

    pub fn set_text<'text>(&mut self, text: &'text str) {
        self.text = text.to_string();
        self.inner.draw(Self::create_draw_callback(text));
        self.inner.redraw();
    }

    fn create_draw_callback(text: &str) -> Box<dyn FnMut(&mut frame::Frame) -> ()> {
        use fltk::draw;
        let content = text.to_string();
        let mut content_wrapped = String::new();
        let mut prev_label_size = Pos::new(-1, -1);

        let cbk = move |label: &mut frame::Frame| {
            if label.w() != prev_label_size.x || label.h() != prev_label_size.y {
                content_wrapped.clear();
                prev_label_size = Pos::new(label.w(), label.h());

                let mut line = String::new();

                let mut acc_h = 0;
                let mut acc_w = 0;
                let (space_w, space_h) = draw::measure(" ", true);

                let mut words_iter = WordsIter::new(&content, " ");
                while let Some(word) = words_iter.next() {
                    let (ww, wh) = draw::measure(&word, true);

                    if acc_w + ww > label.w() {
                        content_wrapped.push_str(&line);
                        content_wrapped.push('\n');
                        line.clear();
                        acc_w = 0;
                        acc_h += wh;
                    }

                    acc_w += ww + space_w;
                    line.push_str(word);
                    line.push(' ');
                }

                acc_h += space_h;
                content_wrapped.push_str(&line);

                label.set_size(label.w(), acc_h);
            }

            draw::push_clip(label.x(), label.y(), label.w(), label.h());
            draw::set_color_rgb(0, 0, 0);
            draw::draw_text2(
                &content_wrapped,
                label.x(),
                label.y(),
                label.w(),
                label.h(),
                fltk::enums::Align::Center,
            );
            draw::pop_clip();
        };

        Box::new(cbk)
    }
}

impl Alignable for MyLabel {
    fn resize(&mut self, w: i32, h: i32) {
        self.inner.set_size(w, h);
        self.inner.redraw();
    }

    fn x(&self) -> i32 {
        self.inner.x()
    }

    fn y(&self) -> i32 {
        self.inner.y()
    }

    fn w(&self) -> i32 {
        self.inner.w()
    }

    fn h(&self) -> i32 {
        self.inner.h()
    }
}

pub struct MyMenuBar {
    mb: menu::MenuBar,
}

#[allow(unused)]
impl MyMenuBar {
    pub fn new(w: i32) -> Self {
        MyMenuBar {
            mb: menu::MenuBar::default().with_size(w, 30),
        }
    }

    pub fn add_emit<'label, TMsg>(&mut self, label: &'label str, tx: Sender<TMsg>, msg: TMsg)
    where
        TMsg: 'static + Clone + Copy + Send + Sync,
    {
        self.mb
            .add_emit(label, Shortcut::None, menu::MenuFlag::Normal, tx, msg);
    }

    pub fn end(&mut self) {
        self.mb.end();
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.mb.activate();
        } else {
            self.mb.deactivate();
        }
    }
}

impl Alignable for MyMenuBar {
    fn resize(&mut self, w: i32, h: i32) {
        self.mb.set_size(w, h);
    }

    fn x(&self) -> i32 {
        self.mb.x()
    }

    fn y(&self) -> i32 {
        self.mb.y()
    }

    fn w(&self) -> i32 {
        self.mb.w()
    }

    fn h(&self) -> i32 {
        self.mb.h()
    }
}

pub struct MyMenuButton {
    btn: menu::MenuButton,
}

#[allow(unused)]
impl MyMenuButton {
    pub fn with_img_and_tooltip(item: AssetItem, tooltip: &str) -> Self {
        let mut btn = menu::MenuButton::default();

        btn.set_image_from_asset(
            item,
            ImgPadding::Sides {
                left: IMG_PADDING,
                top: IMG_PADDING,
                right: IMG_PADDING + Self::ARROW_WIDTH,
                bottom: IMG_PADDING,
            },
        );

        btn.set_tooltip(tooltip);

        MyMenuButton { btn }
    }

    pub fn with_label<'label>(label: &'label str) -> Self {
        let mut btn = menu::MenuButton::default();
        btn.set_label(label);

        let (w, h) = btn.measure_label();
        btn.set_size(w + TEXT_PADDING + Self::ARROW_WIDTH, h + TEXT_PADDING);

        MyMenuButton { btn }
    }

    const ARROW_WIDTH: i32 = 30;

    pub fn add_emit<'label, TMsg>(&mut self, label: &'label str, tx: Sender<TMsg>, msg: TMsg)
    where
        TMsg: 'static + Clone + Copy + Send + Sync,
    {
        self.btn
            .add_emit(label, Shortcut::None, menu::MenuFlag::Normal, tx, msg);
    }

    pub fn set_active(&mut self, active: bool) {
        if active {
            self.btn.activate();
        } else {
            self.btn.deactivate();
        }
    }
}

impl Alignable for MyMenuButton {
    fn resize(&mut self, w: i32, h: i32) {
        self.btn.set_size(w, h);
    }

    fn x(&self) -> i32 {
        self.btn.x()
    }

    fn y(&self) -> i32 {
        self.btn.y()
    }

    fn w(&self) -> i32 {
        self.btn.w()
    }

    fn h(&self) -> i32 {
        self.btn.h()
    }
}

impl MyComponentWithImage for menu::MenuButton {}

pub struct MyProgressBar {
    bar: misc::Progress,
    progress_percents: usize,
    label: String,
}

#[allow(unused)]
impl MyProgressBar {
    pub fn new(w: i32, h: i32) -> Self {
        let mut bar = misc::Progress::default().with_size(w, h);
        bar.set_minimum(0_f64);
        bar.set_maximum(100_f64);
        bar.set_selection_color(fltk::enums::Color::Green);

        MyProgressBar {
            bar,
            progress_percents: 0,
            label: String::new(),
        }
    }

    pub fn set_width(&mut self, new_w: i32) {
        self.bar.set_size(new_w, self.bar.h());
    }

    pub fn reset(&mut self, label: String) {
        self.label = label;
        self.set_value(0);
    }

    pub fn set_value(&mut self, progress_percents: usize) {
        if self.progress_percents != progress_percents {
            self.progress_percents = progress_percents;
            self.bar.set_value(progress_percents as f64);
            self.bar
                .set_label(&format!("{}: {}%", self.label, progress_percents));
        }
    }

    pub fn show(&mut self) {
        self.bar.show();
        self.bar.parent().as_mut().unwrap().redraw();
    }
    pub fn hide(&mut self) {
        self.bar.hide();
        self.bar.parent().as_mut().unwrap().redraw();
    }
}

impl Alignable for MyProgressBar {
    fn resize(&mut self, w: i32, h: i32) {
        self.bar.set_size(w, h);
    }

    fn x(&self) -> i32 {
        self.bar.x()
    }

    fn y(&self) -> i32 {
        self.bar.y()
    }

    fn w(&self) -> i32 {
        self.bar.w()
    }

    fn h(&self) -> i32 {
        self.bar.h()
    }
}

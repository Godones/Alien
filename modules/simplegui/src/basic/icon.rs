use alloc::{string::String, sync::Arc, vec::Vec};
use embedded_graphics::{
    image::Image,
    pixelcolor::Rgb888,
    prelude::{Point, RgbColor, Size},
    Drawable,
};
use tinybmp::Bmp;

use super::{Component, Graphics};
use crate::basic::TextEdit;
use crate::UPIntrFreeCell;

static FILEICON: &[u8] = include_bytes!("../../assert/folder.bmp");

pub struct IconController {
    inner: UPIntrFreeCell<IconControllerInner>,
}
#[allow(unused)]
struct IconControllerInner {
    files: Vec<String>,
    graphic: Graphics,
    parent: Option<Arc<dyn Component>>,
}

impl IconController {
    pub fn new(x: u32, y: u32, files: Vec<String>, parent: Option<Arc<dyn Component>>) -> Self {
        // 将整个桌面作为图床
        IconController {
            inner: UPIntrFreeCell::new(IconControllerInner {
                files,
                graphic: Graphics {
                    size: Size::new(x, y),
                    point: Point::new(0, 0),
                },
                parent,
            }),
        }
    }
}

impl Component for IconController {
    fn paint(&self) {
        let mut inner = self.inner.exclusive_access();
        let mut x = 10;
        let mut y = 10;
        let v = inner.files.clone();
        for file in v {
            let bmp = Bmp::<Rgb888>::from_slice(FILEICON).unwrap();
            Image::new(&bmp, Point::new(x, y))
                .draw(&mut inner.graphic)
                .expect("make image error");
            // let text = Text::new(
            //     file.as_str(),
            //     Point::new(x, y + 80),
            //     MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE),
            // );
            // //20+64
            // text.draw(&mut inner.graphic).expect("draw text error");
            let edit = TextEdit::new(Size::new(64, 20), Point::new(x, y + 64), None);
            edit.with_font_color(Rgb888::WHITE)
                .add_str(file.as_str())
                .repaint();

            if y >= 600 {
                x = x + 70;
                y = 10;
            } else {
                y = y + 90;
            }
        }
    }

    fn add(&self, _comp: Arc<dyn Component>) {
        todo!()
    }

    fn bound(&self) -> (Size, Point) {
        todo!()
    }
}

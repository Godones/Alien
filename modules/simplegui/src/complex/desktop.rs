use crate::basic::{Bar, Button, Component, IconController, ImageComp, Panel, Windows};
use crate::complex::terminal::GodTerminal;
use crate::UPIntrFreeCell;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec;
use embedded_graphics::prelude::{Point, Size};
use log::info;

static DT: &[u8] = include_bytes!("../../assert/desktop.bmp");

pub struct Desktop {
    inner: UPIntrFreeCell<DesktopInner>,
}

pub struct DesktopInner {
    pub desktop: Arc<dyn Component>,
}

impl Desktop {
    pub fn new(x_res: u32, y_res: u32) -> Self {
        let p: Arc<dyn Component + 'static> =
            Arc::new(Panel::new(Size::new(x_res, y_res), Point::new(0, 0)));

        let image = ImageComp::new(
            Size::new(x_res, y_res),
            Point::new(0, 0),
            DT,
            Some(p.clone()),
        );
        let icon = IconController::new(
            x_res,
            y_res,
            vec!["f1".to_string(), "f2".to_string()],
            Some(p.clone()),
        );
        p.add(Arc::new(image)); // background
        p.add(Arc::new(icon)); // icon

        let bar = Arc::new(Bar::new(
            Size::new(x_res, 48),
            Point::new(0, 752),
            Some(p.clone()),
        ));
        static MENU_BMP: &[u8] = include_bytes!("../../assert/rust.bmp");
        let img = ImageComp::new(
            Size::new(48, 48),
            bar.bound().1,
            MENU_BMP,
            Some(bar.clone()),
        );
        bar.add(Arc::new(img));
        let rtc_time = "2020-12-12\n12:12:12";

        let time_button = Arc::new(Button::new(
            Size::new(100, 48),
            Point::new(x_res as i32 - 100, 0),
            Some(bar.clone()),
            rtc_time.to_string(),
        ));
        // bar.add(Arc::new(img));
        bar.add(time_button);
        // bar.add(Arc::new(img1));
        // bar.paint();
        // img.paint();
        p.add(bar.clone());
        Self {
            inner: UPIntrFreeCell::new(DesktopInner { desktop: p }),
        }
    }
    pub fn paint(&self) {
        let inner = self.inner.exclusive_access();
        inner.desktop.paint();
    }
}

pub fn create_desktop(x: usize, y: usize) -> isize {
    // create_windows();
    create_god_terminal();
    6
}

pub fn create_god_terminal() {
    info!("create god terminal");
    let god_terminal = GodTerminal::new(Size::new(500, 500), Point::new(400, 100));
}

pub fn create_windows() {
    let windows = Windows::new(Size::new(500, 500), Point::new(40, 40));
    windows.set_title("windows").paint();
    let windows1 = Windows::new(Size::new(500, 500), Point::new(500, 200));
    windows1.set_title("Terminal").paint();
}

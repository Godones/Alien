use crate::basic::{Component, Windows};
use crate::UPIntrFreeCell;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::cell::RefCell;
use embedded_graphics::prelude::{Point, Size};
use lazy_static::lazy_static;
use log::info;
use spin::Mutex;

const VIRTGPU_XRES: usize = 1280;
const VIRTGPU_YRES: usize = 800;

pub struct Screen {
    inner: UPIntrFreeCell<ScreenInner>,
}
pub struct ScreenInner {
    flag: Box<[[usize; VIRTGPU_XRES as usize]; VIRTGPU_YRES as usize]>,
    windows: BTreeMap<usize, Arc<Windows>>,
}

lazy_static! {
    pub static ref SCREEN_MANAGER: UPIntrFreeCell<Screen> =
        unsafe { UPIntrFreeCell::new(Screen::new()) };
}
impl Screen {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPIntrFreeCell::new(ScreenInner {
                    flag: Box::new([[0; VIRTGPU_XRES as usize]; VIRTGPU_YRES as usize]),
                    windows: BTreeMap::new(),
                })
            },
        }
    }
    pub fn update(&mut self, size: Size, point: Point, window: Arc<Windows>) {
        let mut inner = self.inner.exclusive_access();
        for i in point.y..point.y + size.height as i32 {
            for j in point.x..point.x + size.width as i32 {
                inner.flag[i as usize][j as usize] = window.id();
            }
        }
        inner.windows.insert(window.id(), window);
    }
    pub fn get_window(&self, point: Point) -> Option<Arc<Windows>> {
        let id = self.inner.exclusive_access().flag[point.y as usize][point.x as usize];
        if id == 0 {
            None
        } else {
            Some(
                self.inner
                    .exclusive_access()
                    .windows
                    .get(&id)
                    .unwrap()
                    .clone(),
            )
        }
    }
    pub fn get_windows_num(&self) -> usize {
        self.inner.exclusive_access().windows.len()
    }
    pub fn mouse_left_press(&self, point: Point) {
        if let Some(window) = self.get_window(point) {
            window.paint();
        }
    }
}

use crate::basic::{Component, Graphics};
use crate::UPIntrFreeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::Drawable;

pub struct Bar {
    inner: UPIntrFreeCell<BarInner>,
}

struct BarInner {
    graphic: Graphics,                   //画布
    comps: VecDeque<Arc<dyn Component>>, //组件列表
    #[allow(unused)]
    parent: Option<Arc<dyn Component>>, //父组件
}

impl Component for Bar {
    fn paint(&self) {
        let mut inner = self.inner.exclusive_access();
        let rect = Rectangle::new(Point::new(0, 0), inner.graphic.size);
        rect.into_styled(PrimitiveStyle::with_fill(Rgb888::new(0x80, 0x80, 0x80)))
            .draw(&mut inner.graphic)
            .unwrap();
        let len = inner.comps.len();
        drop(inner);
        for i in 0..len {
            let inner = self.inner.exclusive_access();
            let comp = Arc::downgrade(&inner.comps[i]);
            drop(inner);
            comp.upgrade().unwrap().paint();
        }
    }
    fn add(&self, comp: Arc<dyn Component>) {
        self.inner.exclusive_access().comps.push_back(comp);
    }
    fn bound(&self) -> (Size, Point) {
        let inner = self.inner.exclusive_access();
        let point = inner.graphic.point;
        let size = inner.graphic.size;
        (size, point)
    }
}

impl Bar {
    pub fn new(size: Size, point: Point, parent: Option<Arc<dyn Component>>) -> Self {
        Self {
            inner: UPIntrFreeCell::new(BarInner {
                graphic: Graphics { size, point },
                comps: VecDeque::new(),
                parent,
            }),
        }
    }
    pub fn background() -> Rgb888 {
        Rgb888::new(0x80, 0x80, 0x80)
    }
}

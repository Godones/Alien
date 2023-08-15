use crate::basic::{Component, Graphics};
use crate::UPIntrFreeCell;
use alloc::string::String;
use alloc::sync::Arc;

pub struct Label {
    inner: UPIntrFreeCell<LabelInner>,
}

struct LabelInner {
    text: String,
    graphic: Graphics,
    parent: Option<Arc<dyn Component>>,
}

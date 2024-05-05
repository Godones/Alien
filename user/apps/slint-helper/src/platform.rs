use alloc::rc::Rc;
use core::time::Duration;

use slint::platform::software_renderer::MinimalSoftwareWindow;
use Mstd::time::{TimeSpec, TimeVal};

pub struct MyPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_timer: TimeSpec,
}

impl slint::platform::Platform for MyPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> Duration {
        let old_time = self.start_timer;
        let new_time = TimeSpec::from(TimeVal::now());
        Duration::new(new_time.tv_sec as u64, new_time.tv_nsec as u32)
            - Duration::new(old_time.tv_sec as u64, old_time.tv_nsec as u32)
    }
}

impl MyPlatform {
    pub fn new(window: Rc<MinimalSoftwareWindow>) -> Self {
        Self {
            window,
            start_timer: TimeSpec::from(TimeVal::now()),
        }
    }
}

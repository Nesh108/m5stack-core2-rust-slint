use slint::platform::{Platform, WindowAdapter};
use slint::platform::software_renderer::MinimalSoftwareWindow;
use alloc::rc::Rc;
use alloc::boxed::Box;
use core::time::Duration;
use core::sync::atomic::{AtomicU32, Ordering};

static START_TIME: AtomicU32 = AtomicU32::new(0);

fn get_time_ms() -> u32 {
    unsafe { (esp_idf_sys::esp_timer_get_time() / 1000) as u32 }
}

pub fn init_start_time() {
    START_TIME.store(get_time_ms(), Ordering::Relaxed);
}

pub struct M5StackPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl M5StackPlatform {
    pub fn new(width: u32, height: u32) -> (Box<Self>, Rc<MinimalSoftwareWindow>) {
        let window = MinimalSoftwareWindow::new(Default::default());
        window.set_size(slint::PhysicalSize::new(width, height));
        
        let platform = Box::new(Self {
            window: window.clone(),
        });
        
        (platform, window)
    }
}

impl Platform for M5StackPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    
    fn duration_since_start(&self) -> Duration {
        let start = START_TIME.load(Ordering::Relaxed);
        let now = get_time_ms();
        Duration::from_millis((now - start) as u64)
    }
}

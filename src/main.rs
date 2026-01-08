extern crate alloc;

mod config;
mod m5stack;
mod slint_platform;

use esp_idf_hal::{
    delay::FreeRtos,
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    spi::{SpiDeviceDriver, SpiDriver, config::Config},
    gpio::PinDriver,
    units::Hertz,
};
use esp_idf_sys as _;
use slint::platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};
use slint::platform::{WindowEvent, PointerEventButton};
use slint::PhysicalPosition;
use alloc::rc::Rc;
use alloc::vec::Vec;
use display_interface_spi::SPIInterface;

slint::include_modules!();

#[derive(Debug, PartialEq)]
enum TouchState {
    None,
    Pressed(u16, u16),
}

/// Check if touch is in one of the button zones (A, B, C) at bottom of screen
fn check_button_zone(x: u16, y: u16) -> Option<&'static str> {
    // M5Stack Core2 virtual buttons are in bottom zone (y > 210)
    // Button A: x 0-106, Button B: x 107-213, Button C: x 214-320
    if y < 210 {
        return None;
    }
    
    if x < 107 {
        Some("BtnA")
    } else if x < 214 {
        Some("BtnB")
    } else {
        Some("BtnC")
    }
}

fn process_touch_events(
    i2c: &mut I2cDriver,
    window: &Rc<MinimalSoftwareWindow>,
    touch_state: &mut TouchState,
) {
    if let Some((x, y)) = m5stack::read_touch(i2c) {
        match touch_state {
            TouchState::None => {
                *touch_state = TouchState::Pressed(x, y);
                
                if let Some(button) = check_button_zone(x, y) {
                    println!("{} pressed at ({}, {})", button, x, y);
                }
                
                let position = PhysicalPosition::new(x as i32, y as i32).to_logical(1.0);
                window.dispatch_event(WindowEvent::PointerMoved { position });
                window.dispatch_event(WindowEvent::PointerPressed {
                    position,
                    button: PointerEventButton::Left,
                });
            }
            TouchState::Pressed(last_x, last_y) if *last_x != x || *last_y != y => {
                let position = PhysicalPosition::new(x as i32, y as i32).to_logical(1.0);
                window.dispatch_event(WindowEvent::PointerMoved { position });
                *touch_state = TouchState::Pressed(x, y);
            }
            _ => {}
        }
    } else if let TouchState::Pressed(x, y) = touch_state {
        let position = PhysicalPosition::new(*x as i32, *y as i32).to_logical(1.0);
        window.dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
        window.dispatch_event(WindowEvent::PointerExited);
        *touch_state = TouchState::None;
    }
}

fn render_ui<DI: display_interface::WriteOnlyDataCommand>(
    window: &Rc<MinimalSoftwareWindow>,
    display_interface: &mut DI,
    buffer: &mut Vec<Rgb565Pixel>,
) {
    window.draw_if_needed(|renderer| {
        renderer.render(buffer, config::DISPLAY_WIDTH as usize);
        m5stack::transfer_buffer_to_display(
            display_interface,
            buffer,
            config::TRANSFER_CHUNK_SIZE,
        );
    });
}

fn main() {
    esp_idf_sys::link_patches();
    slint_platform::init_start_time();
    
    let p = Peripherals::take().unwrap();
    
    let mut i2c = I2cDriver::new(
        p.i2c0,
        p.pins.gpio21,
        p.pins.gpio22,
        &I2cConfig::new().baudrate(Hertz(config::I2C_BAUDRATE_HZ)),
    ).unwrap();
    
    m5stack::init_power(&mut i2c);
    
    let spi = SpiDriver::new(
        p.spi2,
        p.pins.gpio18,
        p.pins.gpio23,
        Some(p.pins.gpio38),
        &Default::default(),
    ).unwrap();
    
    let spi_device = SpiDeviceDriver::new(
        spi,
        Some(p.pins.gpio5),
        &Config::new().baudrate(Hertz(config::SPI_BAUDRATE_HZ)),
    ).unwrap();
    
    let dc_pin = PinDriver::output(p.pins.gpio15).unwrap();
    let mut display = SPIInterface::new(spi_device, dc_pin);
    
    m5stack::init_display(&mut display);
    
    let (platform, window) = slint_platform::M5StackPlatform::new(
        config::DISPLAY_WIDTH,
        config::DISPLAY_HEIGHT,
    );
    slint::platform::set_platform(platform).unwrap();
    
    let ui = HelloWorld::new().unwrap();
    ui.show().unwrap();
    
    let mut buffer = Vec::new();
    buffer.resize((config::DISPLAY_WIDTH * config::DISPLAY_HEIGHT) as usize, Rgb565Pixel(0));
    
    let mut touch_state = TouchState::None;
    
    loop {
        slint::platform::update_timers_and_animations();
        process_touch_events(&mut i2c, &window, &mut touch_state);
        render_ui(&window, &mut display, &mut buffer);
        FreeRtos::delay_ms(config::FRAME_TIME_MS);
    }
}

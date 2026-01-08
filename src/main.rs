extern crate alloc;

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
use m5stack::*;

slint::include_modules!();

#[derive(Debug, PartialEq)]
enum TouchState {
    None,
    Pressed(u16, u16),
}

struct AppState {
    led_enabled: bool,
    imu: Imu,
}

fn handle_button_press(
    button: &str,
    i2c: &mut I2cDriver,
    app_state: &mut AppState,
) {
    match button {
        "BtnA" => {
            println!("BtnA: Musical scale!");
            play_musical_scale(i2c);
        }
        "BtnB" => {
            app_state.led_enabled = !app_state.led_enabled;
            set_led(i2c, app_state.led_enabled);
            println!("BtnB: LED {}", if app_state.led_enabled { "ON" } else { "OFF" });
        }
        "BtnC" => {
            println!("BtnC: IMU & Battery");
            app_state.imu.print_stats(i2c);
            
            if let Some(voltage) = read_battery_voltage(i2c) {
                let percent = battery_percentage(voltage);
                println!("  Battery: {:.2}V ({}%)", voltage, percent);
            }
        }
        _ => {}
    }
}

fn play_musical_scale(i2c: &mut I2cDriver) {
    // Temporarily create audio, play scale, drop (releases GPIO2 for display)
    if let Ok(audio_pin) = PinDriver::output(unsafe { esp_idf_hal::gpio::Gpio2::new() }) {
        if let Ok(mut audio) = Audio::new(audio_pin, i2c) {
            audio.play_scale();
        }
    }
}

fn process_touch_events(
    i2c: &mut I2cDriver,
    window: &Rc<MinimalSoftwareWindow>,
    touch_state: &mut TouchState,
    app_state: &mut AppState,
) {
    if let Some((x, y)) = read_touch(i2c) {
        match touch_state {
            TouchState::None => {
                *touch_state = TouchState::Pressed(x, y);
                
                if let Some(button) = check_button_zone(x, y) {
                    println!("Button: {}", button);
                    handle_button_press(button, i2c, app_state);
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
        renderer.render(buffer, DISPLAY_WIDTH as usize);
        transfer_buffer_to_display(display_interface, buffer, DISPLAY_TRANSFER_CHUNK_SIZE);
    });
}

fn main() {
    esp_idf_sys::link_patches();
    slint_platform::init_start_time();
    
    let p = Peripherals::take().unwrap();
    
    // I2C for sensors and power management
    let mut i2c = I2cDriver::new(
        p.i2c0,
        p.pins.gpio21,
        p.pins.gpio22,
        &I2cConfig::new().baudrate(Hertz(I2C_BAUDRATE_HZ)),
    ).unwrap();
    
    // Initialize M5Stack Core2 hardware
    init_power(&mut i2c);
    init_led(&mut i2c);
    let imu_driver = Imu::new(&mut i2c);
    
    // SPI for display
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
        &Config::new().baudrate(Hertz(SPI_BAUDRATE_HZ)),
    ).unwrap();
    
    let dc_pin = PinDriver::output(p.pins.gpio15).unwrap();
    let mut display = SPIInterface::new(spi_device, dc_pin);
    init_display(&mut display);
    
    // Initialize Slint UI
    let (platform, window) = slint_platform::M5StackPlatform::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);
    slint::platform::set_platform(platform).unwrap();
    
    let ui = HelloWorld::new().unwrap();
    ui.show().unwrap();
    
    let mut buffer = Vec::new();
    buffer.resize((DISPLAY_WIDTH * DISPLAY_HEIGHT) as usize, Rgb565Pixel(0));
    
    let mut touch_state = TouchState::None;
    let mut app_state = AppState {
        led_enabled: false,
        imu: imu_driver,
    };
    
    // Main event loop
    loop {
        slint::platform::update_timers_and_animations();
        process_touch_events(&mut i2c, &window, &mut touch_state, &mut app_state);
        render_ui(&window, &mut display, &mut buffer);
        FreeRtos::delay_ms(FRAME_TIME_MS);
    }
}

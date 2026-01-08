# Rust + Slint on M5Stack Core2

A minimal working example of running Slint UI framework on M5Stack Core2 v1.1 using Rust.

## Hardware

- **Device**: M5Stack Core2 v1.1
- **MCU**: ESP32 (Xtensa)
- **Display**: ILI9342C, 320x240, SPI
- **Touch**: FT6336, I2C

## Prerequisites

Install ESP Rust toolchain:

```bash
cargo install espup
espup install
source $HOME/export-esp.sh
```

Install flash tools:

```bash
cargo install espflash ldproxy
```

## Build and Flash

```bash
cargo run
```

This will compile, flash, and monitor the device.

## Project Structure

- `src/main.rs` - Main application loop with touch event handling
- `src/config.rs` - Hardware configuration constants
- `src/m5stack.rs` - M5Stack Core2 hardware abstraction (display, touch, power)
- `src/slint_platform.rs` - Slint platform implementation for ESP32
- `src/imu.rs` - MPU6886 IMU (accelerometer, gyroscope, temperature) driver
- `src/axp192_led.rs` - AXP192 power management (LED control, battery monitoring)
- `ui/hello.slint` - Slint UI definition

## Features

- **Fast Slint UI rendering** (~200ms per frame)
- **Capacitive touch support** with FT6336 driver
- **Interactive UI elements** - tap the red circle to toggle background color
- **Button zones** - touch bottom area for button actions:
  - **Button A** (left): Prints message to console
  - **Button B** (center): Toggle LED on/off
  - **Button C** (right): Display IMU data & battery stats
- **IMU support** - MPU6886 accelerometer, gyroscope, and temperature sensor
- **Battery monitoring** - AXP192 voltage reading and percentage calculation
- **LED control** - AXP192 GPIO1 LED control

## Technical Notes

### Memory Constraints

ESP32 has limited RAM (~200KB usable). The implementation uses chunked buffer transfer to avoid memory allocation failures while maintaining fast rendering.

Chunk size can be adjusted in [`src/config.rs`](src/config.rs) - larger chunks render faster but use more memory.

### Rendering Performance

- SPI frequency: 26MHz (maximum stable for full-duplex)
- Full screen update: ~200ms
- Chunk size: 4KB (adjustable in config)

### Color Configuration

Display controller configured for BGR pixel order with inversion enabled. See [`src/m5stack.rs`](src/m5stack.rs) `init_display()` for details.

## License

MIT
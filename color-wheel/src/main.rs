#![no_std]
#![no_main]

use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm, Timer2Pwm};
use embedded_hal::digital::InputPin;
use embedded_hal::pwm::SetDutyCycle;
use panic_halt as _;

const COMMON_ANODE: bool = false;
const BLUETOOTH_BAUD: u32 = 9600;
const COMMAND_BUFFER_LEN: usize = 32;
const DEBUG_LOGS: bool = true;
const SOFT_SERIAL_BIT_US: u32 = 104;
const SOFT_SERIAL_HALF_BIT_US: u32 = 52;
const SOFT_SERIAL_IDLE_POLL_US: u32 = 20;

#[derive(Clone, Copy)]
struct RgbColor {
    r: u8,
    g: u8,
    b: u8,
}

enum BufferEvent {
    Incomplete,
    Parsed(RgbColor),
}

struct CommandBuffer {
    bytes: [u8; COMMAND_BUFFER_LEN],
    len: usize,
}

impl CommandBuffer {
    fn new() -> Self {
        Self {
            bytes: [0; COMMAND_BUFFER_LEN],
            len: 0,
        }
    }

    fn push(&mut self, byte: u8) -> BufferEvent  {
        match byte {
            b'\r' | b'\n' => {
                if self.len == 0 {
                    return BufferEvent::Incomplete;
                }

                let color = parse_rgb_command(&self.bytes[..self.len]);
                self.len = 0;
                match color {
                    Some(color) => BufferEvent::Parsed(color),
                    None => BufferEvent::Incomplete,
                }
            }
            8 | 127 => {
                if self.len > 0 {
                    self.len -= 1;
                }
                BufferEvent::Incomplete
            }
            _ => {
                if self.len < self.bytes.len() {
                    self.bytes[self.len] = byte;
                    self.len += 1;
                    BufferEvent::Incomplete
                } else {
                    self.len = 0;
                    BufferEvent::Incomplete
                }
            }
        }
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);
    let timer2 = Timer2Pwm::new(dp.TC2, Prescaler::Prescale64);

    let mut red = pins.d9.into_output().into_pwm(&timer1);
    let mut green = pins.d10.into_output().into_pwm(&timer1);
    let mut blue = pins.d11.into_output().into_pwm(&timer2);
    let mut bt_rx = pins.d3.into_pull_up_input();
    let _bt_tx = pins.d4.into_output_high();
    let mut serial = arduino_hal::default_serial!(dp, pins, BLUETOOTH_BAUD);

    red.enable();
    green.enable();
    blue.enable();

    let mut current_color = RgbColor { r: 255, g: 0, b: 0 };
    let mut command_buffer = CommandBuffer::new();
    let mut idle_ticks = 0u32;

    set_rgb(
        &mut red,
        &mut green,
        &mut blue,
        current_color.r,
        current_color.g,
        current_color.b,
    );
    log_startup(&mut serial);

    loop {
        if let Some(byte) = read_software_serial_byte(&mut bt_rx) {
            match command_buffer.push(byte) {
                BufferEvent::Incomplete => {}
                BufferEvent::Parsed(color) => {
                    current_color = color;
                    let _ = ufmt::uwriteln!(serial, "Changing Color to R:{} G:{} B:{}\r", current_color.r, current_color.g, current_color.b);
                    set_rgb(
                        &mut red,
                        &mut green,
                        &mut blue,
                        current_color.r,
                        current_color.g,
                        current_color.b,
                    );
                }
            }
            idle_ticks = 0;
        } else {
            arduino_hal::delay_us(SOFT_SERIAL_IDLE_POLL_US);
            idle_ticks = idle_ticks.saturating_add(1);
        }
    }
}

fn read_software_serial_byte<P>(rx: &mut P) -> Option<u8>
where
    P: InputPin,
{
    if rx.is_high().ok()? {
        return None;
    }

    arduino_hal::delay_us(SOFT_SERIAL_HALF_BIT_US);
    if rx.is_high().ok()? {
        return None;
    }

    let mut byte = 0u8;
    for bit in 0..8 {
        arduino_hal::delay_us(SOFT_SERIAL_BIT_US);
        if rx.is_high().ok()? {
            byte |= 1 << bit;
        }
    }

    arduino_hal::delay_us(SOFT_SERIAL_BIT_US);
    if rx.is_low().ok()? {
        return None;
    }

    Some(byte)
}

fn parse_rgb_command(bytes: &[u8]) -> Option<RgbColor> {
    if bytes.len() != 9 || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }

    Some(RgbColor {
        r: parse_component(&bytes[0..3])?,
        g: parse_component(&bytes[3..6])?,
        b: parse_component(&bytes[6..9])?,
    })
}

fn parse_component(bytes: &[u8]) -> Option<u8> {
    core::str::from_utf8(bytes).ok()?.parse::<u8>().ok()
}

fn set_rgb<R, G, B>(red: &mut R, green: &mut G, blue: &mut B, r: u8, g: u8, b: u8)
where
    R: SetDutyCycle,
    G: SetDutyCycle,
    B: SetDutyCycle,
{
    set_channel(red, r);
    set_channel(green, g);
    set_channel(blue, b);
}

fn set_channel<P>(pin: &mut P, value: u8)
where
    P: SetDutyCycle,
{
    let level = if COMMON_ANODE { 255 - value } else { value };
    let _ = pin.set_duty_cycle_fraction(level as u16, 255);
}

fn log_startup<S>(serial: &mut S)
where
    S: ufmt::uWrite,
{
    if !DEBUG_LOGS {
        return;
    }

    let _ = ufmt::uwriteln!(serial, "Bluetooth RGB debug ready\r");
}
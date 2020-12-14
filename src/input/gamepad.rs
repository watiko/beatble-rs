use std::sync::Arc;

use crossbeam::atomic::AtomicCell;
use eyre::{Result, WrapErr};
use log::{debug, error, info, trace};

use super::ble::{KeyInput, NormalButton, OptionButton};
use super::platform::linux::{Device, Event};

trait CodeExt {
    fn normal_button(self) -> Option<NormalButton>;
    fn option_button(self) -> Option<OptionButton>;
}

impl CodeExt for u8 {
    #[inline]
    fn normal_button(self) -> Option<NormalButton> {
        match self {
            0 => Some(NormalButton::B1),
            1 => Some(NormalButton::B2),
            2 => Some(NormalButton::B3),
            3 => Some(NormalButton::B4),
            4 => Some(NormalButton::B5),
            5 => Some(NormalButton::B6),
            6 => Some(NormalButton::B7),
            _ => None,
        }
    }

    #[inline]
    fn option_button(self) -> Option<OptionButton> {
        match self {
            8 => Some(OptionButton::E1),
            9 => Some(OptionButton::E2),
            10 => Some(OptionButton::E3),
            11 => Some(OptionButton::E4),
            _ => None,
        }
    }
}

#[inline]
fn convert_scratch(value: i16) -> u8 {
    // sensitivity is doubled
    (((((value >> 8) as u8) as u16) * 2) % 0xFF) as u8
}

pub fn create_input_handler(input: &str) -> Result<Arc<AtomicCell<KeyInput>>> {
    debug!(
        "AtomicCell::<KeyInput>::is_lock_free: {}",
        AtomicCell::<KeyInput>::is_lock_free()
    );
    let atomic_key_input = Arc::new(AtomicCell::new(KeyInput::init()));

    let mut device = Device::open(input).context(format!("no gamepad found: {}", input))?;
    info!("connected to {}", input);

    {
        let atomic_key_input = Arc::clone(&atomic_key_input);
        tokio::task::spawn_blocking(move || {
            info!("input handler watching input event");
            let mut key_input = KeyInput::init();
            'e: loop {
                while let Some(event) = device.next() {
                    match event {
                        Event::Disconnected => {
                            error!("controller disconnected");
                            break 'e;
                        }
                        Event::Error(e) => {
                            error!("unknown error: {}", e);
                            break 'e;
                        }
                        Event::ButtonPressed(_)
                        | Event::ButtonReleased(_)
                        | Event::AxisChanged(_, _) => {
                            trace!("event: {:?}", event);
                            update_key_input(&mut key_input, event);
                            trace!("key_input: {:?}", key_input);
                            atomic_key_input.store(key_input);
                        }
                    }
                }
            }
            panic!("input handler exiting");
        });
    };

    Ok(atomic_key_input)
}

#[inline]
fn update_key_input(key_input: &mut KeyInput, event: Event) {
    match event {
        Event::ButtonPressed(button) => {
            if let Some(button) = button.normal_button() {
                key_input.normal_button.insert(button);
            }
            if let Some(button) = button.option_button() {
                key_input.option_button.insert(button);
            }
        }
        Event::ButtonReleased(button) => {
            if let Some(button) = button.normal_button() {
                key_input.normal_button.remove(button);
            }
            if let Some(button) = button.option_button() {
                key_input.option_button.remove(button);
            }
        }
        Event::AxisChanged(_axis, value) => {
            key_input.scratch = convert_scratch(value);
        }
        Event::Disconnected | Event::Error(_) => unreachable!(),
    };
}

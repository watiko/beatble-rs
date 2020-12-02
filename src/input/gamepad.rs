use std::sync::Arc;

use crossbeam::atomic::AtomicCell;
use gilrs::{Event, EventType, GilrsBuilder};
use gilrs::ev::Button;
use log::{debug, error, info, trace};

use super::ble::{KeyInput, NormalButton, OptionButton};

const ENTRY_MODEL_MAPPING: &str = "03000000cf1c00001810000011010000,Konami Amusement beatmania IIDX controller Entry Model,platform:Linux,a:b0,b:b1,y:b2,x:b3,leftshoulder:b4,lefttrigger:b5,rightshoulder:b6,back:b8,start:b9,leftx:a0";
const PHOENIXWAN_MAPPING: &str = "03000000cf1c00004880000010010000,PowerA Controller INF&BMS,platform:Linux,platform:Linux,a:b0,b:b1,y:b2,x:b3,leftshoulder:b4,lefttrigger:b5,rightshoulder:b6,back:b8,start:b9,guide:b10,righttrigger:b11,leftx:a0";

trait CodeExt {
    fn normal_button(self) -> Option<NormalButton>;
    fn option_button(self) -> Option<OptionButton>;
}

impl CodeExt for Button {
    fn normal_button(self) -> Option<NormalButton> {
        match self {
            Button::South => Some(NormalButton::B1),
            Button::East => Some(NormalButton::B2),
            Button::North => Some(NormalButton::B3),
            Button::West => Some(NormalButton::B4),
            Button::LeftTrigger => Some(NormalButton::B5),
            Button::LeftTrigger2 => Some(NormalButton::B6),
            Button::RightTrigger => Some(NormalButton::B7),
            _ => None,
        }
    }

    fn option_button(self) -> Option<OptionButton> {
        match self {
            Button::Select => Some(OptionButton::E1),
            Button::Start => Some(OptionButton::E2),
            Button::Mode => Some(OptionButton::E3),
            Button::RightTrigger2 => Some(OptionButton::E4),
            _ => None,
        }
    }
}

pub fn create_input_handler() -> Result<Arc<AtomicCell<KeyInput>>, Box<dyn std::error::Error>> {
    debug!("AtomicCell::<KeyInput>::is_lock_free: {}", AtomicCell::<KeyInput>::is_lock_free());
    let atomic_key_input = Arc::new(AtomicCell::new(KeyInput::init()));

    {
        let atomic_key_input = Arc::clone(&atomic_key_input);
        let input_handler = std::thread::spawn(move || {
            info!("input handler spawned");
            let mut gilrs = GilrsBuilder::new()
                .with_default_filters(false)
                .add_env_mappings(true)
                .add_mappings(ENTRY_MODEL_MAPPING)
                .add_mappings(PHOENIXWAN_MAPPING)
                .build()
                .expect("failed to create gilrs instance");

            for (id, gamepad) in gilrs.gamepads() {
                debug!("founded gamepad: id({}), name({})", id, gamepad.name());
            }

            // TODO: make selectable
            let (gamepad_id, gamepad) = gilrs.gamepads().next().expect("no gamepad detected");
            info!("connected gamepad name: {}", gamepad.name());
            let mut key_input = KeyInput::init();

            info!("input handler watching input event");
            loop {
                while let Some(Event { id, event, time: _ }) = gilrs.next_event() {
                    if id != gamepad_id {
                        // filter
                        continue;
                    };

                    trace!("event: {:?}", event);
                    match event {
                        EventType::ButtonPressed(button, _code) => {
                            if let Some(button) = button.normal_button() {
                                key_input.normal_button.insert(button);
                            }
                            if let Some(button) = button.option_button() {
                                key_input.option_button.insert(button);
                            }
                        }
                        EventType::ButtonReleased(button, _code) => {
                            if let Some(button) = button.normal_button() {
                                key_input.normal_button.remove(button);
                            }
                            if let Some(button) = button.option_button() {
                                key_input.option_button.remove(button);
                            }
                        }
                        EventType::AxisChanged(_axis, value, _code) => {
                            let scratch = ((value + 1.0) * 128.0) as u8;
                            key_input.scratch = scratch;
                        }
                        EventType::ButtonChanged(_, _, _) | EventType::ButtonRepeated(_, _) => {
                            // ignore
                        }
                        EventType::Connected => {
                            // ignore
                        }
                        EventType::Disconnected | EventType::Dropped => {
                            error!("controller disconnected/dropped: {:?}", event);
                        }
                    };

                    trace!("key_input: {:?}", key_input);
                    atomic_key_input.store(key_input);
                }
            }
        });

        tokio::spawn(async move {
            input_handler.join().expect("input handler exited");
        });
    }

    Ok(atomic_key_input)
}

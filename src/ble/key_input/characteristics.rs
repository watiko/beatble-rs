use std::collections::HashSet;
use std::sync::{Arc, atomic};
use std::thread;

use bluster::{
    gatt::{
        characteristic::{Characteristic, Properties},
        descriptor::Descriptor,
        event::Event,
    },
    SdpShortUuid,
};
use crossbeam::atomic::AtomicCell;
use futures::channel::mpsc::channel;
use futures::StreamExt;
use log::{debug, info, trace};
use tokio::time::Duration;
use uuid::Uuid;

use crate::input::KeyInput;

const CHARACTERISTIC_UUID: u16 = 0xFF01;
const SLEEP_DURATION: Duration = Duration::from_millis(8);

pub fn create_key_input_characteristic(
    key_input: Arc<AtomicCell<KeyInput>>,
    descriptors: HashSet<Descriptor>,
) -> Characteristic {
    debug!("create_key_input_characteristic");

    let (sender, receiver) = channel(1);

    let characteristic_handler = async move {
        debug!("create_key_input_characteristic: handler spawned");
        let notifying = Arc::new(atomic::AtomicBool::new(false));
        let mut rx = receiver;
        while let Some(event) = rx.next().await {
            match event {
                Event::NotifySubscribe(notify_subscribe) => {
                    info!("notify request to UUID({}) received", CHARACTERISTIC_UUID);
                    let notifying = Arc::clone(&notifying);
                    notifying.store(true, atomic::Ordering::Relaxed);

                    let mut counter: u16 = 0;
                    let key_input = Arc::clone(&key_input);
                    thread::spawn(move || {
                        loop {
                            if !(&notifying).load(atomic::Ordering::Relaxed) {
                                break;
                            };

                            let payload =
                                { key_input.load().to_payload((counter & 0xFF) as u8) };
                            trace!("payload: {:?}", payload);

                            notify_subscribe
                                .clone()
                                .notification
                                .try_send(payload.to_vec())
                                .unwrap();

                            counter = (counter + 2) & 0xFF;
                            thread::sleep(SLEEP_DURATION);
                        }
                        debug!("key input handler finished");
                    });
                }
                Event::NotifyUnsubscribe => {
                    info!(
                        "unsubscribe request to UUID({}) received",
                        CHARACTERISTIC_UUID
                    );
                    notifying.store(false, atomic::Ordering::Relaxed);
                }
                _ => {
                    info!(
                        "unimplemented event detected on key input characteristics: {:?}",
                        event
                    );
                }
            }
        }
    };

    tokio::spawn(characteristic_handler);

    Characteristic::new(
        Uuid::from_sdp_short_uuid(CHARACTERISTIC_UUID),
        Properties::new(None, None, Some(sender), None),
        None,
        descriptors,
    )
}

use std::os::unix::io::RawFd;

use bitflags::bitflags;
use eyre::Result;
use nix::errno::Errno;
use nix::{fcntl, unistd};
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Event {
    ButtonPressed(u8),
    ButtonReleased(u8),
    AxisChanged(u8, i16),
    Disconnected,
    Error(String),
}

#[derive(Debug, Error)]
pub enum OpenError {
    #[error("DeviceFileNotFound: {0}")]
    DeviceFileNotFound(String),
    #[error("PermissionDenied: {0}")]
    PermissionDenied(String),
    #[error("InvalidPath")]
    InvalidPath,
    #[error("Unknown: {0}")]
    Unknown(eyre::Report),
}

bitflags! {
    struct EventType: u8 {
        const BUTTON = 0x01;
        const AXIS = 0x02;
        const INIT = 0x80;
    }
}

#[repr(C)]
struct RawEvent {
    time: u32,
    value: i16,
    typ: EventType,
    number: u8,
}

impl From<RawEvent> for Option<Event> {
    #[inline]
    fn from(ev: RawEvent) -> Self {
        if ev.typ.contains(EventType::INIT) {
            // ignore init event
            return None;
        }
        match ev.typ {
            EventType::BUTTON => {
                if ev.value == 0 {
                    Some(Event::ButtonReleased(ev.number))
                } else {
                    Some(Event::ButtonPressed(ev.number))
                }
            }
            EventType::AXIS => Some(Event::AxisChanged(ev.number, ev.value)),
            _ => unreachable!(),
        }
    }
}

pub struct Device(RawFd);

impl Device {
    pub fn open(path: &str) -> Result<Self> {
        // mode is dummy
        let fd = fcntl::open(path, fcntl::OFlag::O_RDONLY, nix::sys::stat::Mode::S_IRUSR).map_err(
            |err| {
                use nix::Error;
                use OpenError::*;

                match err {
                    Error::Sys(Errno::ENOENT) => DeviceFileNotFound(path.to_string()),
                    Error::Sys(Errno::EPERM) => PermissionDenied(path.to_string()),
                    Error::InvalidPath | Error::InvalidUtf8 => InvalidPath,
                    e => Unknown(e.into()),
                }
            },
        )?;

        Ok(Device(fd))
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unistd::close(self.0).unwrap();
    }
}

impl Iterator for Device {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Event> {
        let mut buf = [0u8; 8];
        match unistd::read(self.0, &mut buf) {
            Ok(_) => {
                let raw_ev = unsafe { std::mem::transmute::<[u8; 8], RawEvent>(buf) };
                raw_ev.into()
            }
            Err(nix::Error::Sys(Errno::ENODEV)) => Some(Event::Disconnected),
            Err(e) => Some(Event::Error(format!("read error: {}", e))),
        }
    }
}

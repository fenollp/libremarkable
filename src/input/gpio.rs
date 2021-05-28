use super::ecodes;
use crate::input::{InputDeviceState, InputEvent};
use evdev::EventType;
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PhysicalButton {
    LEFT,
    MIDDLE,
    RIGHT,
    POWER,
    WAKEUP,
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GPIOEvent {
    Press { button: PhysicalButton },
    Unpress { button: PhysicalButton },
    Unknown,
}

pub struct GPIOState {
    states: [AtomicBool; 5],
}

impl ::std::default::Default for GPIOState {
    fn default() -> Self {
        GPIOState {
            states: [
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
                AtomicBool::new(false),
            ],
        }
    }
}

pub fn decode(ev: &evdev::InputEvent, outer_state: &InputDeviceState) -> Option<InputEvent> {
    let state = match outer_state {
        InputDeviceState::GPIOState(ref state_arc) => state_arc,
        _ => unreachable!(),
    };
    match ev.event_type() {
        EventType::SYNCHRONIZATION => {
            /* safely ignored. sync event*/
            None
        }
        EventType::KEY => {
            let nonzero = ev.value() != 0;
            let p = match ev.code() {
                ecodes::KEY_HOME => {
                    state.states[0].store(nonzero, Ordering::Relaxed);
                    PhysicalButton::MIDDLE
                }
                ecodes::KEY_LEFT => {
                    state.states[1].store(nonzero, Ordering::Relaxed);
                    PhysicalButton::LEFT
                }
                ecodes::KEY_RIGHT => {
                    state.states[2].store(nonzero, Ordering::Relaxed);
                    PhysicalButton::RIGHT
                }
                ecodes::KEY_POWER => {
                    state.states[3].store(nonzero, Ordering::Relaxed);
                    PhysicalButton::POWER
                }
                ecodes::KEY_WAKEUP => {
                    state.states[4].store(nonzero, Ordering::Relaxed);
                    PhysicalButton::WAKEUP
                }
                _ => return None,
            };

            let event = if nonzero {
                GPIOEvent::Press { button: p }
            } else {
                GPIOEvent::Unpress { button: p }
            };
            Some(InputEvent::GPIO { event })
        }
        _ty => {
            // Shouldn't happen
            error!("Unknown event on PhysicalButtonHandler (type: {:?})", _ty);
            None
        }
    }
}

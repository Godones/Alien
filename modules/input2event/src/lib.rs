#![cfg_attr(not(test), no_std)]


use slint::LogicalPosition;
use slint::platform::{PointerEventButton, WindowEvent};

use virtio_input_decoder::{Decoder, DecodeType, Key, KeyType, Mouse};

pub fn u64_to_decoder(event: u64) -> Result<DecodeType, ()> {
    let dtype = (event >> 48) as usize;
    let code = (event >> 32) & 0xffff;
    let val = (event & 0xffffffff) as i32;
    let decoder = Decoder::decode(dtype, code as usize, val as isize);
    decoder
}


/// virtio_input_event -> WindowEvent
///
/// virtio_input_event:
///
/// \[type:code:value]\[16:16:32]
pub fn input2event(event: u64, x: &mut i32, y: &mut i32) -> Result<WindowEvent, ()> {
    let decoder = u64_to_decoder(event)?;
    let event = match decoder {
        DecodeType::Key(key, key_type) => {
            match key_type {
                KeyType::Press => {
                    match key {
                        Key::MouseLeft => {
                            let event = WindowEvent::PointerPressed {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Left,
                            };
                            Ok(event)
                        }
                        Key::MouseRight => {
                            let event = WindowEvent::PointerPressed {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Right,
                            };
                            Ok(event)
                        }
                        Key::MouseMid => {
                            let event = WindowEvent::PointerPressed {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Middle,
                            };
                            Ok(event)
                        }
                        Key::MouseScrollDown | Key::MouseScrollUp => {
                            let event = WindowEvent::PointerPressed {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Other,
                            };
                            Ok(event)
                        }
                        _ => {
                            // WindowEvent::KeyPressed {
                            //     text: slint::platform::Key::
                            // };
                            Err(())
                        }
                    }
                }
                KeyType::Release => {
                    match key {
                        Key::MouseLeft => {
                            let event = WindowEvent::PointerReleased {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Left,
                            };
                            Ok(event)
                        }
                        Key::MouseRight => {
                            let event = WindowEvent::PointerReleased {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Right,
                            };
                            Ok(event)
                        }
                        Key::MouseMid => {
                            let event = WindowEvent::PointerReleased {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Middle,
                            };
                            Ok(event)
                        }
                        Key::MouseScrollDown | Key::MouseScrollUp => {
                            let event = WindowEvent::PointerReleased {
                                position: LogicalPosition::new(*x as f32, *y as f32),
                                button: PointerEventButton::Other,
                            };
                            Ok(event)
                        }
                        _ => {
                            Err(())
                        }
                    }
                }
            }
        }
        DecodeType::Mouse(mouse) => {
            match mouse {
                Mouse::X(rel_x) => {
                    *x += rel_x as i32;
                    if *x < 0 {
                        *x = 0;
                    }
                    let event = WindowEvent::PointerMoved {
                        position: LogicalPosition::new(*x as f32, *y as f32),
                    };
                    Ok(event)
                }
                Mouse::Y(rel_y) => {
                    *y += rel_y as i32;
                    if *y < 0 {
                        *y = 0;
                    }
                    let event = WindowEvent::PointerMoved {
                        position: LogicalPosition::new(*x as f32, *y as f32),
                    };
                    Ok(event)
                }
                Mouse::ScrollDown => {
                    let event = WindowEvent::PointerScrolled {
                        position: LogicalPosition::new(*x as f32, *y as f32),
                        delta_x: 1.0,
                        delta_y: 1.0,
                    };
                    Ok(event)
                }
                Mouse::ScrollUp => {
                    let event = WindowEvent::PointerScrolled {
                        position: LogicalPosition::new(*x as f32, *y as f32),
                        delta_x: -1.0,
                        delta_y: -1.0,
                    };
                    Ok(event)
                }
            }
        }
    };
    event
}
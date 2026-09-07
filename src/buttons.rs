use rppal::gpio::{Gpio, InputPin, Level};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::Result;

// Button GPIO pin numbers
pub const PIN_PAGE_FORWARD: u8 = 4;
pub const PIN_PAGE_BACK:    u8 = 5;
pub const PIN_BOOK_1:       u8 = 6;
pub const PIN_BOOK_2:       u8 = 12;
pub const PIN_BOOK_3:       u8 = 13;
pub const PIN_BOOK_4:       u8 = 16;

pub const LONG_PRESS_MS: u128 = 1000;
pub const DEBOUNCE_MS:   u128 = 200;

// Button press types
#[derive(Debug, PartialEq)]
pub enum ButtonPress {
    Short,
    Long,
}

// Button event - which button and how it was pressed
#[derive(Debug)]
pub struct ButtonEvent {
    pub pin:   u8,
    pub press: ButtonPress,
}

pub struct Buttons {
    pins:            Vec<InputPin>,
    last_press_time: HashMap<u8, Instant>,
}

impl Buttons {
    pub fn new() -> Result<Self> {
        let gpio = Gpio::new()?;
        
        let pin_numbers = vec![
            PIN_PAGE_FORWARD,
            PIN_PAGE_BACK,
            PIN_BOOK_1,
            PIN_BOOK_2,
            PIN_BOOK_3,
            PIN_BOOK_4,
        ];
        
        // Set up each pin as input with pull-up resistor
        let mut pins = Vec::new();
        for pin_num in pin_numbers {
            let pin = gpio.get(pin_num)?.into_input_pullup();
            pins.push(pin);
        }
        
        Ok(Buttons {
            pins,
            last_press_time: HashMap::new(),
        })
    }
    
    pub fn poll(&mut self) -> Option<ButtonEvent> {
        for pin in &self.pins {
            if pin.read() == Level::Low {
                let pin_num = pin.pin();
                
                // Check debounce
                let now = Instant::now();
                let last = self.last_press_time.get(&pin_num);
                
                let debounced = match last {
                    Some(t) => now.duration_since(*t).as_millis() > DEBOUNCE_MS,
                    None    => true,
                };
                
                if debounced {
                    self.last_press_time.insert(pin_num, now);
                    
                    // Measure press duration
                    let press_start = Instant::now();
                    while pin.read() == Level::Low {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    let duration = press_start.elapsed().as_millis();
                    
                    let press = if duration >= LONG_PRESS_MS {
                        ButtonPress::Long
                    } else {
                        ButtonPress::Short
                    };
                    
                    return Some(ButtonEvent { pin: pin_num, press });
                }
            }
        }
        None
    }
}
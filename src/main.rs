#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::InputPin;

#[rustfmt::skip]
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{
        timer::Timer,
    },
};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn init() -> ! {
    rtt_init_print!();
    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display_board: [[u8; 5]; 5] = [[1; 5]; 5];
    let mut display = Display::new(board.display_pins);

    let mut button_a = board.buttons.button_a;
    let mut button_b = board.buttons.button_b;

    let mut mode: bool = false;

    rprintln!("Level Started");
    loop {
        if button_a.is_low().unwrap() {
            rprintln!("Course Mode");
            mode = false;
        } else if button_b.is_low().unwrap() {
            rprintln!("Fine Mode");
            mode = true;
        }

        display.show(&mut timer, display_board, 200);
    }
}

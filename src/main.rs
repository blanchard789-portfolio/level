#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embedded_hal::digital::v2::InputPin;
use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};
use microbit::{display::blocking::Display, hal::Timer};
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn init() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display_board: [[u8; 5]; 5] = [[1; 5]; 5];
    let mut display = Display::new(board.display_pins);

    // makes calling button methods easier throughout the program
    let button_a = board.buttons.button_a;
    let button_b = board.buttons.button_b;

    // Responsible for storing current mode selection (true = fine, false = course)
    let mut mode: bool = false;

    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();

    rprintln!("Level Started");
    loop {
        /*
            Checks microbit button inputs for mode selection.
            A button press (by it self): Course Mode (sets mode to false)
            B button press (bt it self): Fine Mode (sets mode to true)
            Combined button presses: Ignored
        */
        if button_a.is_low().unwrap() && button_b.is_high().unwrap() {
            mode = false;
        } else if button_b.is_low().unwrap() && button_a.is_high().unwrap() {
            mode = true;
        }

        // Gets accelerometer readings from microbit
        if sensor.accel_status().unwrap().xyz_new_data {
            let data = sensor.accel_data().unwrap();
            rprintln!(
                "Acceleration: x {} y {} z {} fine {}",
                data.x,
                data.y,
                data.z,
                mode
            );

            /*
                If board is upside down (LED matrix pointing towards the ground), screen is cleared
                and no new pixels are displayed until the board is righted.

                If the board is right side up, prior pixel is cleared and data is passed to the screen_writer
                function, which plots the next pixel.
            */
            if data.z > 0 {
                screen_zero(&mut display_board);
            } else {
                screen_zero(&mut display_board);
                screen_writer(&mut display_board, data.x, data.y, mode);
            }
        }

        // Updates display matrix LEDs every tick (in this case every 200ms), is blocking.
        display.show(&mut timer, display_board, 200);
    }
}

/*
    fn screen_zero(board_in: &mut [[u8; 5]; 5])

    Turns off all the LEDs on the microbit matrix, used for clearing display between program ticks.
*/
fn screen_zero(board_in: &mut [[u8; 5]; 5]) {
    #[allow(clippy::needless_range_loop)]
    for i in 0..5 {
        for j in 0..5 {
            board_in[i][j] = 0;
        }
    }
}

/*
    fn screen_writer(board_in: &mut [[u8; 5]; 5], x: i32, y: i32, mode: bool)

    Takes screen matrix, accelerometer readings, and current mode and finds the respective pixel to plot to.
    In both modes there are five value ranges that the x and y values can map to, since we have a 5x5 LED matrix to work with.
    In course mode that covers a total range of -500 to 500, so to light up the center pixel for example, both x and y must be in the range of -249 to 249.
    In fine mode the scale is 1/10th that, so -50 to 50, otherwise it works exactly the same.

*/
fn screen_writer(board_in: &mut [[u8; 5]; 5], x: i32, y: i32, mode: bool) {
    let mut xr = 0;

    // Multiplier value, used to scale groupings depending on mode.
    // mult = 1: Course (normal scale), mult = 10: Fine (1/10 scale)
    let mut mult = 1;

    if mode {
        mult = 10;
    }

    // Plots x value
    if x <= -500 / mult {
        xr = 4
    } else if (-499 / mult..=-250 / mult).contains(&x) {
        xr = 3
    } else if (-249 / mult..250 / mult).contains(&x) {
        xr = 2
    } else if (250 / mult..500 / mult).contains(&x) {
        xr = 1
    } else if x >= 500 / mult {
        xr = 0
    }

    // Plots y value
    let mut yr = 0;
    if y <= -500 / mult {
        yr = 0
    } else if (-499 / mult..=-250 / mult).contains(&y) {
        yr = 1
    } else if (-249 / mult..250 / mult).contains(&y) {
        yr = 2
    } else if (250 / mult..500 / mult).contains(&y) {
        yr = 3
    } else if y >= 500 / mult {
        yr = 4
    }

    // Sets LED pixel to be displayed this tick, based on above plots.
    board_in[yr][xr] = 1;
}

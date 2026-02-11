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
            rprintln!("Course Mode");
            mode = false;
        } else if button_b.is_low().unwrap() && button_a.is_high().unwrap() {
            rprintln!("Fine Mode");
            mode = true;
        }

        if sensor.accel_status().unwrap().xyz_new_data {
            let data = sensor.accel_data().unwrap();
            // RTT instead of normal print
            rprintln!("Acceleration: x {} y {} z {}", data.x, data.y, data.z);
            if data.z > 0 {
                screen_zero(&mut display_board);
            } else {
                screen_zero(&mut display_board);
                if !mode {
                    course(&mut display_board, data.x, data.y);
                } else {
                    fine(&mut display_board, data.x, data.y);
                }
            }
        }

        // Updates display matrix LEDs every tick (in this case every 200ms), is blocking.
        display.show(&mut timer, display_board, 200);
    }
}

fn screen_zero(board_in: &mut [[u8; 5]; 5]) {
    #[allow(clippy::needless_range_loop)]
    for i in 0..5 {
        for j in 0..5 {
            board_in[i][j] = 0;
        }
    }
}

fn course(board_in: &mut [[u8; 5]; 5], x: i32, y: i32) {
    let mut xr = 0;
    if x <= -500 {
        xr = 4
    } else if (-499..=-250).contains(&x) {
        xr = 3
    } else if (-249..250).contains(&x) {
        xr = 2
    } else if (250..500).contains(&x) {
        xr = 1
    } else if x >= 500 {
        xr = 0
    }

    let mut yr = 0;
    if y <= -500 {
        yr = 0
    } else if (-499..=-250).contains(&y) {
        yr = 1
    } else if (-249..250).contains(&y) {
        yr = 2
    } else if (250..500).contains(&y) {
        yr = 3
    } else if y >= 500 {
        yr = 4
    }
    board_in[yr][xr] = 1;
}

fn fine(board_in: &mut [[u8; 5]; 5], x: i32, y: i32) {
    let mut xr = 0;
    if x <= -50 {
        xr = 4
    } else if (-49..=-25).contains(&x) {
        xr = 3
    } else if (-24..25).contains(&x) {
        xr = 2
    } else if (25..50).contains(&x) {
        xr = 1
    } else if x >= 50 {
        xr = 0
    }

    let mut yr = 0;
    if y <= -50 {
        yr = 0
    } else if (-49..=-25).contains(&y) {
        yr = 1
    } else if (-24..25).contains(&y) {
        yr = 2
    } else if (-25..50).contains(&y) {
        yr = 3
    } else if y >= 50 {
        yr = 4
    }
    board_in[yr][xr] = 1;
}

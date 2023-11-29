use std::time::Duration;

use rppal::gpio::Gpio;

fn main() -> anyhow::Result<()> {
    let gpio = Gpio::new()?;
    let mut pin = gpio.get(26)?.into_output_low();
    let secs = 60.0 / 300.0 / 2.;
    loop {
        pin.set_high();
        std::thread::sleep(Duration::from_secs_f64(secs));
        pin.set_low();
        std::thread::sleep(Duration::from_secs_f64(secs));
    }
    // Ok(())
}

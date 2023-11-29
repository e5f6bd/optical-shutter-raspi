use std::{
    collections::BTreeSet,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use anyhow::{bail, Context};
use log::{error, info};
use rppal::gpio::{Gpio, OutputPin};
use serde::Deserialize;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let config: Config = toml::from_str(&fs_err::read_to_string("config.toml")?)?;

    let gpio = Gpio::new()?;
    let mut shutters = vec![];
    let mut used_pins = BTreeSet::new();
    for shutter in config.shutters {
        for pin in [shutter.pin_open, shutter.pin_close] {
            if !used_pins.insert(pin) {
                bail!("Duplicating pin: {pin}")
            }
        }
        shutters.push(Shutter {
            pin_open: gpio.get(shutter.pin_open)?.into_output_low(),
            pin_close: gpio.get(shutter.pin_close)?.into_output_low(),
        })
    }

    let listener = TcpListener::bind(("0.0.0.0", config.port))?;
    for reader in listener.incoming() {
        info!("Connection opened");
        if let Err(e) = handle(&mut shutters, reader) {
            error!("Error: {e}");
        }
        info!("Connection closed");
    }

    Ok(())
}

fn handle(shutters: &mut [Shutter], reader: std::io::Result<TcpStream>) -> anyhow::Result<()> {
    let mut stream = reader?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buffer = String::new();
    while reader.read_line(&mut buffer)? > 0 {
        info!("Read: {buffer:?}");
        if let Err(e) = handle_command(shutters, &buffer) {
            error!("Error: {e}");
            writeln!(stream, "Error: {e}")?;
        }
        buffer.clear();
    }
    Ok(())
}

fn handle_command(shutters: &mut [Shutter], command: &str) -> anyhow::Result<()> {
    let command: Command = serde_json::from_str(command)?;
    info!("Command: {command:?}");

    let Command::Shutter(id, open_close) = command;
    let shutter = shutters
        .get_mut(id)
        .with_context(|| format!("Invalid shutter ID: {id}"))?;

    use rppal::gpio::Level::*;
    let pin = match open_close {
        OpenClose::Open => &mut shutter.pin_open,
        OpenClose::Close => &mut shutter.pin_close,
    };
    pin.write(High);
    std::thread::sleep(Duration::from_secs_f64(0.1));
    for pin in [&mut shutter.pin_open, &mut shutter.pin_close] {
        pin.write(Low);
    }
    std::thread::sleep(Duration::from_secs_f64(0.1));

    Ok(())
}

#[derive(Deserialize)]
struct Config {
    port: u16,
    shutters: Vec<ShutterConfig>,
}
#[derive(Deserialize)]
struct ShutterConfig {
    pin_open: u8,
    pin_close: u8,
}

struct Shutter {
    pin_open: OutputPin,
    pin_close: OutputPin,
}

#[derive(Debug, Deserialize)]
enum Command {
    Shutter(usize, OpenClose),
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Deserialize)]
enum OpenClose {
    Open,
    Close,
}

#[macro_use]
extern crate clap;
extern crate chrono;
extern crate serial;

use chrono::Local;
use clap::{App, Arg};
use serde::{Deserialize, Serialize};
use serial::prelude::*;
use std::time::Duration;
use std::{thread, time};

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate: serial::Baud9600,
    char_size: serial::Bits8,
    parity: serial::ParityNone,
    stop_bits: serial::Stop1,
    flow_control: serial::FlowNone,
};

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    date: String,
    pm10: f64,
    pm25: f64,
    comment: String,
}

fn main() {
    let matches = App::new("AirStation")
        .version("0.1.0")
        .author("Alexandre Peyroux <alex@px.io>")
        .about("Measures air quality via SDS011.")
        .arg(
            Arg::with_name("serial")
                .short("s")
                .long("serial")
                .value_name("SERIAL")
                .default_value("/dev/ttyUSB0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("comment")
                .short("c")
                .long("comment")
                .value_name("COMMENT")
                .help("A comment for the report.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("duration")
                .short("d")
                .long("duration")
                .value_name("DURATION")
                .help("Time between 2 measures.")
                .default_value("30")
                .takes_value(true),
        )
        .get_matches();

    let serial = matches.value_of("serial").unwrap_or("/dev/ttyUSB0");
    let comment = matches.value_of("comment").unwrap_or("");
    let duration = value_t!(matches.value_of("duration"), u64).unwrap_or_else(|e| e.exit());
    let mut port = match serial::open(&serial) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Error when opening the serial port: {:?}", err);
            std::process::exit(1);
        }
    };
    let mut buf: Vec<u8> = (0..255).collect();
    port.configure(&SETTINGS).unwrap();
    port.set_timeout(Duration::from_secs(5)).unwrap();

    let report_file = std::fs::File::create(format!(
        "report-{}.csv",
        Local::now().format("%d%m%Y-%H%M%S").to_string()
    ))
    .unwrap();
    let mut wtr = csv::Writer::from_writer(report_file);
    
    loop {
        let now = Local::now().format("%d/%m/%Y %H:%M:%S");
        (&mut port as &mut SerialPort).read(&mut buf[..]).unwrap();
        let pm25 = ((buf[3] as f64) * 256.0 + (buf[2] as f64)) / 10.0;
        let pm10 = ((buf[5] as f64) * 256.0 + (buf[4] as f64)) / 10.0;
        if buf[0] != 170 || buf[1] != 192 || buf[9] != 171 {
            println!("Problem with the sensor.");
            break;
        }
        if u16::from(buf[8]) != buf[2..8].into_iter().map(|x| u16::from(*x)).sum::<u16>() % 256 {
            println!("Checksum is not okay.");
            break;
        }
        let r = Record {
            pm10: pm10,
            pm25: pm25,
            comment: comment.to_string(),
            date: now.to_string(),
        };
        wtr.serialize(r).unwrap();
        wtr.flush().unwrap();
        println!("==== {} ====", now.to_string());
        println!("pm10: {} μg/m³", pm10);
        println!("pm25: {} μg/m³", pm25);
        thread::sleep(time::Duration::from_secs(duration));
    }
}

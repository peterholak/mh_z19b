use mh_z19b::mhz19b;
use std::time::Duration;
use std::thread::sleep;

fn main() {
    let port_name = "/dev/ttyAMA0";

    // On Windows with a USB to serial adapter.
    // let port_name = "COM4";

    let mut port = mhz19b::serial::connect(port_name, Duration::from_secs(5))
        .expect("Failed to connect.");

    println!("Connected.");

    if let Err(e) = mhz19b::set_auto_correction(&mut port, false) {
        eprintln!("Failed to disable auto-correction: {}", e);
    }

    for _ in 0..5 {
        match mhz19b::read_co2(&mut port) {
            Ok(co2) => println!("CO2 level: {}", co2),
            Err(e) => eprintln!("{}", e)
        };
        sleep(Duration::from_secs(3));
    }
}

use sensor_simulator::prelude::*;
use std::time::Duration;

fn main() {
    println!("Testing SimulatedDataSource toggle logic...");
    let ds = SimulatedDataSource::new();

    println!("Reading for 25 seconds to see toggle...");
    for i in 0..250 {
        std::thread::sleep(Duration::from_millis(100));
        let (ad1, ad2, ad3, di0, di1) = ds.read_all().unwrap();
        if i % 10 == 0 {
            // Print every second
            println!("[{:.1}s] di0={}, di1={}", i as f64 * 0.1, di0, di1);
        }
    }
}

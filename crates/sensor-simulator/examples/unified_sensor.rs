use sensor_simulator::prelude::*;

fn main() {
    println!("=== Simulated Sensor Example ===\n");

    println!("1. Sine wave sensor:");
    let sine_config = SimulatorConfig {
        amplitude: 10.0,
        frequency: 0.2,
        offset: 50.0,
        noise_level: 0.5,
    };
    let sine_sensor = SimulatedSensor::new(SimulatorType::Sine(sine_config));
    demo_sensor(&sine_sensor);

    println!("\n2. Random sensor:");
    let random_sensor = SimulatedSensor::new(SimulatorType::Random {
        min: 0.0,
        max: 100.0,
    });
    demo_sensor(&random_sensor);

    println!("\n3. Constant sensor:");
    let constant_sensor = SimulatedSensor::new(SimulatorType::Constant(42.0));
    demo_sensor(&constant_sensor);
}

fn demo_sensor<S: SensorProvider>(sensor: &S) {
    println!("Sensor name: {}", sensor.name());
    println!("Connected: {}", sensor.is_connected());

    for i in 0..3 {
        match sensor.read() {
            Ok(value) => println!("  Read {}: {:.2}", i + 1, value),
            Err(e) => println!("  Read {} failed: {}", i + 1, e),
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

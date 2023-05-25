use std::time::Duration;

use metrics::{describe_counter, register_counter, Unit};
use metrics_exporter_cli::CliRegister;

fn main() {
    let mut register = CliRegister::install().expect("Error installing register");
    std::thread::spawn(move || {
        register.print_loop();
    });

    let absolute = register_counter!("absolute");
    let difference = register_counter!("difference");
    describe_counter!("difference", Unit::CountPerSecond, "");
    let histogram = register_counter!("histogram", "view" => "histogram");

    loop {
        absolute.increment(1);
        difference.increment(1);
        histogram.increment(1);
        std::thread::sleep(Duration::from_secs(1));
    }
}

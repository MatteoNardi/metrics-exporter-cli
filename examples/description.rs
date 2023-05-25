use std::time::Duration;

use metrics::{describe_counter, register_counter, Unit};
use metrics_exporter_cli::CliRegister;

use rand::prelude::*;

fn main() {
    let mut register = CliRegister::install().expect("Error installing register");
    std::thread::spawn(move || {
        register.print_loop();
    });

    let iterations = register_counter!(".iterations");
    let val_a = register_counter!("group1.val_a");
    let val_b = register_counter!("group1.val_b");
    describe_counter!("group1.val_b", Unit::CountPerSecond, "Value B of group 1");
    let processed = register_counter!("histogram.processed", "view" => "histogram");
    describe_counter!("histogram.processed", Unit::CountPerSecond, "");

    let mut rng = thread_rng();
    loop {
        iterations.increment(1);
        val_a.increment(rng.gen_range(0..5));
        val_b.increment(rng.gen_range(0..10));
        processed.increment(rng.gen_range(0..20));
        std::thread::sleep(Duration::from_secs(1));
    }
}

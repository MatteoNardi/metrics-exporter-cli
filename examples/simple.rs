use std::time::Duration;

use metrics::{
    counter, describe_counter, describe_gauge, gauge, increment_counter, register_counter,
    register_gauge, Unit,
};
use metrics_exporter_cli::CliRegister;

use rand::prelude::*;

fn main() {
    let mut register = CliRegister::install().expect("Error installing register");
    std::thread::spawn(move || {
        register.print_loop();
    });

    // TODO: split descrition to different example
    register_counter!("group1.B");
    //describe_counter!("group1.B", Unit::CountPerSecond, "Value B of group 1");
    //register_gauge!("histogram.processed", "view" => "histogram");
    //describe_gauge!("histogram.processed", Unit::CountPerSecond, "");

    let mut rng = thread_rng();
    loop {
        increment_counter!("iterations");
        //counter!("group1.A", rng.gen_range(0..5));
        counter!("group1.B", rng.gen_range(0..10));
        //gauge!("histogram.processed",  rng.gen_range(0.0..20.0), "view" => "histogram");
        std::thread::sleep(Duration::from_secs(1));
    }
}

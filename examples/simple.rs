use std::time::Duration;

use metrics::{counter, increment_counter};
use metrics_exporter_cli::CliRegister;

use rand::prelude::*;

fn main() {
    let mut register = CliRegister::install().expect("Error installing register");
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        // print_loop() is the same as:
        // println!("{}", register.header());
        // loop {
        //     println!("{}", register.status());
        //     std::thread::sleep(Duration::from_secs(1));
        // }
        register.print_loop();
    });

    let mut rng = thread_rng();
    loop {
        increment_counter!("iterations");
        counter!("group1.val_a", rng.gen_range(0..5));
        counter!("group1.val_b", rng.gen_range(0..10));
        std::thread::sleep(Duration::from_secs(1));
    }
}

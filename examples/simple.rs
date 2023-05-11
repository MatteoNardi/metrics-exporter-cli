use std::time::Duration;

use metrics::{counter, describe_counter, register_counter, Unit};
use metrics_exporter_cli::CliRegister;

fn main() {
    let mut register = CliRegister::install().expect("Error installing register");
    // TODO: split descrition to different example
    register_counter!("group1.val_b");
    describe_counter!("group1.val_b", Unit::CountPerSecond, "Value B of group 1");
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        register.print_loop();
    });

    let mut iterations = 0;
    counter!("group2", 42);
    loop {
        counter!("group1.val_a", iterations * 10);
        counter!("group1.val_b", iterations * 7);
        iterations += 1;
        std::thread::sleep(Duration::from_secs(1));
    }
}

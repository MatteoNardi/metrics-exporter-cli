use std::time::Duration;

use metrics::counter;
use metrics_exporter_cli::CliRegister;

fn main() {
    let mut register = CliRegister::install().expect("Error installing register");
    std::thread::spawn(|| {
        let mut iterations = 0;
        counter!("group2", 42);
        loop {
            counter!("group1.val_a", iterations * 10);
            counter!("group1.val_b", iterations * 7);
            iterations += 1;
            std::thread::sleep(Duration::from_secs(1));
        }
    });

    std::thread::sleep(Duration::from_secs(1));
    println!("{}", register.header());
    loop {
        println!("{}", register.status());
        std::thread::sleep(Duration::from_secs(1));
    }
}

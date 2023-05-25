# metrics-exporter-cli

A work-in-progress exporter for the [metrics crate](https://github.com/metrics-rs/metrics)
which display a table on CLI.

```
  group1    |
val_a val_b | iterations
    4     4            1
    5     6            2
    8     9            3
    9    15            4
```

This is a simple exporter build on top of
[`metrics_util::debugging::DebuggingRecorder`](https://docs.rs/metrics-util/latest/metrics_util/debugging/struct.DebuggingRecorder.html).
It should be used for debugging purposes and as a simpler (and much more limited)
solution compared to a full blown metrics setup using prometheus.

WARNING: this is a work in progress prototype. Expect bugs, missing features, crappy API and breaking changes.
You're more than welcome to open issues with feedback and feature requests.

## Simple automatic usage

The table above can be created with [this](examples/simple.rs) code:

```rust
use std::time::Duration;

use metrics::{counter, increment_counter};
use metrics_exporter_cli::CliRegister;

use rand::prelude::*;

fn main() {
    let mut register = CliRegister::install().expect("Error installing register");
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
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
```

After installing the CliRegister we spawn the table printing loop, which just
prints to stdout the header and a new line every 1s. We could also do this manually
by printing the output of `register.header()` and `register.status()`.

In this example there is no table configuration. The columns will be grouped by splitting dots in key names.
Every column will be aligned and keep as little space as possible to include the value and header field.

When a value is too big for its cell, its column will be enlarged from that moment on. This will cause an
unalignment with the lines before, but should be readable and quite minimal.

## Descriptive usage

The [`description` exapmle](examples/description.rs) uses metrics labels and units of measure
to configure the table. In particular, all `*PerSecond` metrics will cause the cells to display
a difference with the previous value (this assumes 1 display per second, or the results will be wrong).

```rust
let absolute = register_counter!("absolute");                         
let difference = register_counter!("difference");                     
describe_counter!("difference", Unit::CountPerSecond, "");            
let histogram = register_counter!("histogram", "view" => "histogram");
```

The same monothonically increasing counter will result in this table:

```
absolute difference histogram
       1          1 #        
       2          1 ##       
       3          1 ###      
       4          1 ####     
       5          1 #####    
```

## TODO

- Ideally, I'd like to add a builder API to configure the table as an alternative to the "descriptive usage".
- The histogram feature needs to be improved
- API needs to be improved


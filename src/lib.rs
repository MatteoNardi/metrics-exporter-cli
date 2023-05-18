mod table;

use std::{collections::HashMap, time::Duration};

use metrics::{SetRecorderError, Unit};
use metrics_util::debugging::{DebugValue, DebuggingRecorder, Snapshot, Snapshotter};
use table::{DisplayKind, Table, TableBuilder, Value};

pub struct CliRegister {
    snapshotter: SnapshotterKind,
    table: Table,
}

enum SnapshotterKind {
    Snapshotter(Snapshotter),
    PerThread,
}

impl CliRegister {
    pub fn install() -> Result<Self, SetRecorderError> {
        let recorder = DebuggingRecorder::new();
        let snapshotter = recorder.snapshotter();
        recorder.install()?;
        Ok(Self {
            snapshotter: SnapshotterKind::Snapshotter(snapshotter),
            table: TableBuilder::new().build(),
        })
    }

    pub fn install_on_thread() -> Self {
        let recorder = DebuggingRecorder::per_thread();
        _ = recorder.install();
        Self {
            snapshotter: SnapshotterKind::PerThread,
            table: TableBuilder::new().build(),
        }
    }

    fn snapshot(&self) -> Snapshot {
        match &self.snapshotter {
            SnapshotterKind::Snapshotter(snapshotter) => snapshotter.snapshot(),
            SnapshotterKind::PerThread => {
                Snapshotter::current_thread_snapshot().expect("No current thread snapshot")
            }
        }
    }

    pub fn header(&mut self) -> String {
        // Recompute table header
        self.table = table_from_snapshot(self.snapshot());
        self.table.header()
    }

    pub fn status(&mut self) -> String {
        let snapshot = self.snapshot();
        let mut items: Vec<(Option<usize>, DebugValue)> = snapshot
            .into_vec()
            .into_iter()
            .map(|item| {
                let path = item
                    .0
                    .key()
                    .name()
                    .to_string()
                    .split('.')
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                let i = self.table.position_of(path.clone());
                (i, item.3)
            })
            .collect();
        items.sort_by_key(|x| x.0);
        let values = items
            .into_iter()
            .map(|(_, value)| match value {
                DebugValue::Counter(x) => Value::Int(x as i64),
                DebugValue::Gauge(x) => Value::F64(*x),
                DebugValue::Histogram(_) => todo!(),
            })
            .collect();
        self.table.display_row(values)
    }

    /// Start an infinite loop which prints a table line every second.
    ///
    /// Note: you could write your own loop with a different interval, but be
    /// aware that values with a unit type *PerSecond will just print the difference
    /// from the last print invocation, indipendently of how much time has actually
    /// passed.
    pub fn print_loop(&mut self) -> ! {
        println!("{}", self.header());
        loop {
            println!("{}", self.status());
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}

fn table_from_snapshot(snapshot: Snapshot) -> Table {
    let mut components: Vec<Component> = snapshot
        .into_vec()
        .into_iter()
        .map(|x| Component {
            path: x
                .0
                .key()
                .name()
                .to_string()
                .split(".")
                .map(|x| x.to_string())
                .collect(),
            unit: x.1.unwrap_or(Unit::Count),
            labels: x
                .0
                .key()
                .labels()
                .map(|label| (label.key().to_string(), label.value().to_string()))
                .collect(),
        })
        .collect();
    // TODO: remove clone
    components.sort_by_key(|x| x.path.clone());
    build(TableBuilder::new(), &mut components[..], 0).build()
}

struct Component {
    path: Vec<String>,
    unit: Unit,
    labels: HashMap<String, String>,
}

fn build(mut builder: TableBuilder, components: &mut [Component], depth: usize) -> TableBuilder {
    let mut i = 0;
    while i < components.len() {
        let name = components[i].path[depth].clone();
        if components[i].path.len() == depth + 1 {
            let mut display_kind = match components[i].unit {
                Unit::TerabitsPerSecond
                | Unit::GigabitsPerSecond
                | Unit::MegabitsPerSecond
                | Unit::KilobitsPerSecond
                | Unit::BitsPerSecond
                | Unit::CountPerSecond => DisplayKind::Difference,
                _ => DisplayKind::Number,
            };
            if components[i].labels.get("view") == Some(&"histogram".to_string()) {
                display_kind = DisplayKind::Histogram;
            }
            builder = builder.field(&name, display_kind);
            i = i + 1;
        } else {
            // make group, take out all items which share prefix
            let group_size = components.iter().filter(|c| c.path[depth] == name).count();
            builder = builder.group(&name, |group_builder| {
                build(group_builder, &mut components[i..i + group_size], depth + 1)
            });
            i = i + group_size;
        }
    }
    builder
}

#[cfg(test)]
mod tests {
    use metrics::{counter, describe_counter, register_counter};

    use super::*;

    #[test]
    fn simple_header() {
        unsafe {
            metrics::clear_recorder();
        }
        // TODO: do we want internal mutability?
        let mut register = CliRegister::install_on_thread();
        counter!("val_a", 10);
        counter!("val_b", 20);
        assert_eq!(register.header(), ["val_a val_b"].join("\n"));
    }

    #[test]
    fn composite_header() {
        unsafe {
            metrics::clear_recorder();
        }
        let mut register = CliRegister::install_on_thread();
        counter!("g1.val_a", 10);
        counter!("g1.val_b", 20);
        assert_eq!(register.header(), ["    g1", "val_a val_b"].join("\n"));
    }

    #[test]
    fn simple_status() {
        unsafe {
            metrics::clear_recorder();
        }
        let mut register = CliRegister::install_on_thread();
        counter!("val_a", 10);
        counter!("val_b", 20);
        _ = register.header(); // TODO: this easy to misuse
        assert_eq!(register.status(), ["   10    20"].join("\n"));
    }

    #[test]
    fn simple_difference() {
        unsafe {
            metrics::clear_recorder();
        }
        let mut register = CliRegister::install_on_thread();
        register_counter!("val_a");
        describe_counter!("val_a", Unit::CountPerSecond, "Val A");
        counter!("val_a", 10);
        _ = register.header(); // TODO: this easy to misuse
        assert_eq!(register.status(), ["   10"].join("\n"));
        counter!("val_a", 22);
        assert_eq!(register.status(), ["   22"].join("\n"));
    }

    #[test]
    fn status_preserves_order() {
        // fields should be displayed in the header order
        unsafe {
            metrics::clear_recorder();
        }
        let mut register = CliRegister::install_on_thread();
        register_counter!("a.val_a");
        counter!("val_b", 20);
        counter!("a.val_a", 10);
        _ = register.header(); // TODO: this easy to misuse
        assert_eq!(register.status(), ["   10      20"].join("\n"));
    }
}

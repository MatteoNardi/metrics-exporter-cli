mod table;

use metrics::SetRecorderError;
use metrics_util::debugging::{DebugValue, DebuggingRecorder, Snapshot, Snapshotter};
use table::{Table, TableBuilder};

pub struct CliRegister {
    snapshotter: SnapshotterKind,
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
        })
    }

    pub fn install_on_thread() -> Self {
        let recorder = DebuggingRecorder::per_thread();
        _ = recorder.install();
        Self {
            snapshotter: SnapshotterKind::PerThread,
        }
    }

    fn snapshot(&self) -> Snapshot {
        match &self.snapshotter {
            SnapshotterKind::Snapshotter(snapshotter) => snapshotter.snapshot(),
            SnapshotterKind::PerThread => Snapshotter::current_thread_snapshot().unwrap(),
        }
    }

    pub fn header(&self) -> String {
        table_from_snapshot(self.snapshot()).header()
    }

    pub fn status(&self) -> String {
        let snapshot = self.snapshot();
        let table = table_from_snapshot(self.snapshot());
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
                let i = table.position_of(path);
                (i, item.3)
            })
            .collect();
        items.sort_by_key(|x| x.0);
        let values = items
            .into_iter()
            .map(|(_, value)| match value {
                DebugValue::Counter(x) => x as i64,
                DebugValue::Gauge(_) => todo!(),
                DebugValue::Histogram(_) => todo!(),
            })
            .collect();
        table.display_row(values)
    }
}

fn table_from_snapshot(snapshot: Snapshot) -> Table {
    let mut keys: Vec<String> = snapshot
        .into_vec()
        .into_iter()
        .map(|x| x.0.key().name().to_string())
        .collect();
    keys.sort();
    let components: Vec<Vec<&str>> = keys.iter().map(|key| key.split('.').collect()).collect();
    build(TableBuilder::new(), &components).build()
}

fn build(mut builder: TableBuilder, components: &Vec<Vec<&str>>) -> TableBuilder {
    let mut i = 0;
    while i < components.len() {
        let component = &components[i];
        let name = component[0];
        if component.len() == 1 {
            builder = builder.field(name);
        } else {
            // make group, take out all items which share prefix
            let subset = components
                .iter()
                .filter_map(|c| c.split_first())
                .filter_map(|(first, rest)| (first == &name).then(|| rest.to_vec()))
                .collect();
            builder = builder.group(name, |group_builder| build(group_builder, &subset));
        }
        i = i + 1;
    }
    builder
}

#[cfg(test)]
mod tests {
    use metrics::counter;

    use super::*;

    #[test]
    fn simple_header() {
        unsafe {
            metrics::clear_recorder();
        }
        let register = CliRegister::install_on_thread();
        counter!("val_a", 10);
        counter!("val_b", 20);
        assert_eq!(register.header(), ["val_a val_b"].join("\n"));
    }

    #[test]
    fn simple_status() {
        unsafe {
            metrics::clear_recorder();
        }
        let register = CliRegister::install_on_thread();
        counter!("val_a", 10);
        counter!("val_b", 20);
        assert_eq!(register.status(), ["   10    20"].join("\n"));
    }
}

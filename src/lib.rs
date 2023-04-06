#![allow(dead_code, unused)]
mod table;

use metrics::SetRecorderError;
use metrics_util::debugging::{DebuggingRecorder, Snapshot, Snapshotter};
use table::TableBuilder;

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
        let mut keys: Vec<String> = self
            .snapshot()
            .into_vec()
            .into_iter()
            .map(|x| x.0.key().name().to_string())
            .collect();
        keys.sort();
        let components: Vec<Vec<&str>> = keys.iter().map(|key| key.split('.').collect()).collect();

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
        build(TableBuilder::new(), &components).build().header()
    }

    pub fn status(&self) -> String {
        // TODO: make table stable by rebuilding on header change
        // TODO: transform snapshot to ordered list matching table
        todo!();
    }
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
        counter!("input_counter", 10);
        assert_eq!(register.header(), ["input_counter"].join("\n"));
    }

    #[test]
    fn simple_status() {
        unsafe {
            metrics::clear_recorder();
        }
        let register = CliRegister::install_on_thread();
        counter!("input_counter", 10);
        assert_eq!(register.status(), ["10"].join("\n"));
    }
}

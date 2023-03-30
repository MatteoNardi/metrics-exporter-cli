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
        self.snapshot()
            .into_vec()
            .into_iter()
            .map(|(key, _unit, _, _value)| key.key().name().to_string())
            .collect::<Vec<_>>()
            .join("\t");

        todo!();
        //TableBuilder::new()
        //    .group("input", |input| input.column("counter").column("counter2"))
        //    .build()
        //    .header()
    }

    pub fn status(&self) -> String {
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
}

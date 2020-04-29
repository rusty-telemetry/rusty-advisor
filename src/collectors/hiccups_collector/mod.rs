use std::sync::mpsc::SyncSender;

// pub mod hiccup_monitor;

pub struct HiccupsCollector {
    pub tx: SyncSender<u64>,
}

impl HiccupsCollector {
}


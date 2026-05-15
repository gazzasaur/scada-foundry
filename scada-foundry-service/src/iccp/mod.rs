use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, time::Duration};

use tokio::sync::{RwLock, mpsc};

use crate::config::iccp::IccpConfiguration;

pub struct IccpManager {
    running: Arc<AtomicBool>,
    config: Arc<RwLock<Option<IccpConfiguration>>>,
}

impl IccpManager {
    pub fn new() -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let worker_running = running.clone();
        let mut worker = IccpManagerWorker { config: Arc::new(RwLock::new(None)) };
        tokio::task::spawn(async move {
            while worker_running.load(Ordering::Acquire) {
                if !worker.process() {
                    // Prevent the thread from going to 100% CPU when there is no work to be done.
                    tokio::time::sleep(Duration::from_millis(1));
                }
            }
        });

        IccpManager { running: Arc::new(AtomicBool::new(true)), config: Arc::new(RwLock::new(None)) }
    }

    pub async fn apply_config(config: IccpConfiguration) {
        // config.
    }
}

impl Drop for IccpManager {
    fn drop(&mut self) {
        self.running.store(false, Ordering::AcqRel);
    }
}

enum IccpManagerWorkerTask {

}

struct IccpManagerWorker {
    config: Arc<RwLock<Option<IccpConfiguration>>>,
    task_queue: mpsc::Receiver<IccpManagerWorkerTask>
    task_queuer: mpsc::Sender<IccpManagerWorkerTask>
}

impl IccpManagerWorker {
    fn process(&mut self) -> bool {
        return false;
    }
}

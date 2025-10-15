use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Handle for a running extraction job
pub struct JobHandle {
    /// Flag to signal cancellation
    pub cancel_flag: Arc<AtomicBool>,
    /// The async task handle
    pub task: JoinHandle<Result<extractor::ExtractStats, extractor::ExtractError>>,
    /// Optional sender for password retry
    pub password_sender: Option<mpsc::Sender<String>>,
}

/// Application state managing all active extraction jobs
#[derive(Default)]
pub struct AppState {
    /// Map of job_id to JobHandle
    pub jobs: Arc<Mutex<HashMap<String, JobHandle>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

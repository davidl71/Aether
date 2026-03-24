use std::sync::Arc;

use api::SystemSnapshot;
use tokio::sync::RwLock;

pub type SharedSnapshot = Arc<RwLock<SystemSnapshot>>;

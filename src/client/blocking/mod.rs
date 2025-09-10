mod chat;
mod completion;

use crate::client::core::{OrpheusCore, Sync};

/// Alias for the OrpheusCore client in `Blocking` mode.
pub type Orpheus = OrpheusCore<Sync>;

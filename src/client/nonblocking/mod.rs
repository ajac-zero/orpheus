mod chat;
mod completion;

use crate::client::core::{Async, OrpheusCore};

/// Alias for the OrpheusCore client in `Async` mode.
pub type AsyncOrpheus = OrpheusCore<Async>;

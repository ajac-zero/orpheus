pub(crate) mod core;
mod methods;

pub use core::OrpheusCore;
use core::{Async, Sync};

/// Alias for the OrpheusCore client in `Blocking` mode.
pub type Orpheus = OrpheusCore<Sync>;

/// Alias for the OrpheusCore client in `Async` mode.
pub type AsyncOrpheus = OrpheusCore<Async>;

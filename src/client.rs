pub(crate) mod core;
pub(crate) mod handler;
pub(crate) mod mode;
mod methods {
    mod chat;
    mod completion;
    mod keys;
}

pub use core::OrpheusCore;

/// Alias for the OrpheusCore client in `Blocking` mode.
pub type Orpheus = OrpheusCore<mode::Sync>;

/// Alias for the OrpheusCore client in `Async` mode.
pub type AsyncOrpheus = OrpheusCore<mode::Async>;

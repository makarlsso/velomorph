//! Core runtime library for Velomorph.
//!
//! This crate provides:
//! - `Morph`, a derive macro re-export for generating transformations.
//! - `TryMorph`, the trait implemented by generated code.
//! - `Janitor`, a background deallocation helper to keep hot paths responsive.
//!
//! # Quick Example
//! ```ignore
//! use std::borrow::Cow;
//! use uuid::Uuid;
//! use velomorph::{Janitor, Morph, TryMorph};
//!
//! pub struct RawInput<'a> {
//!     // Legacy/external names from an upstream system:
//!     pub uuid_v4: Option<Uuid>,
//!     pub user_str: &'a str,
//!     pub payload: Option<Vec<u8>>,
//! }
//!
//! #[derive(Morph, Debug)]
//! pub struct Event<'a> {
//!     // Rename `uuid_v4` -> `id` while enforcing presence:
//!     #[morph(from = "uuid_v4")]
//!     pub id: Uuid,
//!
//!     // Rename `user_str` -> `username` while borrowing the string:
//!     #[morph(from = "user_str")]
//!     pub username: Cow<'a, str>,
//! }
//!
//! let janitor = Janitor::new();
//! let raw = RawInput {
//!     uuid_v4: Some(Uuid::new_v4()),
//!     user_str: "edge-a",
//!     payload: Some(vec![1, 2, 3]),
//! };
//! let event: Event = raw.try_morph(&janitor)?;
//! # Ok::<(), velomorph::MorphError>(())
//! ```

use std::thread;
use tokio::sync::mpsc;

/// Derive macro used to generate `TryMorph` implementations.
pub use velomorph_derive::Morph;

/// Errors that can occur while transforming an input type into a target type.
#[derive(thiserror::Error, Debug)]
pub enum MorphError {
    /// A required field was missing in the source value.
    #[error("Required field is missing: {0}")]
    MissingField(String),
}

/// Trait for fallible transformations from a source type to `Target`.
///
/// Implementations are typically generated via `#[derive(Morph)]`.
pub trait TryMorph<Target> {
    /// Attempts to transform `self` into `Target`.
    ///
    /// The `janitor` may be used by implementations to offload expensive
    /// deallocations from the current execution path.
    fn try_morph(self, janitor: &Janitor) -> Result<Target, MorphError>;
}

/// Background deallocation helper that minimizes latency spikes.
///
/// `Janitor` owns a channel to a dedicated thread. Large objects can be sent to
/// that thread for dropping, allowing the main path to continue with minimal
/// interruption.
#[derive(Clone)]
pub struct Janitor {
    sender: mpsc::UnboundedSender<Box<dyn std::any::Any + Send>>,
}

impl Janitor {
    /// Creates a new janitor with a background worker thread.
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        thread::spawn(move || {
            while let Some(item) = rx.blocking_recv() {
                // Drop happens outside the latency-sensitive path.
                std::mem::drop(item);
            }
        });

        Self { sender: tx }
    }

    /// Sends an object to the background thread for deallocation.
    ///
    /// This is useful when immediate drop cost would otherwise add jitter to
    /// a critical execution path.
    #[inline(always)]
    pub fn offload<T: Send + 'static>(&self, item: T) {
        let _ = self.sender.send(Box::new(item));
    }
}

/// Creates a new `Janitor` using `Janitor::new()`.
impl Default for Janitor {
    fn default() -> Self {
        Self::new()
    }
}

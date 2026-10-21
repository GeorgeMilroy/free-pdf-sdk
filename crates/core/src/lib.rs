//! Free PDF SDK core: everything the viewer does that needs neither egui nor a browser, so it
//! is tested natively: the RPC protocol and its validation, `disabledElements`, reading XFDF,
//! and the document operations behind the API.

pub mod doc;
pub mod elements;
pub mod error;
pub mod features;
pub mod profile;
pub mod protocol;
pub mod xfdf;

pub use error::{ErrorCode, RpcError};

/// The SDK version (semver), reported by `version()`.
pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The pdfcraft commit the SDK is built from (from Cargo.lock, see build.rs).
pub const PDFCRAFT_REV: &str = env!("FPS_PDFCRAFT_REV");

pub mod detection;
pub mod ripping;
pub mod scanning;
pub mod types;

// Re-exported types are used in other modules via dvd::* syntax
// Type exports are tagged with #[allow(unused_imports)] to prevent warnings
#[allow(unused_imports, reason = "Public API re-exports used by other modules")]
pub use types::{Dvd, RipTask, Title};

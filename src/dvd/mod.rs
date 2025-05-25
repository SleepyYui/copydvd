pub mod detection;
pub mod ripping;
pub mod scanning;
pub mod types;

// Re-exported types are used in other modules via dvd::* syntax
// Type exports are tagged with #[allow(unused_imports)] to prevent warnings
#[allow(unused_imports)]
pub use types::Dvd;
#[allow(unused_imports)]
pub use types::RipTask;
#[allow(unused_imports)]
pub use types::Title;

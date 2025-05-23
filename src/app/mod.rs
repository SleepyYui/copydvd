mod startup;
pub mod runner;
mod state;

pub use state::{AppState, AppStatus};
pub use startup::auto_detect_dvd;
pub use runner::run;

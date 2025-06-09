pub mod about;
pub mod config;
pub mod handbrake;
pub mod main;
pub mod server;
pub mod updates;

pub use about::render_about_tab;
pub use config::render_config_tab;
pub use handbrake::render_handbrake_tab;
pub use main::render_main_tab;
pub use server::render_server_tab;
pub use updates::UpdatesTab;

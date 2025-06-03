use crate::config::{Config, HandBrakeManagementConfig, ServerConfig};
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_save_and_load() {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_path = temp_dir.path().join("test_config.json");

        // Create a test config
        let original_config = Config {
            handbrake_path: Some(PathBuf::from("/usr/bin/HandBrakeCLI")),
            output_dir: PathBuf::from("/home/user/Videos"),
            encode_algo: "H.264".to_string(),
            video_codec: "x264".to_string(),
            chapter_split: true,
            server: Some(ServerConfig {
                host: "example.com".to_string(),
                username: "testuser".to_string(),
                password: Some("testpass".to_string()),
                path: "/remote/path".to_string(),
            }),
            eject_after_rip: false,
            thread_count: 8,
            handbrake_management: HandBrakeManagementConfig {
                auto_download: false,
                prefer_system: true,
                max_cache_size_mb: 200,
                verify_on_startup: false,
            },
        };

        // Save config
        let config_json =
            serde_json::to_string_pretty(&original_config).expect("Failed to serialize config");
        std::fs::write(&config_path, config_json).expect("Failed to write config");

        // Load config
        let loaded_json = std::fs::read_to_string(&config_path).expect("Failed to read config");
        let loaded_config: Config =
            serde_json::from_str(&loaded_json).expect("Failed to deserialize config");

        // Verify all fields match
        assert_eq!(original_config.handbrake_path, loaded_config.handbrake_path);
        assert_eq!(original_config.output_dir, loaded_config.output_dir);
        assert_eq!(original_config.encode_algo, loaded_config.encode_algo);
        assert_eq!(original_config.video_codec, loaded_config.video_codec);
        assert_eq!(original_config.chapter_split, loaded_config.chapter_split);
        assert_eq!(
            original_config.eject_after_rip,
            loaded_config.eject_after_rip
        );
        assert_eq!(original_config.thread_count, loaded_config.thread_count);

        // Verify server config
        assert!(loaded_config.server.is_some());
        let loaded_server = loaded_config.server.unwrap();
        let original_server = original_config.server.unwrap();
        assert_eq!(original_server.host, loaded_server.host);
        assert_eq!(original_server.username, loaded_server.username);
        assert_eq!(original_server.password, loaded_server.password);
        assert_eq!(original_server.path, loaded_server.path);

        // Verify HandBrake management config
        assert_eq!(
            original_config.handbrake_management.auto_download,
            loaded_config.handbrake_management.auto_download
        );
        assert_eq!(
            original_config.handbrake_management.prefer_system,
            loaded_config.handbrake_management.prefer_system
        );
        assert_eq!(
            original_config.handbrake_management.max_cache_size_mb,
            loaded_config.handbrake_management.max_cache_size_mb
        );
        assert_eq!(
            original_config.handbrake_management.verify_on_startup,
            loaded_config.handbrake_management.verify_on_startup
        );
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();

        assert!(config.handbrake_path.is_none());
        assert!(config.output_dir.ends_with("outs"));
        assert_eq!(config.encode_algo, "MP4");
        assert_eq!(config.video_codec, "x264");
        assert!(!config.chapter_split);
        assert!(config.server.is_none());
        assert!(config.eject_after_rip);
        assert!(config.thread_count >= 1);
        assert!(config.handbrake_management.auto_download);
        assert!(config.handbrake_management.prefer_system);
        assert_eq!(config.handbrake_management.max_cache_size_mb, 100);
        assert!(config.handbrake_management.verify_on_startup);
    }

    #[test]
    fn test_server_config_serialization() {
        let server_config = ServerConfig {
            host: "test.example.com".to_string(),
            username: "user123".to_string(),
            password: None,
            path: "/uploads".to_string(),
        };

        let json =
            serde_json::to_string(&server_config).expect("Failed to serialize server config");
        let loaded: ServerConfig =
            serde_json::from_str(&json).expect("Failed to deserialize server config");

        assert_eq!(server_config.host, loaded.host);
        assert_eq!(server_config.username, loaded.username);
        assert_eq!(server_config.password, loaded.password);
        assert_eq!(server_config.path, loaded.path);
    }
}

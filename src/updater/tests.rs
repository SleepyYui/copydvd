use super::*;

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_filter_detection() {
        let updater = Updater::new();
        let filter = updater.platform_filter;

        // Verify platform filter is appropriate for current platform
        #[cfg(target_os = "windows")]
        assert!(filter.contains("windows"));

        #[cfg(target_os = "macos")]
        assert!(filter.contains("apple-darwin"));

        #[cfg(target_os = "linux")]
        assert!(filter.contains("linux-gnu"));

        assert!(!filter.is_empty());
    }

    #[test]
    fn test_version_comparison() {
        let updater = Updater::new();

        // Test with mock releases
        let releases = vec![
            Release {
                tag_name: "v1.0.0".to_string(),
                name: "Version 1.0.0".to_string(),
                body: "Initial release".to_string(),
                published_at: "2024-01-01T00:00:00Z".to_string(),
                prerelease: false,
                draft: false,
                assets: vec![],
            },
            Release {
                tag_name: "v1.1.0".to_string(),
                name: "Version 1.1.0".to_string(),
                body: "Bug fixes".to_string(),
                published_at: "2024-02-01T00:00:00Z".to_string(),
                prerelease: false,
                draft: false,
                assets: vec![],
            },
        ];

        let status = updater.compare_versions(releases);

        // Should detect newer versions based on current version
        match status {
            UpdateStatus::UpdateAvailable { latest, .. } => {
                assert_eq!(latest.tag_name, "v1.1.0");
            }
            UpdateStatus::UpToDate => {
                // This is expected if running version is already newer
            }
            UpdateStatus::Error(_) => {
                panic!("Should not have error in version comparison");
            }
        }
    }

    #[test]
    fn test_installer_patterns() {
        let updater = Updater::new();
        let installer_patterns = updater.get_installer_patterns();
        let binary_patterns = updater.get_binary_patterns();

        // Should have appropriate patterns for current platform
        #[cfg(target_os = "windows")]
        {
            assert!(installer_patterns.contains(&".msi"));
            assert!(binary_patterns
                .iter()
                .any(|p| p.contains("pc-windows-msvc")));
        }

        #[cfg(target_os = "macos")]
        {
            assert!(installer_patterns.contains(&".app.zip"));
            assert!(binary_patterns.iter().any(|p| p.contains("apple-darwin")));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(!installer_patterns.is_empty()); // Should have various Linux package formats
            assert!(binary_patterns
                .iter()
                .any(|p| p.contains("unknown-linux-gnu")));
        }
    }

    #[test]
    fn test_download_path_generation() {
        let updater = Updater::new();
        let filename = "test-update.exe";

        let path = updater.get_download_path(filename);
        assert!(path.is_ok());

        let path = path.unwrap();
        assert!(path.ends_with("copydvd_update_test-update.exe"));
        assert!(path.is_absolute());
    }

    #[tokio::test]
    async fn test_asset_filtering() {
        let updater = Updater::new();

        let release = Release {
            tag_name: "v1.0.0".to_string(),
            name: "Test Release".to_string(),
            body: "Test release".to_string(),
            published_at: "2024-01-01T00:00:00Z".to_string(),
            prerelease: false,
            draft: false,
            assets: vec![
                Asset {
                    name: "copydvd-x86_64-pc-windows-msvc.exe".to_string(),
                    download_url: "https://example.com/windows.exe".to_string(),
                    size: 1024,
                    content_type: "application/octet-stream".to_string(),
                },
                Asset {
                    name: "copydvd-x86_64-apple-darwin".to_string(),
                    download_url: "https://example.com/macos-x86".to_string(),
                    size: 1024,
                    content_type: "application/octet-stream".to_string(),
                },
                Asset {
                    name: "copydvd-aarch64-apple-darwin".to_string(),
                    download_url: "https://example.com/macos-arm".to_string(),
                    size: 1024,
                    content_type: "application/octet-stream".to_string(),
                },
                Asset {
                    name: "copydvd-x86_64-unknown-linux-gnu".to_string(),
                    download_url: "https://example.com/linux-x86".to_string(),
                    size: 1024,
                    content_type: "application/octet-stream".to_string(),
                },
                Asset {
                    name: "CopyDVD-x86_64-pc-windows-msvc.msi".to_string(),
                    download_url: "https://example.com/installer.msi".to_string(),
                    size: 2048,
                    content_type: "application/octet-stream".to_string(),
                },
                Asset {
                    name: "CopyDVD-x86_64-apple-darwin.app.zip".to_string(),
                    download_url: "https://example.com/macos-app.zip".to_string(),
                    size: 3072,
                    content_type: "application/zip".to_string(),
                },
                Asset {
                    name: "CopyDVD-aarch64-apple-darwin.app.zip".to_string(),
                    download_url: "https://example.com/macos-arm-app.zip".to_string(),
                    size: 3072,
                    content_type: "application/zip".to_string(),
                },
                Asset {
                    name: "copydvd-x86_64-unknown-linux-gnu.deb".to_string(),
                    download_url: "https://example.com/linux.deb".to_string(),
                    size: 2048,
                    content_type: "application/octet-stream".to_string(),
                },
            ],
        };

        let result = updater.find_compatible_asset(&release);

        // Should find a compatible asset
        assert!(result.is_ok());
        #[allow(unused_variables)]
        let asset = result.unwrap();

        // Should prefer installer over binary
        #[cfg(target_os = "windows")]
        assert!(asset.name.ends_with(".msi") || asset.name.contains("pc-windows-msvc"));

        #[cfg(target_os = "macos")]
        assert!(asset.name.ends_with(".app.zip") || asset.name.contains("apple-darwin"));

        #[cfg(target_os = "linux")]
        assert!(asset.name.contains("unknown-linux-gnu"));
    }

    #[test]
    fn test_github_api_url_formation() {
        // Test that the GitHub API URL is correctly formed
        assert_eq!(
            GITHUB_API_RELEASES,
            "https://api.github.com/repos/sleepyyui/copydvd/releases"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_update_script_generation() {
        let updater = Updater::new();

        // Test that we can generate the update script path
        if let Ok(current_exe) = std::env::current_exe() {
            let script_path = current_exe.with_file_name("update_copydvd.bat");
            assert!(script_path.extension().unwrap() == "bat");
        }
    }

    #[tokio::test]
    async fn test_update_check_timeout() {
        let _updater = Updater::new();

        // This test verifies that update checks respect timeouts
        // We can't test with a real network call in unit tests,
        // but we can verify the timeout constant is reasonable
        assert!(UPDATE_CHECK_TIMEOUT.as_secs() >= 5);
        assert!(UPDATE_CHECK_TIMEOUT.as_secs() <= 30);
        assert!(DOWNLOAD_TIMEOUT.as_secs() >= 60);
    }

    #[test]
    fn test_current_version_parsing() {
        let updater = Updater::new();
        let version_str = updater.get_current_version();

        // Should be able to parse the current version
        assert!(!version_str.is_empty());

        // Should be in semver format or at least have version-like structure
        assert!(version_str.contains('.') || version_str.chars().any(|c| c.is_numeric()));
    }
}

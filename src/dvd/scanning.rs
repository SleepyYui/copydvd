use crate::dvd::types::{Dvd, Title, DvdSize, Chapter};
use crate::error::{Result, AppError};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};
use regex::Regex; // For parsing HandBrake output

impl Dvd {
    pub async fn scan_titles(&mut self) -> Result<()> {
        info!("Scanning titles for DVD at: {}", self.path.display());

        let handbrake_path = self.config.handbrake_path.as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "HandBrakeCLI".to_string());

        let mut cmd = Command::new(&handbrake_path);
        cmd.arg("-i")
            .arg(self.path.as_os_str())
            .arg("--scan")
            .arg("--json"); // Request JSON output for easier parsing

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await
            .map_err(|e| AppError::HandbrakeError(format!("Failed to execute HandBrakeCLI for scan: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("HandBrakeCLI scan error: {}", stderr);
            // Try to parse stderr for common errors like "no main feature found"
            if stderr.contains("No title found") || stderr.contains("no main feature found") {
                 self.titles = Vec::new(); // Ensure titles list is empty
                 return Ok(()); // Not a fatal error, just no titles
            }
            return Err(AppError::HandbrakeError(format!(
                "HandBrakeCLI scan failed with status {}: {}",
                output.status, stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // First, check if HandBrakeCLI outputted the JSON directly
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(title_list) = json_val.get("TitleList").and_then(|v| v.as_array()) {
                self.titles = parse_handbrake_json_titles(title_list)?;
                info!("Found {} titles via JSON output.", self.titles.len());
                return Ok(());
            }
        }

        // Fallback: try to find JSON block in text output if direct parsing failed
        // HandBrakeCLI sometimes prints text before the JSON block.
        let re_json = Regex::new(r"JSON Title Set:(?s)(.*)").unwrap();
        if let Some(caps) = re_json.captures(&stdout) {
            if let Some(json_match) = caps.get(1) {
                let json_str = json_match.as_str().trim();
                 if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(title_list) = json_val.get("TitleList").and_then(|v| v.as_array()) {
                        self.titles = parse_handbrake_json_titles(title_list)?;
                        info!("Found {} titles via extracted JSON block.", self.titles.len());
                        return Ok(());
                    }
                } else {
                    warn!("Failed to parse extracted JSON from HandBrake scan: {}", json_str);
                }
            }
        }
        
        warn!("Could not parse HandBrake scan output as JSON. Stdout: {}", stdout);
        Err(AppError::HandbrakeError("Failed to parse HandBrakeCLI scan JSON output".to_string()))
    }


}

fn parse_handbrake_json_titles(title_list: &Vec<serde_json::Value>) -> Result<Vec<Title>> {
    let mut titles = Vec::new();
    for (index, title_json) in title_list.iter().enumerate() {
        let duration_ms = title_json.get("Duration").and_then(|d| d.get("Milliseconds")).and_then(|ms| ms.as_u64()).unwrap_or(0);
        let width = title_json.get("Geometry").and_then(|g| g.get("Width")).and_then(|w| w.as_u64()).unwrap_or(0) as u32;
        let height = title_json.get("Geometry").and_then(|g| g.get("Height")).and_then(|h| h.as_u64()).unwrap_or(0) as u32;
        let description = title_json.get("Name").and_then(|n| n.as_str()).map(String::from);

        let empty_chapters = Vec::new(); // Create a longer-lived binding
        let chapter_list_json = title_json.get("ChapterList").and_then(|cl| cl.as_array()).unwrap_or(&empty_chapters);
        let mut chapters = Vec::new();
        for (chap_idx, chap_json) in chapter_list_json.iter().enumerate() {
             let chap_duration_ms = chap_json.get("Duration").and_then(|d| d.get("Milliseconds")).and_then(|ms| ms.as_u64()).unwrap_or(0);
             chapters.push(Chapter {
                 number: chap_idx + 1,
                 duration: std::time::Duration::from_millis(chap_duration_ms),
             });
        }


        titles.push(Title {
            number: index + 1, // HandBrake titles are usually 1-indexed in its non-JSON output, JSON might be 0-indexed in array
            duration: std::time::Duration::from_millis(duration_ms),
            size: DvdSize { width, height },
            chapters,
            description,
        });
    }
    Ok(titles)
}


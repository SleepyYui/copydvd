use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{info, warn};

use crate::app::startup::auto_detect_dvd;
use crate::app::state::{AppState, AppStatus};
use crate::cli;
use crate::config::Config;
use crate::dvd::types::Dvd;
use crate::error::Result;
#[cfg(feature = "gui")]
use crate::gui;
use crate::upload;

/// Run the application
pub async fn run() -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Create application state
    let app_state = Arc::new(Mutex::new(AppState::new(config.clone())));

    // Parse CLI arguments
    let args = cli::parse_args();

    if args.cli_mode {
        // CLI mode was explicitly requested
        info!("Starting in CLI mode (forced by --cli flag)");
        run_cli(app_state, args).await?;
    } else {
        // Try GUI mode first (now using egui instead of iced)
        #[cfg(feature = "gui")]
        {
            info!("Starting in GUI mode (default - using egui)");
            match gui::run() {
                Ok(_) => {
                    info!("GUI mode completed successfully");
                }
                Err(e) => {
                    warn!("GUI failed to start ({}), falling back to CLI mode", e);
                    run_cli(app_state, args).await?;
                }
            }
        }

        #[cfg(not(feature = "gui"))]
        {
            // GUI feature not available, fall back to CLI
            info!("Starting in CLI mode (GUI feature not enabled)");
            run_cli(app_state, args).await?;
        }
    }

    Ok(())
}

/// Run the application with specified CLI arguments (for direct invocation)
#[allow(dead_code)]
pub async fn run_with_args(args: cli::Args) -> Result<()> {
    // Load configuration
    let config = Config::load()?;

    // Create application state
    let app_state = Arc::new(Mutex::new(AppState::new(config.clone())));

    // Run in CLI mode
    info!("Starting in CLI mode with specified arguments");
    run_cli(app_state, args).await
}

/// Run the application in CLI mode
async fn run_cli(app_state: Arc<Mutex<AppState>>, args: cli::Args) -> Result<()> {
    // Set initial status
    {
        let mut state_guard = app_state.lock().await;
        state_guard.status = AppStatus::Scanning;
    } // Guard drops here

    // Determine DVD path, either from args or auto-detect
    let dvd_path = match args.input {
        Some(ref path) => path.clone(),
        None => {
            info!("No DVD path specified, attempting to auto-detect");
            match auto_detect_dvd() {
                Ok(path) => path,
                Err(e) => {
                    // Set error state
                    let mut state_guard = app_state.lock().await;
                    state_guard.status = AppStatus::Error(e.to_string());
                    return Err(e);
                }
            }
        }
    };

    info!("Using DVD path: {}", dvd_path.display());

    // Get config
    let config_clone = {
        let state_guard = app_state.lock().await;
        state_guard.config.clone()
    };

    // Create DVD object
    let dvd = Dvd::new(dvd_path, config_clone).await?;
    let dvd_arc = Arc::new(Mutex::new(dvd));

    {
        let mut state_guard_update = app_state.lock().await;
        state_guard_update.dvd = Some(Arc::clone(&dvd_arc));
    }

    let tasks = {
        let mut dvd_lock_guard = dvd_arc.lock().await;
        dvd_lock_guard.scan_titles().await?;

        let output_dir = args
            .output
            .unwrap_or_else(|| dvd_lock_guard.config.output_dir.clone());

        let selected_titles_numbers = if args.main_feature {
            info!("Selecting main feature only");
            dvd_lock_guard
                .find_main_feature()
                .map(|title| vec![title.number])
        } else if let Some(titles_args) = args.titles {
            info!("Selecting titles: {:?}", titles_args);
            Some(titles_args)
        } else {
            info!("Selecting all titles");
            None
        };

        // Ensure selected_titles_numbers is Option<Vec<usize>> for create_rip_tasks
        let final_selected_titles = match selected_titles_numbers {
            Some(numbers) if !numbers.is_empty() => Some(numbers),
            Some(_) => {
                // Empty vec from main_feature if not found
                info!("No specific titles selected or main feature not found, ripping all titles.");
                None
            }
            None => None, // Rip all
        };

        let chapter_split = args
            .chapter_split
            .unwrap_or(dvd_lock_guard.config.chapter_split);
        dvd_lock_guard.create_rip_tasks(output_dir, final_selected_titles, chapter_split)
    };

    let task_count = tasks.len();
    if task_count == 0 {
        info!("No tasks to perform.");
        let mut final_state_guard = app_state.lock().await;
        final_state_guard.status = AppStatus::Completed; // Or an appropriate status
        return Ok(());
    }
    info!("Created {} ripping tasks", task_count);

    {
        let mut state_guard_tasks = app_state.lock().await;
        state_guard_tasks.rip_tasks = tasks.clone();
        state_guard_tasks.status = AppStatus::Ripping {
            completed: 0,
            total: task_count,
        };
    }

    let (tx, mut rx) = mpsc::channel(task_count.max(1));
    let thread_count = {
        let state_guard_threads = app_state.lock().await;
        args.threads
            .unwrap_or(state_guard_threads.config.thread_count)
    };

    info!("Using {} threads for ripping", thread_count);

    for (i, task) in tasks.iter().enumerate() {
        let dvd_clone_task: Arc<Mutex<Dvd>> = Arc::clone(&dvd_arc);
        let task_clone_exec = task.clone();
        let tx_clone = tx.clone();
        let _app_state_clone_task = Arc::clone(&app_state);

        tokio::spawn(async move {
            let result = dvd_clone_task
                .lock()
                .await
                .rip_title(&task_clone_exec)
                .await;
            // Send result and then update progress via handle_task_result
            if tx_clone.send((i, result)).await.is_err() {
                warn!(
                    "Receiver dropped, could not send task result for task index {}",
                    i
                );
            }
        });

        // This batching logic is a bit simplistic. For true parallelism up to thread_count,
        // you'd typically use a semaphore or a pool, or rely on tokio's scheduler with a bounded number of workers.
        // The current mpsc channel and recv loop will process results as they come in.
        // The original logic of waiting every `thread_count` tasks might not be ideal for maximizing throughput.
        // We'll let the tasks run and collect results in the loop below.
    }
    drop(tx);

    while let Some((task_index, result)) = rx.recv().await {
        handle_task_result(Arc::clone(&app_state), task_index, result).await;
    }

    // Final check if all tasks completed (e.g. if any failed, status might not be Completed yet)
    let final_status = app_state.lock().await.status.clone();
    if let AppStatus::Ripping { completed, total } = final_status {
        if completed == total {
            app_state.lock().await.status = AppStatus::Completed;
        } else {
            // Not all tasks might have reported success, or some failed.
            // The status should reflect this based on handle_task_result logic.
            warn!(
                "Ripping ended but not all tasks reported completion. Status: {:?}",
                final_status
            );
        }
    }

    let (eject_after_rip, server_config_opt) = {
        let dvd_lock_guard_final = dvd_arc.lock().await;
        (
            dvd_lock_guard_final.config.eject_after_rip,
            dvd_lock_guard_final.config.server.clone(),
        )
    };

    if eject_after_rip {
        info!("Ejecting DVD");
        if let Err(e) = dvd_arc.lock().await.eject().await {
            warn!("Failed to eject DVD: {}", e);
        }
    }

    if args.upload {
        if let Some(server_config) = server_config_opt {
            info!("Uploading ripped files to server");
            let movie_name = {
                dvd_arc
                    .lock()
                    .await
                    .find_main_feature()
                    .and_then(|title| title.description.clone()) // Use description if available
                    .unwrap_or_else(|| "DVD_Rip".to_string()) // Fallback name
            };

            for task_item in &tasks {
                if task_item.output_path.exists() {
                    let upload_task = upload::create_upload_task(
                        &task_item.output_path,
                        &server_config,
                        &movie_name,
                    );
                    info!(
                        "Uploading {} to {}",
                        task_item.output_path.display(),
                        upload_task.remote_path
                    );
                    match upload::upload_file(&upload_task).await {
                        Ok(_) => info!("Upload complete for {}", task_item.output_path.display()),
                        Err(e) => warn!(
                            "Upload failed for {}: {}",
                            task_item.output_path.display(),
                            e
                        ),
                    }
                }
            }
        } else {
            warn!("Upload requested but no server configuration found");
        }
    }

    // Ensure status is set to Completed if not already set by the last task handler
    let mut state_guard_complete = app_state.lock().await;
    if state_guard_complete.status != AppStatus::Error("".to_string()) {
        // Avoid overwriting an error state
        if let AppStatus::Ripping { completed, total } = state_guard_complete.status {
            if completed == total {
                state_guard_complete.status = AppStatus::Completed;
            }
        } else if state_guard_complete.status == AppStatus::Idle && task_count == 0 {
            // No tasks were run
            state_guard_complete.status = AppStatus::Completed;
        }
    }

    info!(
        "CLI run finished. Final status: {:?}",
        state_guard_complete.status
    );
    Ok(())
}

async fn handle_task_result(
    app_state: Arc<Mutex<AppState>>,
    task_index: usize,
    result: Result<()>,
) {
    let mut state = app_state.lock().await;
    if let AppStatus::Ripping { completed, total } = &mut state.status {
        *completed += 1;
        if let Err(e) = result {
            warn!("Task {} failed: {}", task_index, e);
            // Transition to an error state or mark partial success?
            // For now, we continue counting completed tasks, but the overall process might be considered failed.
            // A more robust error handling might set a general error flag.
            state.status = AppStatus::Error(format!("Task {} failed: {}", task_index, e));
            return; // Exit if a task fails to prevent marking as Completed
        }

        if *completed >= *total {
            state.status = AppStatus::Completed;
            info!("All ripping tasks completed.");
        } else {
            info!("Task {}/{} completed.", *completed, *total);
        }
    } else {
        warn!("Received task result but app not in Ripping state or state mismatch. Current state: {:?}, Task index: {}", state.status, task_index);
    }
}

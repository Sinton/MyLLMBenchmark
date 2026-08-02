pub mod benchmark;
pub mod commands;
pub mod config;
pub mod data;
pub mod db;
pub mod domain;
pub mod endpoint_probe;
pub mod error;
pub mod mock;
pub mod models;
pub mod report;
pub mod security;
pub mod services;
pub mod state;
pub mod storage;
pub mod tasks;
pub mod telemetry;

use commands::{
    append_dataset_samples, create_dataset_sample, create_provider, delete_benchmark_request_logs,
    delete_dataset, delete_dataset_sample, delete_dataset_samples_batch,
    delete_endpoint_probe_batch, delete_provider, diagnose_provider, export_dataset, export_report,
    generate_report, get_app_config, get_benchmark_request_log_detail, get_benchmark_task,
    get_dashboard_summary, get_endpoint_probe_batch_detail, get_endpoint_probe_run_detail,
    get_provider_diagnostics, get_report_detail, import_dataset, import_providers,
    list_benchmark_request_logs_page, list_benchmark_ticks, list_dataset_samples_page,
    list_datasets, list_endpoint_probe_batches_page, list_provider_models, list_providers,
    list_reports, preview_dataset_samples, promote_endpoint_probe_target,
    scan_endpoint_probe_models, scan_provider_models, start_benchmark, start_endpoint_probe,
    stop_benchmark, stop_endpoint_probe, test_provider_connection, update_app_config,
    update_dataset, update_dataset_sample, update_provider, validate_dataset_samples,
};
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let data_dir = app.path().app_data_dir()?;
            let state = tauri::async_runtime::block_on(AppState::initialize(config_dir, data_dir))?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard_summary,
            get_app_config,
            update_app_config,
            list_providers,
            create_provider,
            import_providers,
            update_provider,
            delete_provider,
            test_provider_connection,
            list_provider_models,
            scan_provider_models,
            diagnose_provider,
            get_provider_diagnostics,
            list_datasets,
            import_dataset,
            update_dataset,
            delete_dataset,
            preview_dataset_samples,
            list_dataset_samples_page,
            create_dataset_sample,
            update_dataset_sample,
            delete_dataset_sample,
            append_dataset_samples,
            delete_dataset_samples_batch,
            export_dataset,
            validate_dataset_samples,
            start_benchmark,
            stop_benchmark,
            get_benchmark_task,
            list_benchmark_ticks,
            list_benchmark_request_logs_page,
            get_benchmark_request_log_detail,
            delete_benchmark_request_logs,
            generate_report,
            list_reports,
            get_report_detail,
            export_report,
            start_endpoint_probe,
            stop_endpoint_probe,
            scan_endpoint_probe_models,
            promote_endpoint_probe_target,
            list_endpoint_probe_batches_page,
            get_endpoint_probe_batch_detail,
            get_endpoint_probe_run_detail,
            delete_endpoint_probe_batch
        ])
        .run(tauri::generate_context!())
        .expect("error while running MyLLMBenchmark");
}

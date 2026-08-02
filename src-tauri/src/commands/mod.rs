mod benchmark;
mod config;
mod dashboard;
mod dataset;
mod provider;
mod report;
mod site_probe;

pub use benchmark::{
    delete_benchmark_request_logs, get_benchmark_request_log_detail, get_benchmark_task,
    list_benchmark_request_logs_page, list_benchmark_ticks, start_benchmark, stop_benchmark,
};
pub use config::{get_app_config, update_app_config};
pub use dashboard::get_dashboard_summary;
pub use dataset::{
    append_dataset_samples, create_dataset_sample, delete_dataset, delete_dataset_sample,
    delete_dataset_samples_batch, export_dataset, import_dataset, list_dataset_samples_page,
    list_datasets, preview_dataset_samples, update_dataset, update_dataset_sample,
    validate_dataset_samples,
};
pub use provider::{
    create_provider, delete_provider, diagnose_provider, get_provider_diagnostics,
    list_provider_models, list_providers, scan_provider_models, test_provider_connection,
    update_provider,
};
pub use report::{export_report, generate_report, get_report_detail, list_reports};
pub use site_probe::{
    delete_site_probe_run, get_site_probe_run_detail, list_site_probe_runs_page, run_site_probe,
    scan_site_probe_models,
};

use crate::error::AppError;

pub(crate) fn error_to_string(error: AppError) -> String {
    error.user_message()
}

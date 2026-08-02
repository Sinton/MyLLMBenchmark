use crate::error::{AppError, AppResult};
use crate::models::{CreateProviderInput, UpdateProviderInput};
use crate::security::{mask_base_url, mask_secret};

#[derive(Debug, Clone)]
pub struct ExistingProviderConfig {
    pub base_url: String,
    pub base_url_masked: String,
    pub api_key_masked: String,
    pub api_key_plaintext: String,
    pub interface_type: String,
    pub status: String,
    pub last_checked_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PreparedProviderCreate {
    pub name: String,
    pub base_url: String,
    pub base_url_masked: String,
    pub api_key_masked: String,
    pub api_key_plaintext: String,
    pub interface_type: String,
}

#[derive(Debug, Clone)]
pub struct PreparedProviderUpdate {
    pub name: String,
    pub base_url: String,
    pub base_url_masked: String,
    pub api_key_masked: String,
    pub api_key_plaintext: String,
    pub interface_type: String,
    pub status: String,
    pub last_checked_at: Option<String>,
    pub config_changed: bool,
}

pub fn prepare_provider_create(input: CreateProviderInput) -> AppResult<PreparedProviderCreate> {
    let name = normalize_required(&input.name, "provider name is required")?;
    let base_url = normalize_base_url(&input.base_url)?;
    let interface_type = normalize_interface_type(&input.interface_type);

    Ok(PreparedProviderCreate {
        name,
        base_url: base_url.clone(),
        base_url_masked: mask_base_url(&base_url),
        api_key_masked: mask_secret(input.api_key.as_deref()),
        api_key_plaintext: input.api_key.unwrap_or_default().trim().to_string(),
        interface_type,
    })
}

pub fn prepare_provider_update(
    input: UpdateProviderInput,
    current: ExistingProviderConfig,
) -> AppResult<PreparedProviderUpdate> {
    let name = normalize_required(&input.name, "provider name is required")?;
    let requested_base_url = normalize_base_url(&input.base_url)?;
    let interface_type = normalize_interface_type(&input.interface_type);

    let base_url_changed =
        requested_base_url != current.base_url && requested_base_url != current.base_url_masked;
    let base_url = if base_url_changed {
        requested_base_url
    } else {
        current.base_url
    };

    let (api_key_masked, api_key_plaintext, api_key_changed) =
        if let Some(requested_api_key) = input.api_key.as_deref() {
            let requested_api_key = requested_api_key.trim();
            if requested_api_key == current.api_key_masked {
                (current.api_key_masked, current.api_key_plaintext, false)
            } else {
                (
                    mask_secret(Some(requested_api_key)),
                    requested_api_key.to_string(),
                    requested_api_key != current.api_key_plaintext,
                )
            }
        } else {
            (current.api_key_masked, current.api_key_plaintext, false)
        };

    let interface_changed = interface_type != current.interface_type;
    let config_changed = base_url_changed || api_key_changed || interface_changed;

    Ok(PreparedProviderUpdate {
        name,
        base_url: base_url.clone(),
        base_url_masked: mask_base_url(&base_url),
        api_key_masked,
        api_key_plaintext,
        interface_type,
        status: if config_changed {
            "unchecked".to_string()
        } else {
            current.status
        },
        last_checked_at: if config_changed {
            None
        } else {
            current.last_checked_at
        },
        config_changed,
    })
}

fn normalize_required(value: &str, message: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::validation(message));
    }
    Ok(trimmed.to_string())
}

pub fn normalized_provider_identity(base_url: &str, interface_type: &str) -> (String, String) {
    (
        base_url.trim().trim_end_matches('/').to_string(),
        normalize_interface_type(interface_type),
    )
}

fn normalize_base_url(value: &str) -> AppResult<String> {
    let value = normalize_required(value, "Base URL is required")?;
    Ok(value.trim_end_matches('/').to_string())
}

fn normalize_interface_type(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "OpenAI".to_string()
    } else {
        trimmed.to_string()
    }
}

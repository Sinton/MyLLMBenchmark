pub mod secrets;

pub fn mask_secret(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "未配置".to_string();
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "未配置".to_string();
    }

    trimmed.to_string()
}

pub fn mask_base_url(base_url: &str) -> String {
    base_url.trim().to_string()
}

pub mod secrets;

const MASK: &str = "********************";

pub fn mask_secret(value: Option<&str>) -> String {
    let Some(value) = value else {
        return "未配置".to_string();
    };

    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "未配置".to_string();
    }

    if trimmed == "未配置" {
        return trimmed.to_string();
    }

    let chars = trimmed.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        return "********".to_string();
    }

    let suffix = chars[chars.len() - 4..].iter().collect::<String>();
    let prefix = if trimmed.starts_with("sk-") {
        "sk-"
    } else {
        ""
    };
    format!("{prefix}{MASK}{suffix}")
}

pub fn mask_base_url(base_url: &str) -> String {
    base_url.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::mask_secret;

    #[test]
    fn secret_mask_keeps_only_a_recognizable_prefix_and_suffix() {
        let secret = "sk-example-secret-abcd";
        let masked = mask_secret(Some(secret));

        assert_eq!(masked, "sk-********************abcd");
        assert!(!masked.contains(secret));
    }

    #[test]
    fn short_or_missing_secrets_are_not_exposed() {
        assert_eq!(mask_secret(None), "未配置");
        assert_eq!(mask_secret(Some("")), "未配置");
        assert_eq!(mask_secret(Some("abcd")), "********");
    }
}

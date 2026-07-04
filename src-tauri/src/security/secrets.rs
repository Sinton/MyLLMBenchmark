#[derive(Debug, Clone)]
pub struct SecretRef {
    pub namespace: String,
    pub key: String,
}

impl SecretRef {
    pub fn provider_api_key(provider_id: &str) -> Self {
        Self {
            namespace: "provider_api_key".to_string(),
            key: provider_id.to_string(),
        }
    }
}

pub trait SecretStore: Send + Sync {
    fn put_secret(&self, secret_ref: &SecretRef, value: &str) -> anyhow::Result<()>;
    fn get_secret(&self, secret_ref: &SecretRef) -> anyhow::Result<Option<String>>;
    fn delete_secret(&self, secret_ref: &SecretRef) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct UnavailableSecretStore {
    reason: String,
}

impl UnavailableSecretStore {
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    fn unavailable(&self) -> anyhow::Error {
        anyhow::anyhow!("secret store is unavailable: {}", self.reason)
    }
}

impl Default for UnavailableSecretStore {
    fn default() -> Self {
        Self::new("system keychain / stronghold integration is not enabled")
    }
}

impl SecretStore for UnavailableSecretStore {
    fn put_secret(&self, _secret_ref: &SecretRef, _value: &str) -> anyhow::Result<()> {
        Err(self.unavailable())
    }

    fn get_secret(&self, _secret_ref: &SecretRef) -> anyhow::Result<Option<String>> {
        Err(self.unavailable())
    }

    fn delete_secret(&self, _secret_ref: &SecretRef) -> anyhow::Result<()> {
        Err(self.unavailable())
    }
}

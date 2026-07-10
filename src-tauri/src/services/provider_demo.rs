use crate::domain::model_catalog::{model_templates_for_interface, CatalogFlavor};
use crate::domain::model_type::default_capabilities;
use crate::models::{DiscoveredModel, ProviderConnectionConfig, ProviderConnectionResult};

pub(super) fn test_connection(
    config: &ProviderConnectionConfig,
    checked_at: String,
) -> ProviderConnectionResult {
    ProviderConnectionResult {
        provider_id: config.id.clone(),
        ok: true,
        status: "online".to_string(),
        message: format!("演示连接测试通过（Mock 引擎）：{}", config.name),
        checked_at,
    }
}

pub(super) fn discover_models(config: &ProviderConnectionConfig) -> Vec<DiscoveredModel> {
    model_templates_for_interface(&config.interface_type, CatalogFlavor::Demo)
        .into_iter()
        .map(|template| DiscoveredModel {
            name: template.name,
            model_type: template.model_type.clone(),
            capabilities: if template.capabilities.is_empty() {
                default_capabilities(&template.model_type)
            } else {
                template.capabilities
            },
            supports_streaming: template.supports_streaming,
            recommended_concurrency: template.recommended_concurrency,
        })
        .collect()
}

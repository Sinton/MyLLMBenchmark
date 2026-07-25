pub mod real;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkEngineKind {
    Mock,
    OpenAICompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderProtocol {
    OpenAICompatible,
    OpenAIResponses,
    Anthropic,
    Gemini,
    JinaRerank,
}

pub fn executable_engine() -> BenchmarkEngineKind {
    BenchmarkEngineKind::Mock
}

pub fn planned_real_protocols() -> &'static [ProviderProtocol] {
    &[
        ProviderProtocol::OpenAICompatible,
        ProviderProtocol::OpenAIResponses,
        ProviderProtocol::Anthropic,
        ProviderProtocol::Gemini,
        ProviderProtocol::JinaRerank,
    ]
}

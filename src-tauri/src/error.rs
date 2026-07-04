use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),

    #[error("{0} not found")]
    NotFound(String),

    #[error("{0}")]
    InvalidTaskState(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error(transparent)]
    Unexpected(anyhow::Error),
}

impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        match error.downcast::<AppError>() {
            Ok(app_error) => app_error,
            Err(error) => Self::Unexpected(error),
        }
    }
}

impl AppError {
    pub fn user_message(&self) -> String {
        self.to_string()
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }

    pub fn invalid_task_state(message: impl Into<String>) -> Self {
        Self::InvalidTaskState(message.into())
    }

    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage(message.into())
    }
}

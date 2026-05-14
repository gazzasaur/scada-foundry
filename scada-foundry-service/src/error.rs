use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScadaFoundryError {
    #[error("ApplicationError: {0}")]
    ApplicationError(String),
}

pub(crate) fn to_app_error<T: Debug>(message: &str) -> impl FnOnce(T) -> ScadaFoundryError {
    move |error| ScadaFoundryError::ApplicationError(format!("{}: {:?}", message, error))
}

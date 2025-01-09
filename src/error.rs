use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Error, Debug)]
pub enum Error {
    #[error("No input device available")]
    NoInputDevice,
    #[error("Failed to play stream")]
    PlayStreamError(#[from] cpal::PlayStreamError),
    #[error("Request error")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to build input stream")]
    StreamBuildError(#[from] cpal::BuildStreamError),
    #[error("Failed to create WAV writer")]
    WavWriterError(#[from] hound::Error),
}

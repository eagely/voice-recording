use crate::error::Result;
use reqwest::blocking::Client;
use std::time::Duration;

pub struct TranscriptionClient {
    endpoint: String,
    timeout: Duration,
}

impl TranscriptionClient {
    pub fn new(endpoint: String, timeout_secs: u64) -> Self {
        Self {
            endpoint,
            timeout: Duration::from_secs(timeout_secs),
        }
    }

    pub fn transcribe(&self, wav_data: Vec<u8>) -> Result<String> {
        let client = Client::builder()
            .timeout(self.timeout)
            .build()
            .expect("Failed to build HTTP client");

        let response = client
            .post(&self.endpoint)
            .header("Content-Type", "audio/wav")
            .body(wav_data)
            .send()?
            .error_for_status()?
            .text()?;

        Ok(response.trim().to_owned())
    }
}

impl Default for TranscriptionClient {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:3000/transcribe".to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

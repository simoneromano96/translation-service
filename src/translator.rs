use std::{
    path::PathBuf,
    sync::{
        atomic::AtomicBool,
        mpsc::{self},
    },
    sync::{atomic::Ordering, Arc},
    thread::{self, JoinHandle},
};

use rust_bert::{
    pipelines::{
        common::{ModelResource, ModelType},
        translation::{Language, TranslationConfig, TranslationModel, TranslationModelBuilder},
    },
    resources::LocalResource,
    RustBertError,
};
use tch::Device;
use tokio::sync::oneshot;

use thiserror::Error;
use tracing::debug;
use utoipa::ToSchema;

use crate::settings::SETTINGS;

/// Custom error type for the Translator.
#[derive(Error, Debug, ToSchema)]
pub enum TranslatorError {
    #[error("RustBert error: {0}")]
    RustBertError(#[from] RustBertError),

    #[error("Failed to join the translation thread")]
    ThreadJoinError,

    #[error("Failed to send a message")]
    SendError,
}

/// Result type for translation operations using the Translator.
type TranslationModelResult = Result<Vec<String>, TranslatorError>;

/// Message type for the internal channel, used to pass texts and return value senders.
type Message = (String, oneshot::Sender<TranslationModelResult>);

/// Represents the direction of translation.
#[derive(Debug, Clone, Copy)]
pub enum TranslationDirection {
    EnglishToItalian,
    ItalianToEnglish,
}

/// The Translator struct is used to facilitate text translation.
#[derive(Debug)]
pub struct Translator {
    sender: mpsc::SyncSender<Message>,
    handle: JoinHandle<Result<(), TranslatorError>>,
    stop_flag: Arc<AtomicBool>, // Flag to signal thread termination
}

/// The default buffer length for the message channel.
const BUFFER_LENGTH: usize = 100;

impl Translator {
    /// Spawns a new Translator instance for the specified translation direction.
    ///
    /// # Arguments
    ///
    /// * `direction` - The direction of translation (e.g., English to Italian).
    ///
    /// # Returns
    ///
    /// A `Translator` instance.
    #[tracing::instrument]
    pub fn spawn(direction: TranslationDirection) -> Self {
        debug!("Spawning a new Translator instance for {:?}", direction);
        let (sender, receiver) = mpsc::sync_channel(BUFFER_LENGTH);

        // Create a stop flag shared between the main thread and the translator thread.
        let stop_flag = Arc::new(AtomicBool::new(false));

        let stop_flag_clone = Arc::clone(&stop_flag);
        let handle = thread::spawn(move || Self::runner(receiver, direction, stop_flag_clone));

        Self {
            sender,
            handle,
            stop_flag,
        }
    }

    /// The translation runner function that handles translation requests.
    #[tracing::instrument]
    fn runner(
        receiver: mpsc::Receiver<Message>,
        direction: TranslationDirection,
        stop_flag: Arc<AtomicBool>,
    ) -> Result<(), TranslatorError> {
        debug!("Initialising model");

        let mut base_path = PathBuf::from(&SETTINGS.path);

        // Create a translation model based on the specified direction
        let (source_lang, target_lang) = match direction {
            TranslationDirection::EnglishToItalian => {
                base_path.push("opus-mt-en-ROMANCE");
                (Language::English, Language::Italian)
            }
            TranslationDirection::ItalianToEnglish => {
                base_path.push("opus-mt-ROMANCE-en");
                (Language::Italian, Language::English)
            }
        };

        debug!("Derived base_path {base_path:?}");

        let model_resource = LocalResource {
            local_path: base_path.join("rust_model.ot"),
        };
        let config_resource = LocalResource {
            local_path: base_path.join("config.json"),
        };
        let vocab_resource = LocalResource {
            local_path: base_path.join("vocab.json"),
        };

        let translation_config = TranslationConfig::new(
            ModelType::Marian,
            ModelResource::Torch(Box::new(model_resource)),
            config_resource,
            vocab_resource,
            None,
            vec![source_lang],
            vec![target_lang],
            Device::cuda_if_available(),
        );
        debug!("Derived translation_config");

        let model = TranslationModel::new(translation_config)?;
        debug!("Initialised model");

        // Process incoming translation requests
        while !stop_flag.load(Ordering::Relaxed) {
            match receiver.try_recv() {
                Ok((text, sender)) => {
                    // Add trace for receiving a translation request
                    debug!("Received translation request: {:?}", text);

                    let translation = model
                        .translate(&[&text], source_lang, target_lang)
                        .map_err(|error| TranslatorError::RustBertError(error));

                    // Add trace for processing the translation request
                    debug!("Processing translation request: {:?}", &text);

                    sender
                        .send(translation)
                        .map_err(|_| TranslatorError::SendError)?;

                    // Add trace for completing the translation request
                    debug!("Completed translation request: {:?}", &text);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No messages in the channel, continue processing
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Channel disconnected, exit the loop
                    break;
                }
            }
        }

        Ok(())
    }

    /// Translates the given text and returns the translation result.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to be translated.
    ///
    /// # Returns
    ///
    /// A `TranslationModelResult` representing the translation result.
    #[tracing::instrument]
    pub async fn translate(&self, text: String) -> TranslationModelResult {
        // Add trace for initiating a translation request
        debug!("Initiating translation request: {:?}", &text);

        let (sender, receiver) = oneshot::channel();
        self.sender
            .send((text.clone(), sender))
            .map_err(|_| TranslatorError::ThreadJoinError)?;

        let translation_result = receiver
            .await
            .map_err(|_| TranslatorError::ThreadJoinError)?;

        // Add trace for completing the translation request
        debug!("Completed translation request: {:?}", text);

        translation_result
    }

    /// Stops the translator thread gracefully and joins it.
    pub fn stop(self) -> Result<(), TranslatorError> {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.handle
            .join()
            .map_err(|_| TranslatorError::ThreadJoinError)??;
        Ok(())
    }
}

/// Module containing tests for the Translator.
#[cfg(test)]
mod tests {
    use super::*;

    /// Test case for English to Italian translation.
    #[actix_web::test]
    async fn test_english_to_italian_translation() {
        // Create an instance of Translator for English to Italian translation
        let en_it = Translator::spawn(TranslationDirection::EnglishToItalian);

        // Translate the text and assert the result
        let translation_result = en_it
            .translate("Hello".to_string())
            .await
            .unwrap()
            .join(" ")
            .trim()
            .to_owned();

        assert_eq!(translation_result, "Ciao.");

        // Stop the translator thread gracefully
        en_it.stop().unwrap();
    }

    /// Test case for Italian to English translation.
    #[actix_web::test]
    async fn test_italian_to_english_translation() {
        // Create an instance of Translator for Italian to English translation
        let it_en = Translator::spawn(TranslationDirection::ItalianToEnglish);

        // Translate the text and assert the result
        let translation = it_en
            .translate("Ciao".to_string())
            .await
            .unwrap()
            .join(" ")
            .trim()
            .to_owned();

        assert_eq!(translation, "Hello.");

        // Stop the translator thread gracefully
        it_en.stop().unwrap();
    }
}

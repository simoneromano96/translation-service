use tracing::debug;

use crate::translator::{TranslationDirection, Translator};

/// Application context
#[derive(Debug)]
pub struct AppContext {
    /// English to Italian model translation
    pub en_it: Translator,
    /// Italian to English model translation
    pub it_en: Translator,
}

/// Prepares an instance of the Application context
#[tracing::instrument]
pub fn prepare_app_context() -> AppContext {
    // Add trace for initializing the AppContext
    debug!("Initializing AppContext");

    // Spawn a new English to Italian translation process and get its handle and resulting model
    let en_it = Translator::spawn(TranslationDirection::EnglishToItalian);

    // Add trace for spawning the English to Italian translation process
    debug!("Spawning English to Italian Translator process");

    // Spawn a new Italian to English translation process and get its handle and resulting model
    let it_en = Translator::spawn(TranslationDirection::ItalianToEnglish);

    // Add trace for spawning the Italian to English translation process
    debug!("Spawning Italian to English Translator process");

    AppContext { en_it, it_en }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn prepare_app_context_should_not_panic() {
        prepare_app_context();
    }
}

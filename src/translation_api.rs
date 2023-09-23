use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

use crate::{
    context::{self, AppContext},
    translator::TranslatorError,
};

use actix_web::{
    http::StatusCode,
    post,
    web::{Data, Json, ServiceConfig},
    HttpResponse, ResponseError,
};

/// Enum representing supported languages for translation.
#[derive(Deserialize, Serialize, ToSchema)]
pub enum SupportedLanguages {
    Italian,
    English,
}

/// Enum representing possible error responses.
#[derive(Error, Debug)]
pub enum ErrorResponse {
    #[error("An unspecified internal error occurred: {0}")]
    TranslatorError(#[from] TranslatorError),
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::TranslatorError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

/// Struct representing a translation request.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranslationRequest {
    /// The text to be translated.
    #[schema(example = "Ciao, come stai?")]
    pub text: String,
    /// The source language of the text.
    #[schema(example = "Italian")]
    pub from_language: SupportedLanguages,
}

/// Struct representing a translation response.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResponse {
    /// The translated text.
    #[schema(example = "Hello, how are you?")]
    pub translation: String,
}

/// Translate a text from a specified language to English or Italian.
///
/// This function takes a translation request as a JSON payload and translates the given text
/// based on the specified source language to either English or Italian.
///
/// # Parameters
///
/// - `translation`: JSON payload containing the text and source language for translation.
///
/// # Returns
///
/// Returns a JSON response with the translated text or an error response if translation fails.
///
/// # Errors
///
/// If translation encounters an error, it will return an error response with an appropriate status code.
///
/// # Example
///
/// ```
/// POST /translate
/// {
///     "text": "Ciao, come stai?",
///     "fromLanguage": "Italian"
/// }
/// ```
///
/// Response:
/// ```
/// {
///     "translation": "Hello, how are you?"
/// }
/// ```
#[utoipa::path(
    request_body = TranslationRequest,
    responses(
        (status = 200, description = "Translation result", body = [TranslationResponse])
    )
)]
#[post("/translate")]
async fn translate(
    app_context: Data<context::AppContext>,
    translation: Json<TranslationRequest>,
) -> Result<Json<TranslationResponse>, ErrorResponse> {
    let TranslationRequest {
        text,
        from_language,
    } = translation.into_inner();

    let translation = match from_language {
        SupportedLanguages::Italian => app_context.it_en.translate(text).await,
        SupportedLanguages::English => app_context.en_it.translate(text).await,
    }?
    .join(" ")
    .trim()
    .to_owned();

    Ok(Json(TranslationResponse { translation }))
}

/// Configure Actix Web service with the provided application context.
///
/// This function configures an Actix Web service with the provided `AppContext`, allowing it to
/// handle translation requests.
///
/// # Parameters
///
/// - `app_context`: Application context containing translation services.
///
/// # Returns
///
/// Returns a closure that configures the service with the specified context and translation endpoint.
///
/// # Example
///
/// ```rust
/// let app_context = // Create an instance of AppContext with translation services.
///
/// let service_config = App::new()
///     .configure(configure(app_context.clone()));
/// ```
#[tracing::instrument]
pub fn configure(app_context: Data<AppContext>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config.app_data(app_context).service(translate);
    }
}

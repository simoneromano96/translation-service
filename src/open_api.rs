use crate::translation_api;
use utoipa::OpenApi;

use once_cell::sync::Lazy;

#[derive(OpenApi)]
#[openapi(
    paths(
        translation_api::translate,
    ),
    components(
        schemas(
          translation_api::TranslationRequest, 
          translation_api::SupportedLanguages, 
          translation_api::TranslationResponse, 
          // translation_api::ErrorResponse,
        )
    ),
    tags(
        (name = "translation_api", description = "Translation endpoints.")
    ),
)]
pub struct ApiDoc;

pub static OPEN_API: Lazy<utoipa::openapi::OpenApi> = Lazy::new(|| ApiDoc::openapi());

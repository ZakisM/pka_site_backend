use axum::Router;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::docs;

pub fn router() -> Router<AppState> {
    let openapi = docs::openapi();

    Router::new().merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", openapi))
}

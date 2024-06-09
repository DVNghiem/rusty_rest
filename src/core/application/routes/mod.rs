use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};
use crate::core::application::controllers::health_check;

pub fn core_routing(cfg: &mut web::ServiceConfig, path: &str) {
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "BasicAPI",
            version = "0.1.0",
        ),
        paths(
            health_check::health_check
        ),
        components(
            schemas(
            )
        ),
        tags((name = "BasicAPI", description = "A very Basic API")),
    )]
    struct ApiDoc;
    let url = Url::with_primary("Rusty rest", "/api-docs/openapi.json", true);
    cfg.service(
        web::scope(path)
            .service(
                SwaggerUi::new("/docs/{_:.*}").url(url, ApiDoc::openapi()),
            )
            .route("/health_check", web::get().to(health_check::health_check)),
    );
}
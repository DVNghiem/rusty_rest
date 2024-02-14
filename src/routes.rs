use crate::controllers::health_check;
use crate::schema::health_check::HealthCheckSchema;
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

pub fn routing(cfg: &mut web::ServiceConfig) {
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
                HealthCheckSchema,
            )
        ),
        tags((name = "BasicAPI", description = "A very Basic API")),
    )]
    struct ApiDoc;
    let url = Url::with_primary("Rusty rest", "/api-docs/openapi.json", true);
    cfg.service(
        web::scope("")
            .service(
                SwaggerUi::new("/docs/{_:.*}").url(url, ApiDoc::openapi()),
            )
            .route("/health_check", web::get().to(health_check::health_check)),
    );
}

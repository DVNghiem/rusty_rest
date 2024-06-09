use crate::apis::routes::{api_routing, ApiDoc};
use crate::core::application::routes::{core_routing, CoreApiDoc};
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

pub fn router(cfg: &mut web::ServiceConfig) {
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "BasicAPI",
            version = "0.1.0",
        ),
        paths(
        ),
        components(
            schemas(
            )
        ),
        tags((name = "BasicAPI", description = "A very Basic API")),
    )]
    struct MainApiDoc;
    let mut openapi = MainApiDoc::openapi();
    openapi.merge(CoreApiDoc::openapi());
    openapi.merge(ApiDoc::openapi());
    let url = Url::with_primary("Rusty rest", "/api-docs/openapi.json", true);
    cfg.service(SwaggerUi::new("/docs/{_:.*}").url(url, openapi));
    core_routing(cfg, "core");
    api_routing(cfg, "");
}

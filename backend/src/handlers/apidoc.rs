use crate::Meta;
use crate::MetaServerName;
use crate::handlers::healthz::{__path_healthz, HealthData, HealthResponse, HealthStatus};
use crate::handlers::sites::{
    __path_create_site, __path_list_sites, __path_preview_site, __path_stop_preview,
    CreateSiteRequest, SiteData, SiteListResponse, SiteResponse
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Blog Engine API", version = env!("CARGO_PKG_VERSION"), description = "API for managing Astro blog sites"),
    paths(healthz, list_sites, create_site, preview_site, stop_preview),
    components(schemas(
        HealthResponse, HealthData, HealthStatus,
        SiteResponse, SiteListResponse, SiteData, CreateSiteRequest,
        Meta, MetaServerName
    ))
)]
pub struct ApiDoc;

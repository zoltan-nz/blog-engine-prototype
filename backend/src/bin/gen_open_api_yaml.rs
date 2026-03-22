use backend_rust::ApiDoc;
use utoipa::OpenApi;

fn main() {
    let yaml = ApiDoc::openapi()
        .to_yaml()
        .expect("Failed to start serialize spec");
    std::fs::write("../../open-api-contracts/api.yaml", yaml)
        .expect("Failed to write open-api-contracts/api.yaml");
    println!("Open API spec exported to open-api-contracts/api.yaml");
}

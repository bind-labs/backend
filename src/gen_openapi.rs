use api::http::router;
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut json = false;
    for arg in &args {
        if arg == "--json" {
            json = true;
            break;
        }
    }
    if json {
        std::fs::write("./openapi.json", router().to_openapi().to_json().unwrap()).unwrap();
    } else {
        std::fs::write("./openapi.yaml", router().to_openapi().to_yaml().unwrap()).unwrap();
    }
}

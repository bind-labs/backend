use bind_backend::http::router;

fn main() {
    std::fs::write("./openapi.yaml", router().to_openapi().to_yaml().unwrap()).unwrap();
}

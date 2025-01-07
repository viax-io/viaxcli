fn main() {
    cynic_codegen::register_schema("viax")
        .from_sdl_file("../schemas/schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}

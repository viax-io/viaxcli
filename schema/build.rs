fn main() {
    cynic_codegen::register_schema("viax")
        .from_sdl_file("../schemas/pe.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}

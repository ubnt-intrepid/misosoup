#![no_main]
#[macro_use] extern crate libfuzzer_sys;
extern crate mison;

fuzz_target!(|data: &[u8]| {
    if let Ok(data) = std::str::from_utf8(data) {
        let backend = mison::index_builder::backend::FallbackBackend::default();
        let index_builder = mison::index_builder::IndexBuilder::new(backend, 10);
        let parser = mison::parser::Parser::new(index_builder);
        let _ = parser.parse(data);
    }
});

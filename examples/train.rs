#[cfg(feature = "avx-accel")]
extern crate mison;

#[cfg(feature = "avx-accel")]
mod imp {
    use std::env;
    use std::io::{BufRead, BufReader};
    use std::fs::File;

    use mison::query::QueryTree;
    use mison::query_parser::QueryParser;
    use mison::index_builder::IndexBuilder;
    use mison::index_builder::backend::AvxBackend;

    pub fn main() {
        let mut tree = QueryTree::default();
        tree.add_path("$._id.$oid").unwrap();
        tree.add_path("$.partners").unwrap();
        tree.add_path("$.twitter_username").unwrap();
        tree.add_path("$.total_money_raised").unwrap();

        let index_builder = IndexBuilder::new(AvxBackend::default(), tree.max_level());
        let parser = QueryParser::new(index_builder, tree);

        let path = env::args().nth(1).unwrap();
        let f = BufReader::new(File::open(path).unwrap());
        for input in f.lines().filter_map(Result::ok) {
            let _ = parser.parse(&input).unwrap();
        }
        println!("{:#?}", parser);
    }
}

#[cfg(not(feature = "avx-accel"))]
mod imp {
    pub fn main() {}
}

fn main() {
    imp::main()
}

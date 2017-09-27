#![allow(missing_docs)]

use errors::{Error, ErrorKind, Result, ResultExt};
use index_builder::{IndexBuilder, StructuralIndex};
use index_builder::backend::Backend;
use query::{QueryNode, QueryTree};


#[derive(Debug)]
pub struct QueryParser<'a, B: Backend> {
    index_builder: IndexBuilder<B>,
    query_tree: QueryTree<'a>,
}

impl<'a, B: Backend> QueryParser<'a, B> {
    pub fn new(index_builder: IndexBuilder<B>, query_tree: QueryTree<'a>) -> Self {
        Self {
            index_builder,
            query_tree,
        }
    }

    pub fn parse<'s>(&self, record: &'s str) -> Result<Vec<Option<&'s str>>> {
        let record = record.trim();
        if !record.starts_with("{") {
            return Err(Error::from(ErrorKind::InvalidRecord)).chain_err(|| "QueryParser supports only object parsing");
        }
        let index = self.index_builder.build(record)?;

        let mut result = vec![None; self.query_tree.num_paths()];
        self.parse_inner(
            &index,
            0,
            record.len(),
            self.query_tree.as_node(),
            &mut result[..],
        )?;
        Ok(result)
    }

    #[inline]
    fn parse_inner<'b, 's>(
        &self,
        index: &StructuralIndex<'b, 's>,
        begin: usize,
        mut end: usize,
        node: &QueryNode,
        results: &mut [Option<&'s str>],
    ) -> Result<()> {
        let cp = index
            .colon_positions(begin, end, node.level())
            .ok_or_else(|| Error::from(ErrorKind::InvalidRecord))
            .chain_err(|| "mismatched level")?;

        let mut num_found = 0;
        for i in (0..cp.len()).rev() {
            if num_found == node.num_children() {
                break;
            }

            let (field, fsi) = index.find_object_field(if i == 0 { begin } else { cp[i - 1] }, cp[i])?;
            if let Some(ch) = node.find_child(field.as_raw_str()) {
                num_found += 1;

                let (vsi, vei) = index.find_object_value(cp[i] + 1, end, i == cp.len() - 1);

                if let Some(id) = ch.path_id() {
                    results[id] = Some(index.substr(vsi, vei));
                }

                if !ch.is_leaf() {
                    self.parse_inner(index, vsi, vei, ch, results)?;
                }
            }

            end = fsi - 1;
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::index_builder::backend::FallbackBackend;

    #[test]
    fn basic_parsing() {
        let paths = &["$.f1", "$.f2.e1", "$.f3"];
        let record = r#"{
            "f1": true,
            "f2": {
                "e2": "\"foo\\",
                "e1": { "c1": null }
            },
            "f3": [ true, "10", null ]
        }"#;

        let mut query_tree = QueryTree::default();
        for path in paths {
            query_tree.add_path(path).unwrap();
        }

        let index_builder = IndexBuilder::new(FallbackBackend::default(), query_tree.max_level());
        let parser = QueryParser::new(index_builder, query_tree);

        let result = parser.parse(record).unwrap();
        assert_eq!(
            result,
            &[
                Some("true"),
                Some(r#"{ "c1": null }"#),
                Some(r#"[ true, "10", null ]"#)
            ]
        );
    }
}

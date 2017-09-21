#![allow(missing_docs)]

use index_builder::{IndexBuilder, StructuralIndex};
use index_builder::backend::{Backend, Sse2Backend};
use query::{QueryNode, QueryTree};
use errors::{ErrorKind, Result};


#[derive(Debug)]
pub struct Parser<'a, B: Backend = Sse2Backend> {
    queries: QueryTree<'a>,
    index_builder: IndexBuilder<B>,
}

impl<'a, B: Backend> Parser<'a, B> {
    pub fn new(queries: QueryTree<'a>, index_builder: IndexBuilder<B>) -> Self {
        Self {
            queries,
            index_builder,
        }
    }

    pub fn parse<'s>(&self, record: &'s str) -> Result<Vec<Option<&'s str>>> {
        let index = self.index_builder
            .build(record.as_bytes(), self.queries.max_level());
        let mut result = vec![None; self.queries.num_paths()];
        self.parse_impl(
            record,
            0,
            record.len(),
            &index,
            self.queries.as_node(),
            &mut result,
        )?;
        Ok(result)
    }

    fn parse_impl<'s>(
        &self,
        record: &'s str,
        begin: usize,
        mut end: usize,
        index: &StructuralIndex,
        node: &QueryNode,
        result: &mut [Option<&'s str>],
    ) -> Result<()> {
        // TODO: use Iterator to avoid allocation
        let cp = index.colon_positions(begin, end, node.level());

        let mut num_found = 0;
        for i in (0..cp.len()).rev() {
            let (fsi, fei) = index.find_field(if i == 0 { begin } else { cp[i - 1] }, cp[i])?;
            let field = &record[fsi..fei];

            if let Some(c) = node.find_child(field) {
                let (vsi, vei) = index.find_value(record.as_bytes(), cp[i] + 1, end, i == cp.len() - 1)?;
                let value = record[vsi..vei].trim();
                if value.is_empty() {
                    Err(ErrorKind::InvalidRecord)?;
                }

                if let Some(i) = c.path_id() {
                    // FIXME:
                    // assign only if the result is empty.
                    result[i] = Some(value);
                }

                if !c.is_leaf() {
                    self.parse_impl(record, vsi, vei, index, c, result)?;
                }

                num_found += 1;
                if num_found == node.num_children() {
                    break;
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

    #[test]
    fn basic_parsing() {
        let paths = &["$.f1", "$.f2.e1", "$.f2.e1.c2"];
        let record = r#"{
            "f1": true,
            "f2": {
                "e2": "\"foo\\",
                "e1": { "c1": null }
            },
            "f3": false
        }"#;

        let mut queries = QueryTree::default();
        for &path in paths {
            queries.add_path(path).unwrap();
        }

        let backend = Sse2Backend::default();
        let index_builder = IndexBuilder::new(backend);

        let parser = Parser::new(queries, index_builder);

        let result = parser.parse(record).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Some("true"));
        assert_eq!(result[1], Some(r#"{ "c1": null }"#));
        assert_eq!(result[2], None);
    }

    #[test]
    fn basic_parsing_failure_case() {
        let record = r#"{ "f1": }"#;

        let mut queries = QueryTree::default();
        queries.add_path("$.f1").unwrap();

        let backend = Sse2Backend::default();
        let index_builder = IndexBuilder::new(backend);

        let parser = Parser::new(queries, index_builder);

        assert!(parser.parse(record).is_err());
    }

}

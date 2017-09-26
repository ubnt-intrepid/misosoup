#![allow(missing_docs)]

use index_builder::{IndexBuilder, StructuralIndex};
use index_builder::backend::Backend;
use errors::{ErrorKind, Result};
use query::{QueryNode, QueryTree};


#[derive(Debug)]
pub struct QueryParser<'a, B: Backend> {
    queries: QueryTree<'a>,
    index_builder: IndexBuilder<B>,
}

impl<'a, B: Backend> QueryParser<'a, B> {
    pub fn new(queries: QueryTree<'a>, index_builder: IndexBuilder<B>) -> Self {
        Self {
            queries,
            index_builder,
        }
    }

    pub fn parse<'s>(&self, record: &'s str) -> Result<Vec<Option<&'s str>>> {
        let index = self.index_builder
            .build(record.as_bytes(), self.queries.max_level())?;
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
        end: usize,
        index: &StructuralIndex,
        node: &QueryNode,
        result: &mut [Option<&'s str>],
    ) -> Result<()> {
        // TODO: use Iterator to avoid allocation
        let cp = index.colon_positions(begin, end, node.level());

        let mut num_found = 0;
        for i in 0..cp.len() {
            if num_found == node.num_children() {
                break;
            }

            let field = index.find_field(record, if i == 0 { begin } else { cp[i - 1] }, cp[i])?;
            let c = match node.find_child(field) {
                Some(c) => {
                    num_found += 1;
                    c
                }
                None => continue,
            };

            let vsi = cp[i] + 1;
            let vei = if i == cp.len() - 1 { end } else { cp[i + 1] };

            if let Some(i) = c.path_id() {
                let value = index.find_value(record, vsi, vei, node.level())?;
                if value.is_empty() {
                    Err(ErrorKind::InvalidRecord)?;
                }
                // FIXME: assign only if result[i] is empty.
                result[i] = Some(value);
            }

            if !c.is_leaf() {
                self.parse_impl(record, vsi, vei, index, c, result)?;
            }
        }

        Ok(())
    }
}


#[derive(Debug)]
pub enum Value<'a> {
    Atomic(&'a str),
    Object(Vec<(&'a str, Value<'a>)>),
}

#[derive(Debug)]
pub struct Parser<B: Backend> {
    index_builder: IndexBuilder<B>,
    max_level: usize,
}

impl<B: Backend> Parser<B> {
    pub fn new(index_builder: IndexBuilder<B>, max_level: usize) -> Self {
        Self {
            index_builder,
            max_level,
        }
    }

    pub fn parse<'s>(&self, record: &'s str) -> Result<Value<'s>> {
        let index = self.index_builder.build(record.as_bytes(), self.max_level)?;
        self.basic_parse(record, 0, record.len(), &index, 0)
    }

    fn basic_parse<'s>(&self, record: &'s str, begin: usize, end: usize, index: &StructuralIndex, level: usize) -> Result<Value<'s>> {
        let mut result = Vec::new();

        let cp = index.colon_positions(begin, end, level);
        for (i, _) in cp.iter().enumerate() {
            let field = index.find_field(record, if i == 0 { begin } else { cp[i - 1] }, cp[i])?;

            let vsi = cp[i] + 1;
            let vei = if i == cp.len() - 1 { end } else { cp[i + 1] };
            let value = match index.find_value(record, vsi, vei, level)? {
                "" => return Err(ErrorKind::InvalidRecord.into()),
                s if s.starts_with("{") && s.ends_with("}") => if level + 1 < self.max_level {
                    self.basic_parse(record, vsi, vei, index, level + 1)?
                } else {
                    Value::Atomic(s)
                },
                s => Value::Atomic(s),
            };

            result.push((field, value));
        }

        Ok(Value::Object(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::index_builder::backend::FallbackBackend;

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

        let backend = FallbackBackend::default();
        let index_builder = IndexBuilder::new(backend);

        let parser = QueryParser::new(queries, index_builder);

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

        let backend = FallbackBackend::default();
        let index_builder = IndexBuilder::new(backend);

        let parser = QueryParser::new(queries, index_builder);

        assert!(parser.parse(record).is_err());
    }
}

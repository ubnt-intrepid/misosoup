#![allow(missing_docs)]

use std::cell::RefCell;
use std::collections::VecDeque;
use errors::{Error, ErrorKind, Result, ResultExt};
use index_builder::{IndexBuilder, StructuralIndex};
use index_builder::backend::Backend;
use query::{QueryNode, QueryTree};
use pattern_tree::PatternTree;


#[derive(Debug)]
pub enum QueryParserMode {
    Basic,
    Speculative,
}

#[derive(Debug)]
pub struct QueryParser<'a, B: Backend> {
    index_builder: IndexBuilder<B>,
    query_tree: QueryTree<'a>,
    colon_positions: Vec<RefCell<Vec<usize>>>,
    pattern_trees: Vec<RefCell<PatternTree>>,
    save_patterns: bool,
    allow_fallback: bool,
}

impl<'a, B: Backend> QueryParser<'a, B> {
    pub fn new(index_builder: IndexBuilder<B>, query_tree: QueryTree<'a>) -> Self {
        let num_nodes = query_tree.num_nodes();

        let mut pattern_trees = Vec::with_capacity(num_nodes);
        for _ in 0..num_nodes {
            pattern_trees.push(RefCell::new(Default::default()));
        }

        Self {
            index_builder,
            query_tree,
            colon_positions: vec![RefCell::new(vec![]); num_nodes],
            pattern_trees,
            save_patterns: false,
            allow_fallback: true,
        }
    }

    pub fn save_patterns(&mut self, v: bool) {
        self.save_patterns = v;
    }

    pub fn allow_fallback(&mut self, v: bool) {
        self.allow_fallback = v;
    }

    pub fn parse<'s>(&self, record: &'s str, mode: QueryParserMode) -> Result<Vec<Option<&'s str>>> {
        let record = record.trim();
        if !record.starts_with("{") {
            return Err(Error::from(ErrorKind::InvalidRecord)).chain_err(|| "QueryParser supports only object parsing");
        }

        let index = self.index_builder.build(record)?;

        let mut result = vec![None; self.query_tree.num_paths()];
        match mode {
            QueryParserMode::Basic => {
                self.parse_basic(
                    &index,
                    0,
                    record.len(),
                    self.query_tree.as_node(),
                    &mut result[..],
                )?;
            }
            QueryParserMode::Speculative => {
                let success = self.parse_speculative(
                    &index,
                    0,
                    record.len(),
                    self.query_tree.as_node(),
                    &mut result[..],
                )?;
                if !success {
                    if !self.allow_fallback {
                        return Err(ErrorKind::FailedSpeculativeParse.into());
                    }
                    self.parse_basic(
                        &index,
                        0,
                        record.len(),
                        self.query_tree.as_node(),
                        &mut result[..],
                    )?;
                }
            }
        }

        Ok(result)
    }

    #[inline]
    fn parse_basic<'b, 's>(
        &self,
        index: &StructuralIndex<'b, 's>,
        begin: usize,
        mut end: usize,
        node: &QueryNode,
        results: &mut [Option<&'s str>],
    ) -> Result<()> {
        // TODO: avoid to calculate colon positions if it has already generated.
        if !index.colon_positions(
            begin,
            end,
            node.level(),
            &mut *RefCell::borrow_mut(&self.colon_positions[node.node_id()]),
        ) {
            return Err(Error::from(ErrorKind::InvalidRecord)).chain_err(|| "mismatched level");
        }
        let cp = self.colon_positions[node.node_id()].borrow();

        let mut pattern = VecDeque::with_capacity(node.num_children());

        for i in (0..cp.len()).rev() {
            let (field, fsi) = index.find_object_field(if i == 0 { begin } else { cp[i - 1] }, cp[i])?;
            if let Some(ch) = node.find_child(field.as_raw_str()) {
                let (vsi, vei) = index.find_object_value(cp[i] + 1, end, i == cp.len() - 1);

                if let Some(id) = ch.path_id() {
                    results[id] = Some(index.substr(vsi, vei));
                }

                if !ch.is_leaf() {
                    self.parse_basic(index, vsi, vei, ch, results)?;
                }

                pattern.push_front((field.as_raw_str().to_owned(), i));
                if pattern.len() == node.num_children() {
                    if self.save_patterns {
                        self.pattern_trees[node.node_id()]
                            .borrow_mut()
                            .append(pattern);
                    }
                    break;
                }
            }

            end = fsi - 1;
        }

        Ok(())
    }

    #[inline]
    fn parse_speculative<'b, 's>(
        &self,
        index: &StructuralIndex<'b, 's>,
        begin: usize,
        end: usize,
        node: &QueryNode,
        results: &mut [Option<&'s str>],
    ) -> Result<bool> {
        if !index.colon_positions(
            begin,
            end,
            node.level(),
            &mut *RefCell::borrow_mut(&self.colon_positions[node.node_id()]),
        ) {
            return Err(Error::from(ErrorKind::InvalidRecord)).chain_err(|| "mismatched level");
        }
        let cp = self.colon_positions[node.node_id()].borrow();

        let pattern_tree = self.pattern_trees[node.node_id()].borrow();
        let mut pattern_node = pattern_tree.root_node();

        while !pattern_node.is_leaf() {
            let mut success = false;
            for child in pattern_node.children() {
                let i = child.position();
                let (field, _) = index.find_object_field(if i == 0 { begin } else { cp[i - 1] }, cp[i])?;
                success = field.as_raw_str() == child.field();
                if success {
                    let ch_node = node.find_child(field.as_raw_str()).unwrap();

                    let fsi = if i == cp.len() - 1 {
                        end
                    } else {
                        index.find_object_field(cp[i], cp[i + 1])?.1 - 1
                    };
                    let (vsi, vei) = index.find_object_value(cp[i] + 1, fsi, i == cp.len() - 1);

                    if let Some(id) = ch_node.path_id() {
                        results[id] = Some(index.substr(vsi, vei));
                    }

                    if !ch_node.is_leaf() {
                        success &= self.parse_speculative(index, vsi, vei, ch_node, results)?;
                    }

                    pattern_node = child;
                    break;
                }
            }

            if !success {
                break;
            }
        }

        Ok(!pattern_node.is_root() && pattern_node.is_leaf())
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

        let result = parser.parse(record, QueryParserMode::Basic).unwrap();
        assert_eq!(
            result,
            &[
                Some("true"),
                Some(r#"{ "c1": null }"#),
                Some(r#"[ true, "10", null ]"#)
            ]
        );
    }

    #[test]
    fn speculative_parsing() {
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
        let mut parser = QueryParser::new(index_builder, query_tree);
        parser.save_patterns(true);
        parser.allow_fallback(false);

        let _ = parser.parse(record, QueryParserMode::Basic).unwrap();

        let result = parser.parse(record, QueryParserMode::Speculative).unwrap();
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

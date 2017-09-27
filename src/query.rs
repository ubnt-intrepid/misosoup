//! Definition of pattern tree and query parsing

use std::cmp;
use fnv::FnvHashMap;
use errors::{ErrorKind, Result};

/// Child node in pattern tree
#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct QueryNode<'a> {
    /// identifier of this node
    node_id: usize,
    /// identifier of associated query path
    query_id: Option<usize>,
    /// level in the associated tree
    level: usize,
    /// child nodes
    children: FnvHashMap<&'a str, QueryNode<'a>>,
}

impl<'a> QueryNode<'a> {
    #[allow(missing_docs)]
    pub fn level(&self) -> usize {
        self.level
    }

    #[allow(missing_docs)]
    pub fn path_id(&self) -> Option<usize> {
        self.query_id
    }

    #[allow(missing_docs)]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    #[allow(missing_docs)]
    pub fn find_child(&self, field: &str) -> Option<&QueryNode> {
        self.children.get(field)
    }

    #[allow(missing_docs)]
    pub fn num_children(&self) -> usize {
        self.children.len()
    }
}

/// A pattern tree
#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct QueryTree<'a> {
    /// root node
    root: QueryNode<'a>,
    /// query paths
    paths: Vec<&'a str>,
    /// maximal level in this tree
    max_level: usize,
    /// number of nodes in this tree, exclude the root node
    num_nodes: usize,
}

impl<'a> Default for QueryTree<'a> {
    fn default() -> Self {
        Self {
            root: QueryNode {
                node_id: !0,
                ..Default::default()
            },
            paths: vec![],
            max_level: 0,
            num_nodes: 0,
        }
    }
}

impl<'a> QueryTree<'a> {
    /// Parse query path and append it to the pattern tree.
    pub fn add_path(&mut self, path: &'a str) -> Result<()> {
        if !path.starts_with("$.") {
            Err(ErrorKind::InvalidQuery)?;
        }

        let mut cur = &mut self.root;
        for field in path[2..].split('.') {
            if field.is_empty() {
                Err(ErrorKind::InvalidQuery)?;
            }

            let level = cur.level + 1;
            let num_nodes = &mut self.num_nodes;

            let cur1 = cur;
            cur = cur1.children.entry(field).or_insert_with(|| {
                let node = QueryNode {
                    node_id: *num_nodes,
                    level,
                    ..Default::default()
                };
                *num_nodes += 1;
                node
            });
        }

        cur.query_id = Some(self.paths.len());

        self.max_level = cmp::max(self.max_level, cur.level);
        self.paths.push(path);

        Ok(())
    }

    #[allow(missing_docs)]
    pub fn max_level(&self) -> usize {
        self.max_level
    }

    #[allow(missing_docs)]
    pub fn num_paths(&self) -> usize {
        self.paths.len()
    }

    #[allow(missing_docs)]
    pub fn as_node(&self) -> &QueryNode {
        &self.root
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! hashmap {
        (@single $($x:tt)*) => (());
        (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

        ($($k:expr => $v:expr,)+) => { hashmap!($($k => $v),+) };
        ($($k:expr => $v:expr),*) => {{
            let mut _map = ::std::collections::HashMap::with_capacity_and_hasher(hashmap!(@count $($k),*), Default::default());
            $( _map.insert($k, $v); )*
            _map
        }}
    }

    #[test]
    fn invalid_query() {
        let cases: &[&str] = &["", "$", "$.."];
        for c in cases {
            let mut tree = QueryTree::default();
            assert!(tree.add_path(c).is_err());
        }
    }

    #[test]
    fn add_path() {
        struct TestCase {
            input: &'static [&'static str],
            expect: QueryTree<'static>,
        }
        let cases: &[TestCase] = &[
            TestCase {
                input: &["$.foo"],
                expect: QueryTree {
                    max_level: 1,
                    num_nodes: 1,
                    paths: vec!["$.foo"],
                    root: QueryNode {
                        node_id: !0,
                        query_id: None,
                        level: 0,
                        children: hashmap!{
                            "foo" => QueryNode {
                                node_id: 0,
                                query_id: Some(0),
                                level: 1,
                                children: Default::default(),
                            },
                        },
                    },
                },
            },
            TestCase {
                input: &["$.foo.bar"],
                expect: QueryTree {
                    max_level: 2,
                    num_nodes: 2,
                    paths: vec!["$.foo.bar"],
                    root: QueryNode {
                        node_id: !0,
                        query_id: None,
                        level: 0,
                        children: hashmap!{
                            "foo" => QueryNode {
                                node_id: 0,
                                query_id: None,
                                level: 1,
                                children: hashmap!{
                                    "bar" => QueryNode {
                                        node_id: 1,
                                        query_id: Some(0),
                                        level: 2,
                                        children: Default::default(),
                                    }
                                },
                            },
                        },
                    },
                },
            },
            TestCase {
                input: &["$.f1.e1", "$.f1.e1.c3", "$.f2.e1"],
                expect: QueryTree {
                    max_level: 3,
                    num_nodes: 5,
                    paths: vec!["$.f1.e1", "$.f1.e1.c3", "$.f2.e1"],
                    root: QueryNode {
                        node_id: !0,
                        query_id: None,
                        level: 0,
                        children: hashmap!{
                            "f1" => QueryNode {
                                node_id: 0,
                                query_id: None,
                                level: 1,
                                children: hashmap!{
                                    "e1" => QueryNode {
                                        node_id: 1,
                                        query_id: Some(0),
                                        level: 2,
                                        children: hashmap!{
                                            "c3" => QueryNode {
                                                node_id: 2,
                                                query_id: Some(1),
                                                level: 3,
                                                children: Default::default(),
                                            },
                                        },
                                    }
                                },
                            },
                            "f2" => QueryNode {
                                node_id: 3,
                                query_id: None,
                                level: 1,
                                children: hashmap!{
                                    "e1" => QueryNode {
                                        node_id: 4,
                                        query_id: Some(2),
                                        level: 2,
                                        children: Default::default(),
                                    }
                                },
                            },
                        },
                    },
                },
            },
        ];
        for t in cases {
            let mut tree = QueryTree::default();
            for i in t.input {
                assert!(tree.add_path(i).is_ok());
            }
            assert_eq!(tree, t.expect);
        }
    }
}

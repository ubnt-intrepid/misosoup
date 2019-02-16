#![allow(missing_docs)]

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PatternNode {
    field: String,
    pos: usize,
    weight: usize,
    children: Vec<PatternNode>,
}

impl Default for PatternNode {
    fn default() -> Self {
        PatternNode {
            field: "$".to_owned(),
            pos: !0,
            weight: 0,
            children: vec![],
        }
    }
}

impl PatternNode {
    #[inline]
    pub fn field(&self) -> &str {
        &self.field
    }

    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    #[inline]
    pub fn children(&self) -> &[PatternNode] {
        self.children.as_slice()
    }

    #[inline]
    pub fn is_root(&self) -> bool {
        self.field == "$"
    }

    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Default)]
pub struct PatternTree {
    root: PatternNode,
}

impl PatternTree {
    /// Add a pattern into this pattern tree.
    ///
    /// The pattern should be represented as a sequence of pairs of field and its appearance
    /// position.
    ///
    /// ```{text,ignore}
    /// { "A": "", "B": "", "Z": "", "_dummy": { ... }, "Y": "" }
    /// ```
    ///
    /// ```{text, ignore}
    /// ["$.A", "$.B", "$.Y", "$.Z"]
    /// ```
    ///
    /// ```{text,ignore}
    /// [("A", 0), ("B", 1), ("Z", 2), ("Y", 4)]
    /// ```
    pub fn append<'a, I>(&mut self, pattern: I)
    where
        I: IntoIterator<Item = (String, usize)>,
    {
        let mut cur = &mut self.root;
        cur.weight += 1;
        for (field, pos) in pattern {
            let cur1 = cur;
            cur = match cur1
                .children
                .iter()
                .position(|ch| ch.field == field && ch.pos == pos)
            {
                Some(i) => &mut cur1.children[i],
                None => {
                    cur1.children.push(PatternNode {
                        field,
                        pos,
                        ..Default::default()
                    });
                    cur1.children.last_mut().unwrap()
                }
            };
            cur.weight += 1;
        }
    }

    #[allow(missing_docs)]
    #[inline]
    pub fn root_node(&self) -> &PatternNode {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_tree() {
        let mut tree = PatternTree::default();
        tree.append(vec![
            ("foo".to_owned(), 0),
            ("bar".to_owned(), 1),
            ("baz".to_owned(), 2),
        ]);
        tree.append(vec![
            ("foo".to_owned(), 0),
            ("baz".to_owned(), 1),
            ("bar".to_owned(), 3),
        ]);
        tree.append(vec![
            ("foo".to_owned(), 0),
            ("bar".to_owned(), 2),
            ("baz".to_owned(), 3),
        ]);

        let expected = PatternNode {
            field: "$".to_owned(),
            pos: !0,
            weight: 3,
            children: vec![PatternNode {
                field: "foo".to_owned(),
                pos: 0,
                weight: 3,
                children: vec![
                    PatternNode {
                        field: "bar".to_owned(),
                        pos: 1,
                        weight: 1,
                        children: vec![PatternNode {
                            field: "baz".to_owned(),
                            pos: 2,
                            weight: 1,
                            children: vec![],
                        }],
                    },
                    PatternNode {
                        field: "baz".to_owned(),
                        pos: 1,
                        weight: 1,
                        children: vec![PatternNode {
                            field: "bar".to_owned(),
                            pos: 3,
                            weight: 1,
                            children: vec![],
                        }],
                    },
                    PatternNode {
                        field: "bar".to_owned(),
                        pos: 2,
                        weight: 1,
                        children: vec![PatternNode {
                            field: "baz".to_owned(),
                            pos: 3,
                            weight: 1,
                            children: vec![],
                        }],
                    },
                ],
            }],
        };
        assert_eq!(tree.root, expected);
    }
}

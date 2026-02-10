use std::fmt::Display;

pub struct BTree {
    order: usize,
    root: Option<Node>,
}

impl BTree {
    pub fn new(order: usize) -> Self {
        let root = Node::Leaf {
            keys: vec![],
            values: vec![],
        };
        Self {
            order,
            root: Some(root),
        }
    }
    pub fn order(&self) -> usize {
        self.order
    }
    pub fn root(&self) -> Option<&Node> {
        self.root.as_ref()
    }
    pub fn search(&self, key: i32) -> Option<String> {
        None
    }
}

impl Display for BTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BTree(order={}) <empty>", self.order)
    }
}

pub enum Node {
    // A leaf node carrying keys and associated values.
    Leaf { keys: Vec<i32>, values: Vec<String> },
    Internal { keys: Vec<i32>, values: Vec<String> },
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Leaf { keys, .. } => write!(
                f,
                "[Leaf [{}]]",
                keys.into_iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Node::Internal { keys, .. } => write!(
                f,
                "[Internal [{}]]",
                keys.into_iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Node;

    #[test]
    fn render_leaf_node() {
        let node = crate::Node::Leaf {
            keys: vec![1, 2, 3],
            values: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        assert_eq!(format!("{node}"), "[Leaf [1 2 3]]")
    }
    #[test]
    fn render_internal_node() {
        let node = crate::Node::Internal {
            keys: vec![10, 20],
            values: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        };
        assert_eq!(format!("{node}"), "[Internal [10 20]]")
    }
    #[test]
    fn render_empty_btree_with_order() {
        for order in [3, 4, 5] {
            let btree = crate::BTree::new(order);
            assert_eq!(
                format!("{btree}"),
                format!("BTree(order={}) <empty>", order)
            );
        }
    }
    #[test]
    fn select_order_from_empty_btree() {
        for order in [3, 4, 5] {
            let btree = crate::BTree::new(order);
            assert_eq!(btree.order(), order);
        }
    }
    #[test]
    fn select_root_from_empty_btree() {
        let btree = crate::BTree::new(3);
        assert!(matches!(btree.root(), Some(Node::Leaf { .. })))
    }
    #[test]
    fn search_value_from_empty_btree() {
        let btree = crate::BTree::new(3);
        let item = btree.search(10);
        assert!(matches!(item, None))
    }
}

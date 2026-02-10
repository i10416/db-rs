use std::{cell::RefCell, fmt::Display, rc::Rc};

type Link<T> = Option<Rc<RefCell<T>>>;

pub struct BTree {
    order: usize,
    root: Link<Node>,
}

impl BTree {
    pub fn new(order: usize) -> Self {
        let root = Node {
            keys: vec![],
            values: vec![],
            parent: None,
            children: vec![],
            is_leaf: true,
        };
        Self {
            order,
            root: Some(Rc::new(RefCell::new(root))),
        }
    }
    pub fn new_with_internal(order: usize) -> Self {
        let root = Node {
            keys: vec![],
            values: vec![],
            parent: None,
            children: vec![],
            is_leaf: false,
        };
        Self {
            order,
            root: Some(Rc::new(RefCell::new(root))),
        }
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn search(&self, key: i32) -> Option<String> {
        match self.root.as_ref() {
            None => None,
            Some(root) => root.borrow().search(key),
        }
    }
}

impl Display for BTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BTree(order={}) <empty>", self.order)
    }
}

pub struct Node {
    /// sorted keys
    keys: Vec<i32>,
    values: Vec<String>,
    parent: Link<Node>,
    children: Vec<Box<Node>>,
    is_leaf: bool,
}

impl Node {
    pub fn leaf<S: AsRef<str>>(keys: &[i32], values: &[S]) -> Self {
        Node {
            keys: keys.to_vec(),
            values: values
                .iter()
                .map(|item| item.as_ref().to_string())
                .collect(),
            parent: None,
            children: vec![],
            is_leaf: true,
        }
    }
    pub fn leaf_with_parent<S: AsRef<str>>(
        keys: &[i32],
        values: &[S],
        parent: Rc<RefCell<Node>>,
    ) -> Self {
        Node {
            keys: keys.to_vec(),
            values: values
                .iter()
                .map(|item| item.as_ref().to_string())
                .collect(),
            parent: Some(parent),
            children: vec![],
            is_leaf: true,
        }
    }
    pub fn keys(&self) -> &[i32] {
        match self {
            Node { keys, .. } => keys,
        }
    }

    pub fn insert_keys(&mut self, new_keys: &[i32]) {
        match self {
            Node { keys, .. } => keys.extend(new_keys),
        }
    }
    pub fn insert_values(&mut self, new_values: &[String]) {
        match self {
            Node { values, .. } => {
                values.extend_from_slice(new_values);
            }
        }
    }
    pub fn insert_children(&mut self, new_children: Vec<Box<Node>>) {
        match self {
            Node { children, .. } => {
                children.extend(new_children);
            }
        }
    }
    pub fn search(&self, key: i32) -> Option<String> {
        let node = self;
        let keys = node.keys();
        let idx = keys.binary_search(&key);
        match idx {
            Ok(idx) => match &*node {
                Node { values, .. } => Some(values[idx].clone()),
            },
            Err(idx) => match &*node {
                Node {
                    children,
                    is_leaf: false,
                    ..
                } => children[idx].search(key),
                _ => None,
            },
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node {
                keys,
                is_leaf: true,
                ..
            } => write!(
                f,
                "[Leaf [{}]]",
                keys.into_iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Node {
                keys,
                is_leaf: false,
                ..
            } => write!(
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
    use std::ops::Deref;

    use crate::Node;

    #[test]
    fn render_leaf_node() {
        let node = crate::Node::leaf(&[1, 2, 3], &["a", "b", "c"]);
        assert_eq!(format!("{node}"), "[Leaf [1 2 3]]")
    }
    #[test]
    fn render_internal_node() {
        let node = crate::Node {
            keys: vec![10, 20],
            children: vec![],
            is_leaf: false,
            values: vec!["a".to_string(), "b".to_ascii_lowercase()],
            parent: None,
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
        assert!(matches!(
            btree.root.unwrap().borrow().deref(),
            Node { is_leaf: true, .. }
        ))
    }
    #[test]
    fn search_value_from_empty_btree() {
        let btree = crate::BTree::new(3);
        let item = btree.search(10);
        assert!(matches!(item, None))
    }
    #[test]
    fn search_value_from_single_level_btree() {
        let btree = crate::BTree::new(3);
        btree
            .root
            .as_ref()
            .unwrap()
            .borrow_mut()
            .insert_keys(&[10, 20, 30]);
        btree.root.as_ref().unwrap().borrow_mut().insert_values(&[
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
        ]);
        let item = btree.search(10);
        assert!(matches!(item.as_deref(), Some("a")));
        let item = btree.search(20);
        assert!(matches!(item.as_deref(), Some("b")));
        let item = btree.search(30);
        assert!(matches!(item.as_deref(), Some("c")));
        let item = btree.search(5);
        assert!(matches!(item.as_deref(), None));
        let item = btree.search(25);
        assert!(matches!(item.as_deref(), None));
        let item = btree.search(35);
        assert!(matches!(item.as_deref(), None));
    }
    #[test]
    fn search_value_from_multiple_level_btree() {
        let mut btree = crate::BTree::new_with_internal(3);
        btree
            .root
            .as_mut()
            .unwrap()
            .borrow_mut()
            .insert_keys(&[10, 20]);
        let child1 = Box::new(crate::Node::leaf_with_parent(
            &[1, 5, 9],
            &["a", "b", "c"],
            btree.root.clone().unwrap(),
        ));
        let child2 = Box::new(crate::Node::leaf_with_parent(
            &[15, 18, 19],
            &["d", "e", "f"],
            btree.root.clone().unwrap(),
        ));
        let child3 = Box::new(crate::Node::leaf_with_parent(
            &[25, 28, 30],
            &["g", "h", "i"],
            btree.root.clone().unwrap(),
        ));
        btree
            .root
            .as_mut()
            .unwrap()
            .borrow_mut()
            .insert_children(vec![child1, child2, child3]);

        let item = btree.search(1);
        assert!(matches!(item.as_deref(), Some("a")));
        let item = btree.search(18);
        assert!(matches!(item.as_deref(), Some("e")));
        let item = btree.search(28);
        assert!(matches!(item.as_deref(), Some("h")));
        let item = btree.search(22);
        assert!(matches!(item.as_deref(), None));
        let item = btree.search(100);
        assert!(matches!(item.as_deref(), None));
    }
}

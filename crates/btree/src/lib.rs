use std::{
    cell::RefCell,
    fmt::Display,
    rc::{Rc, Weak},
};

type Link<T> = Rc<RefCell<T>>;
type WeakLink<T> = Weak<RefCell<T>>;

/// A data structure to hold a reference to root node
///
/// The "order" determines the maximum number of children
/// and keys a node can have.
///  The properties of a BTree of order m are as follows:
/// - Each node can have at most m children and m - 1 keys.
/// - Each internal node (except the root) must have at least \[m/2\]
///   children and \[m/2\] - 1 keys.
/// - The root node must have at least 2 children if it is not a leaf,
///   and at least 1 key if it is not empty.
///
/// For example, for a BTree of order 3:
/// - Maximum children per node: m = 3
/// - Maximum keys per node: m - 1 = 2
/// - Minimum children (for internal nodes): \[m/2\] = 2
/// - Minimum keys: 1
///
/// where m = order of the BTree
#[cfg_attr(test, derive(Debug))]
pub struct BTree {
    order: usize,
    root: Link<Node>,
}

pub enum BtreeOperationError {
    KeyAlreadyExists(usize),
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
            root: Rc::new(RefCell::new(root)),
        }
    }
    pub fn empty_with_internal(order: usize) -> Self {
        let root = Node {
            keys: vec![],
            values: vec![],
            parent: None,
            children: vec![],
            is_leaf: false,
        };
        Self {
            order,
            root: Rc::new(RefCell::new(root)),
        }
    }

    pub fn order(&self) -> usize {
        self.order
    }

    pub fn search(&self, key: i32) -> Option<String> {
        self.root.as_ref().borrow().search(key)
    }
    pub fn insert_keys(&mut self, keys: &[i32]) {
        self.root.borrow_mut().insert_keys(keys);
    }
    pub fn insert_root_children(&mut self, children: Vec<Link<Node>>) {
        for child in children.iter() {
            child.borrow_mut().parent = Some(Rc::downgrade(&self.root));
        }
        self.root.borrow_mut().children.extend(children);
    }

    pub fn insert(&mut self, key: i32, value: &str) -> Result<(), BtreeOperationError> {
        let mutated = {
            let mut root = self.root.borrow_mut();
            root.insert(key, value)?
        };
        if let Some(mutated) = mutated {
            if mutated.borrow().is_overflown(self.order) {
                self.handle_overflow(mutated)
            } else {
                Ok(())
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if self.root.borrow().is_overflown(self.order) {
                self.handle_overflow(self.root.clone())
            } else {
                Ok(())
            }
        }
    }
    fn handle_overflow(&mut self, node: Link<Node>) -> Result<(), BtreeOperationError> {
        // Given node [a, b, c], the median is b, and the new node will be [c], and the original node will be [a]
        let (new_node, (key, value)) = node.borrow_mut().split();
        // parent of the node being split will be the parent of the new node
        let parent = node.borrow().parent.clone();
        match (parent, new_node) {
            (Some(parent), new_node) => {
                new_node.borrow_mut().parent = Some(parent.clone());
                let parent = parent.upgrade().unwrap();
                let idx = match parent.borrow().keys().binary_search(&key) {
                    Ok(idx) => Err(BtreeOperationError::KeyAlreadyExists(idx)),
                    Err(idx) => Ok(idx),
                }?;
                // bubble up the median key and value to the parent node,
                // and insert the new node as the child of the parent node
                parent.borrow_mut().children.insert(idx + 1, new_node);
                parent.borrow_mut().keys.insert(idx, key);
                parent.borrow_mut().values.insert(idx, value);
                if parent.borrow().is_overflown(self.order) {
                    self.handle_overflow(parent)
                } else {
                    Ok(())
                }
            }
            (None, new_node) => {
                let new_root = Node {
                    keys: vec![key],
                    values: vec![value],
                    parent: None,
                    children: vec![node.clone(), new_node.clone()],
                    is_leaf: false,
                };
                let new_root = Rc::new(RefCell::new(new_root));
                new_node.borrow_mut().parent = Some(Rc::downgrade(&new_root));
                node.borrow_mut().parent = Some(Rc::downgrade(&new_root));
                self.root = new_root;
                Ok(())
            }
        }
    }
}

impl Display for BTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BTree(order={}) <empty>", self.order)
    }
}

#[cfg_attr(test, derive(Debug))]
pub struct Node {
    /// sorted keys
    keys: Vec<i32>,
    values: Vec<String>,
    parent: Option<WeakLink<Node>>,
    children: Vec<Link<Node>>,
    is_leaf: bool,
}

impl Node {
    /// create a new orphan leaf
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

    pub fn keys(&self) -> &[i32] {
        let Node { keys, .. } = self;
        keys
    }
    pub fn is_empty(&self) -> bool {
        self.keys().is_empty()
    }

    pub fn is_overflown(&self, order: usize) -> bool {
        self.keys().len() >= order
    }
    pub fn append(&mut self, key: i32, value: &str) {
        self.insert_keys(&[key]);
        self.insert_values(&[value.to_string()]);
    }

    /// None indicates the key is inserted into the current node,
    /// and Some indicates the key is inserted into the child node
    pub fn insert(
        &mut self,
        key: i32,
        value: &str,
    ) -> Result<Option<Link<Node>>, BtreeOperationError> {
        if self.is_empty() {
            self.append(key, value);
            Ok(None)
        } else {
            let try_idx = self.keys().binary_search(&key);
            match try_idx {
                Ok(idx) => Err(BtreeOperationError::KeyAlreadyExists(idx)),
                Err(idx) => self.insert_in_node(key, value, idx),
            }
        }
    }
    /// Returns the node that is mutated by the insertion.
    /// None indicates the key is inserted into the current node,
    /// and Some indicates the key is inserted into the child node
    fn insert_in_node(
        &mut self,
        key: i32,
        value: &str,
        idx: usize,
    ) -> Result<Option<Link<Node>>, BtreeOperationError> {
        if self.is_leaf {
            if idx == self.keys().len() {
                self.append(key, value);
            } else {
                self.keys.insert(idx, key);
                self.values.insert(idx, value.to_string());
            };
            Ok(None)
        } else {
            let target = &self.children[idx];
            let mutated = target.borrow_mut().insert(key, value)?;
            Ok(mutated.or(Some(target.clone())))
        }
    }

    pub fn insert_keys(&mut self, new_keys: &[i32]) {
        let Node { keys, .. } = self;
        keys.extend(new_keys)
    }
    pub fn insert_values(&mut self, new_values: &[String]) {
        let Node { values, .. } = self;
        values.extend_from_slice(new_values)
    }
    pub fn insert_children(&mut self, new_children: Vec<Link<Node>>) {
        let Node { children, .. } = self;
        children.extend(new_children);
    }
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    /// Returns the new node, the median key and the median value for the parent node
    fn split(&mut self) -> (Link<Node>, (i32, String)) {
        let median_idx = self.len() / 2;
        let median_key = self.keys[median_idx];
        let median_value = &self.values[median_idx].to_string();
        println!(
            "split: median_idx={median_idx}, median_key={median_key}, median_value={median_value}"
        );
        let mut new_empty = Self {
            keys: vec![],
            values: vec![],
            parent: None,
            children: vec![],
            is_leaf: self.is_leaf,
        };
        let sibling = if median_idx < self.len() - 1 {
            new_empty
                .keys
                .extend(self.keys.drain(median_idx + 1..).collect::<Vec<_>>());
            self.keys.truncate(median_idx);
            new_empty
                .values
                .extend(self.values.drain(median_idx + 1..).collect::<Vec<_>>());
            self.values.truncate(median_idx);
            if !self.is_leaf {
                let sibling_children = self.children.split_off(median_idx + 1);
                new_empty.children.extend(sibling_children);
                Rc::<RefCell<Node>>::new_cyclic(|node| {
                    for child in new_empty.children.iter_mut() {
                        child.borrow_mut().parent = Some(node.clone())
                    }
                    RefCell::new(new_empty)
                })
            } else {
                Rc::new(RefCell::new(new_empty))
            }
        } else {
            Rc::new(RefCell::new(new_empty))
        };
        (sibling, (median_key, median_value.to_string()))
    }
    pub fn search(&self, key: i32) -> Option<String> {
        let node = self;
        let keys = node.keys();
        let idx = keys.binary_search(&key);
        match idx {
            Ok(idx) => {
                let Node { values, .. } = node;
                Some(values[idx].clone())
            }
            Err(idx) => match node {
                Node {
                    children,
                    is_leaf: false,
                    ..
                } => children[idx].borrow().search(key),
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
                keys.iter()
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
                keys.iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, ops::Deref, rc::Rc};

    use crate::{BTree, BtreeOperationError, Node};

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
            btree.root.borrow().deref(),
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
        btree.root.borrow_mut().insert_keys(&[10, 20, 30]);
        btree
            .root
            .borrow_mut()
            .insert_values(&["a".to_string(), "b".to_string(), "c".to_string()]);
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
        let mut btree = crate::BTree::empty_with_internal(3);
        btree.insert_keys(&[10, 20]);
        let child1 = crate::Node::leaf(&[1, 5, 9], &["a", "b", "c"]);
        let child2 = crate::Node::leaf(&[15, 18, 19], &["d", "e", "f"]);
        let child3 = crate::Node::leaf(&[25, 28, 30], &["g", "h", "i"]);
        let children = vec![
            Rc::new(RefCell::new(child1)),
            Rc::new(RefCell::new(child2)),
            Rc::new(RefCell::new(child3)),
        ];
        btree.insert_root_children(children);

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
    #[test]
    fn insert_into_empty_root() {
        let mut tree = BTree::new(3);
        let _ = tree.insert(10, "a");
        let maybe_value = tree.search(10);
        assert!(matches!(maybe_value.as_deref(), Some("a")))
    }
    #[test]
    fn insert_into_empty_root_with_two_distinct_items() {
        let mut tree = BTree::new(3);
        let _ = tree.insert(10, "a");
        let maybe_value = tree.search(10);
        assert!(matches!(maybe_value.as_deref(), Some("a")));
        let _ = tree.insert(20, "b");
        let maybe_value = tree.search(20);
        assert!(matches!(maybe_value.as_deref(), Some("b")));
    }
    #[test]
    fn insert_into_empty_root_as_many_children_as_order() {
        let mut tree = BTree::new(3);
        let _ = tree.insert(10, "a");
        let maybe_value = tree.search(10);
        assert!(matches!(maybe_value.as_deref(), Some("a")));
        let _ = tree.insert(20, "b");
        let maybe_value = tree.search(20);
        assert!(matches!(maybe_value.as_deref(), Some("b")));
        let _ = tree.insert(30, "c");
        let maybe_value = tree.search(30);
        assert!(matches!(maybe_value.as_deref(), Some("c")));
    }
    /// Inserting 4 items into an empty BTree with order 3 will cause the root node to split once,
    /// and the final structure of the tree will be as follows:
    ///
    /// ```txt
    ///           +------------+
    ///           |  (20, "b") |   <-- Root Node (Full)
    ///           +-----+------+
    ///               /   \
    ///              /     \
    ///   +----------+   +---------------------+
    ///   | (10,"a") |   | (30,"c") | (40,"d") |  <-- Leaf Nodes
    ///   +----------+   +---------------------+
    /// ```
    ///
    #[test]
    fn insert_into_empty_root_children_more_than_order() {
        let mut tree = BTree::new(3);
        let _ = tree.insert(10, "a");
        let maybe_value = tree.search(10);
        assert!(matches!(maybe_value.as_deref(), Some("a")));
        let _ = tree.insert(20, "b");
        let maybe_value = tree.search(20);
        assert!(matches!(maybe_value.as_deref(), Some("b")));
        let _ = tree.insert(30, "c");
        let maybe_value = tree.search(30);
        assert!(matches!(maybe_value.as_deref(), Some("c")));
        let _ = tree.insert(40, "d");
        let maybe_value = tree.search(40);
        assert!(matches!(maybe_value.as_deref(), Some("d")));
        assert_eq!(tree.root.borrow().keys(), [20]);
        assert_eq!(tree.root.borrow().values, ["b".to_string()]);
        assert_eq!(tree.root.borrow().children[0].borrow().keys(), [10]);
        assert_eq!(
            tree.root.borrow().children[0].borrow().values,
            ["a".to_string()]
        );
        assert_eq!(tree.root.borrow().children[1].borrow().keys(), [30, 40]);
        assert_eq!(
            tree.root.borrow().children[1].borrow().values,
            ["c".to_string(), "d".to_string()]
        );
    }
    ///
    ///
    /// Inserting 5 items into an empty BTree with order 3 will cause the root node to split twice,
    /// and the final structure of the tree will be as follows:
    ///
    ///
    /// ```txt
    ///         +------------------------+
    ///         |  (20, "b") | (40, "d") |  <-- Root Node (Full)
    ///         +----+-------+-------+---+
    ///             /        |        \
    ///            /         |         \
    /// +----------+    +----------+    +----------+
    /// | (10,"a") |    | (30,"c") |    | (50,"e") |  <-- Leaf Nodes
    /// +----------+    +----------+    +----------+
    /// ```
    #[test]
    fn insert_into_empty_root_children_more_and_more_than_order() {
        let mut tree = BTree::new(3);
        let _ = tree.insert(10, "a");
        let maybe_value = tree.search(10);
        assert!(matches!(maybe_value.as_deref(), Some("a")));
        let _ = tree.insert(20, "b");
        let maybe_value = tree.search(20);
        assert!(matches!(maybe_value.as_deref(), Some("b")));
        let _ = tree.insert(30, "c");
        let maybe_value = tree.search(30);
        assert!(matches!(maybe_value.as_deref(), Some("c")));
        let _ = tree.insert(40, "d");
        let maybe_value = tree.search(40);
        assert!(matches!(maybe_value.as_deref(), Some("d")));
        let _ = tree.insert(50, "e");
        let maybe_value = tree.search(50);
        assert!(matches!(maybe_value.as_deref(), Some("e")));
        println!("tree after inserting 5 items: {:?}", tree.root);
    }

    #[test]
    fn fails_on_inserting_value_with_conflicting_key() {
        let mut tree = BTree::new(3);
        let _ = tree.insert(20, "b");
        let result = tree.insert(20, "b'");
        assert!(matches!(
            result,
            Err(BtreeOperationError::KeyAlreadyExists(_))
        ))
    }
}

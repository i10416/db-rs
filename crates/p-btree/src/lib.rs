use prelude::result::swap;
/// A data structure to hold a reference to root node
///
/// The "order" determines the maximum number of children
/// and keys a node can have.
///  The properties of a BTree of order `m` are as follows:
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
pub struct BTree<K: Ord, V> {
    order: usize,
    root: Option<Box<Node<K, V>>>,
}

impl<K: Ord, V> BTree<K, V> {
    pub fn empty(order: usize) -> Self {
        Self { order, root: None }
    }
}

pub trait BTreeOps<K: Ord, V> {
    fn insert(&mut self, key: K, value: V) -> Result<(), BTreeOperationError>;
    fn get(&self, key: &K) -> Option<&V>;
    fn is_empty(&self) -> bool;
    // fn delete(&mut self, key: &K) -> Option<V>;
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct Node<K: Ord, V> {
    /// sorted keys
    keys: Vec<K>,
    values: Vec<V>,
    children: Vec<Box<Node<K, V>>>,
    is_leaf: bool,
}

impl<K: Ord, V> Node<K, V> {
    fn new_leaf(key: K, value: V) -> Self {
        Self {
            keys: vec![key],
            values: vec![value],
            children: vec![],
            is_leaf: true,
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        let idx = self.keys.binary_search(key);
        match idx {
            Ok(i) => {
                let Node { values, .. } = self;
                values.get(i)
            }
            Err(idx) => match self {
                Node {
                    children,
                    is_leaf: false,
                    ..
                } => children.get(idx).and_then(|child| child.get(key)),
                _ => None,
            },
        }
    }
    pub fn insert(&mut self, key: K, value: V, order: usize) -> Result<(), BTreeOperationError> {
        let idx =
            swap(self.keys.binary_search(&key)).map_err(BTreeOperationError::KeyAlreadyExists)?;
        // terminate
        if self.is_leaf {
            // any "leaf node" is guaranteed to have no more than order - 1 keys, so we can insert without worrying about overflow
            self.keys.insert(idx, key);
            self.values.insert(idx, value);
            Ok(())
        } else {
            // this is an internal node, so we need to find the child node to insert into
            let safe_to_insert = self.children[idx].is_safe_to_insert(order);
            // proactive splitting: if the child node is full, split it before inserting
            if safe_to_insert {
                self.children[idx].insert(key, value, order)
            } else {
                let ((m_key, m_value), new_node) = self.children[idx].split_off_at_median();
                self.keys.insert(idx, m_key);
                self.values.insert(idx, m_value);
                self.children.insert(idx + 1, Box::new(new_node));
                if key < self.keys[idx] {
                    self.children[idx].insert(key, value, order)
                } else {
                    self.children[idx + 1].insert(key, value, order)
                }
            }
        }
    }

    fn split_off_at_median(&mut self) -> ((K, V), Node<K, V>) {
        let median_idx = self.keys.len() / 2;
        let median_key = self.keys.remove(median_idx);
        let median_value = self.values.remove(median_idx);
        let rest = if self.is_leaf {
            vec![]
        } else {
            self.children.split_off(median_idx + 1)
        };
        let new_node = Node {
            keys: self.keys.split_off(median_idx),
            values: self.values.split_off(median_idx),
            children: rest,
            is_leaf: self.is_leaf,
        };
        ((median_key, median_value), new_node)
    }

    fn is_safe_to_insert(&self, order: usize) -> bool {
        self.keys.len() < order - 1
    }
}

impl<K: Ord, V> BTreeOps<K, V> for BTree<K, V> {
    fn is_empty(&self) -> bool {
        self.root.is_none()
    }
    fn get(&self, key: &K) -> Option<&V> {
        self.root.as_ref().and_then(|node| node.get(key))
    }
    fn insert(&mut self, key: K, value: V) -> Result<(), BTreeOperationError> {
        if let Some(mut root) = self.root.take() {
            if !root.is_safe_to_insert(self.order) {
                let ((m_key, m_value), new_node) = root.split_off_at_median();
                let mut new_root = Node {
                    keys: vec![m_key],
                    values: vec![m_value],
                    children: vec![root, Box::new(new_node)],
                    is_leaf: false,
                };
                let res = new_root.insert(key, value, self.order);
                self.root = Some(Box::new(new_root));
                res
            } else {
                let res = root.insert(key, value, self.order);
                self.root = Some(root);
                res
            }
        } else {
            self.root = Some(Box::new(Node::new_leaf(key, value)));
            Ok(())
        }
    }
}

pub enum BTreeOperationError {
    KeyAlreadyExists(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_btree() {
        let btree = BTree::<i32, String>::empty(3);
        assert!(btree.is_empty());
    }
    #[test]
    fn insert_single_key() {
        let mut btree = BTree::empty(3);
        let _ = btree.insert(1, "one");
        assert_eq!(btree.get(&1), Some(&"one"));
        assert_eq!(btree.get(&2), None);
    }
    #[test]
    fn insert_less_than_order() {
        let mut btree = BTree::empty(3);
        let _ = btree.insert(1, "one");
        let _ = btree.insert(2, "two");
        assert_eq!(btree.get(&1), Some(&"one"));
        assert_eq!(btree.get(&2), Some(&"two"));
    }
    #[test]
    fn insert_as_many_as_order() {
        let mut btree = BTree::empty(3);
        let _ = btree.insert(1, "one");
        let _ = btree.insert(2, "two");
        let _ = btree.insert(3, "three");
        assert_eq!(btree.get(&1), Some(&"one"));
        assert_eq!(btree.get(&2), Some(&"two"));
        assert_eq!(btree.get(&3), Some(&"three"));
        assert_eq!(btree.root.as_ref().unwrap().keys, vec![2]);
        assert_eq!(btree.root.as_ref().unwrap().values, vec!["two"]);
        assert_eq!(
            btree.root.as_ref().unwrap().children,
            vec![
                Box::new(Node {
                    keys: vec![1],
                    values: vec!["one"],
                    children: vec![],
                    is_leaf: true,
                }),
                Box::new(Node {
                    keys: vec![3],
                    values: vec!["three"],
                    children: vec![],
                    is_leaf: true,
                }),
            ]
        );
    }

    #[test]
    fn conflincting_key_returns_error() {
        let mut btree = BTree::empty(3);
        let _ = btree.insert(1, "one");
        println!("inserted");
        let result = btree.insert(1, "uno");
        assert!(matches!(
            result,
            Err(BTreeOperationError::KeyAlreadyExists(_))
        ));
    }
}

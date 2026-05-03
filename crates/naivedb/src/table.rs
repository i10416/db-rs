mod poormans;
use anyhow::Result;
use naivedb_bplustree_storage::BTree;
use naivedb_kernel::{
    disk::{PageId, tuple},
    inmemory::BufferPoolManager,
};
pub struct SimpleTable {
    pub meta_page_id: PageId,
    pub num_key_elems: usize,
}

impl SimpleTable {
    pub fn create(&mut self, bufman: &mut BufferPoolManager) -> Result<()> {
        let btree = BTree::create(bufman)?;
        self.meta_page_id = btree.meta_page_id;
        Ok(())
    }
    // A record is a list of sequences of bytes. Record is a row and each sequence of bytes is a column value.
    pub fn insert(&self, bufman: &mut BufferPoolManager, record: &[&[u8]]) -> Result<()> {
        let btree = BTree::new(self.meta_page_id);
        let mut key = vec![];
        tuple::encode(record[..self.num_key_elems].iter(), &mut key);
        let mut value = vec![];
        tuple::encode(record[self.num_key_elems..].iter(), &mut value);
        btree.insert(bufman, &key, &value)?;
        Ok(())
    }
}

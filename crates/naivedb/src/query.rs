use anyhow::Result;
use naivedb_bplustree_storage::{BTree, Iter, SearchMode};
use naivedb_kernel::{
    disk::{PageId, tuple},
    inmemory::BufferPoolManager,
};

pub type Tuple = Vec<Vec<u8>>;
pub type TupleSlice<'a> = &'a [Vec<u8>];

pub struct ExecSeqScan<'a> {
    table_iter: Iter,
    while_cond: &'a dyn Fn(TupleSlice) -> bool,
}

pub struct ExecFilter<'a> {
    item_iter: BoxExecutor<'a>,
    filter_cond: &'a dyn Fn(TupleSlice) -> bool,
}

pub trait Executor {
    fn next(&mut self, bufman: &mut BufferPoolManager) -> Result<Option<Tuple>>;
}

impl<'a> Executor for ExecSeqScan<'a> {
    fn next(&mut self, bufman: &mut BufferPoolManager) -> Result<Option<Tuple>> {
        let (primary_key_enc, cols_enc) = match self.table_iter.next(bufman)? {
            Some(pair) => pair,
            None => return Ok(None),
        };
        let mut pkey = vec![];
        tuple::decode(&primary_key_enc, &mut pkey);
        if (self.while_cond)(&pkey) {
            let mut tuple = pkey;
            tuple::decode(&cols_enc, &mut tuple);
            Ok(Some(tuple))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Executor for ExecFilter<'a> {
    fn next(&mut self, bufman: &mut BufferPoolManager) -> Result<Option<Tuple>> {
        while let Some(item) = self.item_iter.next(bufman)? {
            if (self.filter_cond)(&item) {
                return Ok(Some(item));
            } else {
                continue;
            }
        }
        Ok(None)
    }
}

pub type BoxExecutor<'a> = Box<dyn Executor + 'a>;

pub trait PlanNode {
    fn start(&'_ self, bufman: &mut BufferPoolManager) -> Result<BoxExecutor<'_>>;
}

pub enum TupleSearchMode<'a> {
    Start,
    Key(&'a [&'a [u8]]),
}

impl<'a> TupleSearchMode<'a> {
    pub fn encode(&self) -> SearchMode {
        match self {
            Self::Start => SearchMode::Start,
            Self::Key(keys) => {
                let mut keys_enc = vec![];
                tuple::encode(keys.iter(), &mut keys_enc);
                SearchMode::Key(keys_enc)
            }
        }
    }
}

impl<'a> From<&TupleSearchMode<'a>> for SearchMode {
    fn from(value: &TupleSearchMode<'a>) -> Self {
        value.encode()
    }
}

pub struct SeqScan<'a> {
    pub table_meta_page_id: PageId,
    pub search_mode: TupleSearchMode<'a>,
    pub while_cond: &'a dyn Fn(TupleSlice) -> bool,
}

impl<'a> PlanNode for SeqScan<'a> {
    fn start(&'_ self, bufman: &mut BufferPoolManager) -> Result<BoxExecutor<'_>> {
        let btree = BTree::new(self.table_meta_page_id);
        let iter = btree.search(bufman, (&self.search_mode).into())?;
        let exec = ExecSeqScan {
            table_iter: iter,
            while_cond: self.while_cond,
        };
        Ok(Box::new(exec))
    }
}

pub struct Filter<'a> {
    pub cond: &'a dyn Fn(TupleSlice) -> bool,
    pub inner_plan: &'a dyn PlanNode,
}

impl<'a> PlanNode for Filter<'a> {
    fn start(&'_ self, bufman: &mut BufferPoolManager) -> Result<BoxExecutor<'_>> {
        let inner = self.inner_plan.start(bufman)?;
        Ok(Box::new(ExecFilter {
            item_iter: inner,
            filter_cond: self.cond,
        }))
    }
}

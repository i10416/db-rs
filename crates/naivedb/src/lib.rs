use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    ops::{Index, IndexMut},
    rc::Rc,
};

use crate::disk::{DiskManager, Page, PageId};

pub mod disk;
pub mod inmemory;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("no free buffer available in buffer pool")]
    NoFreeBuffer,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BufferId(usize);

pub struct BufferPool {
    buffers: Vec<Frame>,
    next_victim_id: BufferId,
}

impl Index<BufferId> for BufferPool {
    type Output = Frame;

    fn index(&self, index: BufferId) -> &Self::Output {
        &self.buffers[index.0]
    }
}

impl IndexMut<BufferId> for BufferPool {
    fn index_mut(&mut self, index: BufferId) -> &mut Self::Output {
        &mut self.buffers[index.0]
    }
}

pub struct Frame {
    // usage_count: u64,
    buffer: Rc<Buffer>,
}

pub struct Buffer {
    page_id: PageId,
    page: RefCell<Page>,
    is_dirty: Cell<bool>,
}

// Buffer pool stores a limited number of pages in memory for faster access.
// If the page gets "dirty", it writes it back to the disk to persist the changes.
pub struct BufferPoolManager {
    page_table: HashMap<PageId, BufferId>,
    pool: BufferPool,
    disk: DiskManager,
}

impl BufferPoolManager {
    // fetch a page from the buffer pool if any. Otherwise, it reads the disk and loads data
    // onto the buffer pool.
    // If the buffer pool is full, it evicts a page to make a space for the requested page.
    pub fn fetch_page(&mut self, page_id: PageId) -> Result<Rc<Buffer>, Error> {
        if let Some(&buffer_id) = self.page_table.get(&page_id) {
            let frame = &mut self.pool[buffer_id];
            // increase usage count
            Ok(frame.buffer.clone())
        } else {
            let buffer_id = self.pool.evict().ok_or(Error::NoFreeBuffer)?;
            // get a frame with a page to be evicted
            let frame = &mut self.pool[buffer_id];
            let evict_page_id = frame.buffer.page_id;
            {
                let buf = Rc::get_mut(&mut frame.buffer).unwrap();
                if buf.is_dirty.get() {
                    // write the changes to the disk
                    self.disk
                        .write_page_data(evict_page_id, buf.page.get_mut())
                        .unwrap();
                }
                // replace the buffer with new data from the disk
                buf.page_id = page_id;
                buf.is_dirty.set(false);
                self.disk.read_page_data(page_id, buf.page.get_mut())?;
            }
            let page = Rc::clone(&frame.buffer);
            self.page_table.remove(&evict_page_id);
            self.page_table.insert(page_id, buffer_id);
            Ok(page)
        }
    }
}

impl BufferPool {
    fn size(&self) -> usize {
        self.buffers.len()
    }
    pub fn evict(&mut self) -> Option<BufferId> {
        let pool_size = self.size();

        todo!()
    }
}

mod syntax;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    rc::Rc,
};

use crate::disk::{DiskManager, Page, PageId};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("no free buffer available in buffer pool")]
    NoFreeBuffer,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BufferId(usize);

/// [BufferPool] holds disk data cache in a consecutive memory location as a vector of [Frame]s
pub struct BufferPool {
    buffers: Vec<Frame>,
    next_victim_id: BufferId,
}

impl BufferPool {
    pub fn new(pool_size: usize) -> Self {
        let mut buffers = vec![];
        buffers.resize_with(pool_size, Default::default);

        Self {
            buffers,
            next_victim_id: Default::default(),
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        todo!()
    }
}

/// [Frame] is a logical unit to store underlying page data(A.K.A [Buffer])
/// combined with [Frame::usage_count] metadata for cache invalidation
pub struct Frame {
    // if this valud is > 0, this frame is in use
    usage_count: u64,
    buffer: Rc<Buffer>,
}

pub struct Buffer {
    pub page_id: PageId,
    pub page: RefCell<Page>,
    pub is_dirty: Cell<bool>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            page_id: Default::default(),
            page: RefCell::new([0u8; DiskManager::PAGE_SIZE]),
            is_dirty: Cell::new(false),
        }
    }
}

// Buffer pool stores a limited number of pages in memory for faster access.
// If the page gets "dirty", it writes it back to the disk to persist the changes.
pub struct BufferPoolManager {
    // map from a page id to a buffer(cache) id for quick access
    page_table: HashMap<PageId, BufferId>,
    pool: BufferPool,
    disk: DiskManager,
}

impl BufferPoolManager {
    pub fn new(disk: DiskManager, pool: BufferPool) -> Self {
        Self {
            page_table: HashMap::default(),
            pool,
            disk,
        }
    }
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

    pub fn create_page(&mut self) -> Result<Rc<Buffer>, Error> {
        let buffer_id = self.pool.evict().ok_or(Error::NoFreeBuffer)?;
        let frame = &mut self.pool[buffer_id];
        let evict_page_id = frame.buffer.page_id;
        let allocated_page_id = {
            let buf = Rc::get_mut(&mut frame.buffer).unwrap();
            if buf.is_dirty.get() {
                self.disk
                    .write_page_data(evict_page_id, buf.page.get_mut())?;
            }
            let page_id = self.disk.allocate_page();
            *buf = Buffer::default();
            buf.page_id = page_id;
            buf.is_dirty.set(true);
            frame.usage_count = 1;
            page_id
        };
        let page = Rc::clone(&frame.buffer);
        self.page_table.remove(&evict_page_id);
        self.page_table.insert(allocated_page_id, buffer_id);
        Ok(page)
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        for (&page_id, &buffer_id) in self.page_table.iter() {
            let frame = &self.pool[buffer_id];
            let mut page = frame.buffer.page.borrow_mut();
            self.disk.write_page_data(page_id, page.as_mut())?;
            frame.buffer.is_dirty.set(false);
        }
        self.disk.sync()?;
        Ok(())
    }
}

impl BufferPool {
    fn size(&self) -> usize {
        self.buffers.len()
    }
    // returns None if no buffer space is available
    pub fn evict(&mut self) -> Option<BufferId> {
        let pool_size = self.size();
        let mut consecutive_pinned = 0;
        let victim_id = loop {
            let next_victim_id = self.next_victim_id;
            let frame = &mut self[next_victim_id];
            if frame.usage_count == 0 {
                break self.next_victim_id;
            }
            // Rc::get_mut returns some, meaning no other reference exists,
            // so we must clean up stale usage count here.
            if Rc::get_mut(&mut frame.buffer).is_some() {
                frame.usage_count -= 1;
                consecutive_pinned = 0;
            } else {
                // the frame is in use(it is referenced from somewhere)
                consecutive_pinned += 1;
                if pool_size == consecutive_pinned {
                    // buffer is out of stock, so giving up
                    return None;
                }
            }
            self.next_victim_id = self.increment_id(self.next_victim_id);
        };
        Some(victim_id)
    }
    fn increment_id(&self, buf_id: BufferId) -> BufferId {
        // circulating by self.size(): 0, 1, 2, 3, ..., 0, 1, 2, 3, ...
        BufferId((buf_id.0 + 1) % self.size())
    }
}

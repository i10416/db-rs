use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

use zerocopy::{AsBytes, FromBytes};

// data structure to store variable length data into fixed size pages
pub mod slotted;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, FromBytes, AsBytes)]
#[repr(C)]
pub struct PageId(pub u64);

pub type Page = [u8; DiskManager::PAGE_SIZE];

impl PageId {
    pub fn to_u64(&self) -> u64 {
        self.0
    }
    pub const INVALID_PAGE_ID: PageId = PageId(u64::MAX);

    pub fn valid(self) -> Option<PageId> {
        if self == Self::INVALID_PAGE_ID {
            None
        } else {
            Some(self)
        }
    }
}
impl Default for PageId {
    fn default() -> Self {
        Self::INVALID_PAGE_ID
    }
}

impl From<Option<PageId>> for PageId {
    fn from(page_id: Option<PageId>) -> Self {
        page_id.unwrap_or_default()
    }
}

impl From<&[u8]> for PageId {
    fn from(bytes: &[u8]) -> Self {
        let arr = bytes.try_into().unwrap();
        PageId(u64::from_ne_bytes(arr))
    }
}

impl From<PageId> for u64 {
    fn from(value: PageId) -> Self {
        value.0
    }
}

pub struct DiskManager {
    // File performs disk I/O caching on read and write, but it is still slow.
    // On Linux platform, std::os::unix::fs::OpenOptionsExt provides a way to
    // disable disk I/O caching via `custom_flags(O_DIRECT)`;
    heap_file: File,
    next_page_id: u64,
}

impl DiskManager {
    pub const PAGE_SIZE: usize = 4096;
    ///
    /// ```txt
    /// heap_file: | page 0 | page 1 | ... | page N-1 |
    /// ```
    ///
    pub fn new(heap_file: File) -> io::Result<Self> {
        let size = heap_file.metadata()?.len();
        // every operation is performed per page, so it is safe to assume that
        // size is divisible by PAGE_SIZE.
        let next_page_id = size / Self::PAGE_SIZE as u64;
        Ok(Self {
            heap_file,
            next_page_id,
        })
    }
    pub fn open(data_file_path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(data_file_path)?;
        Self::new(file)
    }

    pub fn allocate_page(&mut self) -> PageId {
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        PageId(page_id)
    }
    pub fn read_page_data(&mut self, page_id: PageId, data: &mut [u8]) -> io::Result<()> {
        let offset = Self::PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(SeekFrom::Start(offset))?;
        // NOTE:
        // Rust has read_exact_at under os::unix::fs::FileExt namespace which allows us to read data in a single step.
        // On Windows platform, it does move the cursor first and then read the data.
        self.heap_file.read_exact(data)
    }
    pub fn write_page_data(&mut self, page_id: PageId, data: &[u8]) -> io::Result<()> {
        let offset = Self::PAGE_SIZE as u64 * page_id.to_u64();
        self.heap_file.seek(SeekFrom::Start(offset))?;
        self.heap_file.write_all(data)
    }
    pub fn sync(&mut self) -> io::Result<()> {
        self.heap_file.sync_all()
    }
}

use naivedb_bplustree_storage::Pair;
use naivedb_kernel::disk::{DiskManager, PageId, slotted::Slotted};
use prelude::traversal::binary_search_by;
use zerocopy::ByteSlice;

// extremely naive, unsafe implementation to demonstrate the naivest table implementation idea
// without b+tree
#[allow(unused)]
pub struct Table {
    disk: DiskManager,
    page_count: usize,
}

impl Table {
    #[allow(unused)]
    pub fn insert(&mut self, key: &[u8], value: &[u8]) {
        let page_count = self.disk.page_count();
        let last_page_id = page_count - 1;
        let last_page_id = PageId(last_page_id as _);
        let mut page = vec![];
        let _ = self.disk.read_page_data(last_page_id, &mut page).unwrap();
        let mut slotted = Slotted::<&mut [u8]>::new(&mut page);
        let slot_id = Self::search_slot_id(&slotted, key).unwrap();
        slotted.insert(slot_id, value.len()).unwrap();
        let pair = Pair { key, value };
        slotted[0].copy_from_slice(&pair.to_bytes()[..]);
        let _ = self.disk.write_page_data(last_page_id, &page).unwrap();
        let _ = self.disk.sync().unwrap();
    }

    fn search_slot_id<T: ByteSlice>(slotted: &Slotted<T>, key: &[u8]) -> Result<usize, usize> {
        binary_search_by(slotted.num_slots(), |slot_id| {
            Pair::from_bytes(&slotted[slot_id]).key.cmp(key)
        })
    }
    #[allow(unused)]
    pub fn scan(&mut self, key: &[u8]) -> Vec<Vec<u8>> {
        let mut page_id = 0;
        let mut result = vec![];
        while page_id < self.disk.page_count() {
            let mut data = vec![];
            let _ = self
                .disk
                .read_page_data(PageId(page_id as _), &mut data)
                .unwrap();
            let slotted = Slotted::<&[u8]>::new(&mut data);
            match Self::search_slot_id(&slotted, key).ok() {
                Some(slot_id) => {
                    let pair = Pair::from_bytes(&slotted[slot_id]);
                    result.push(pair.value.to_vec());
                    page_id += 1;
                }
                None => {
                    page_id += 1;
                }
            };
        }
        result
    }
}

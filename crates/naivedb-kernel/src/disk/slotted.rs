use std::ops::{Index, IndexMut, Range};

use zerocopy::{AsBytes, ByteSlice, ByteSliceMut, FromBytes, FromZeroes, LayoutVerified};

pub struct Slotted<B> {
    header: LayoutVerified<B, Header>,
    body: B,
}

impl<T> Slotted<T> {
    pub fn size_of_ptr() -> usize {
        size_of::<Ptr>()
    }
}

// typed, read only view to underlying bytes
impl<B: ByteSlice> Slotted<B> {
    pub fn new(bytes: B) -> Self {
        let (header, body) = LayoutVerified::new_from_prefix(bytes).expect("ok");
        Self { header, body }
    }

    pub(crate) fn data(&self, ptr: Ptr) -> &[u8] {
        &self.body[ptr.range()]
    }
    pub fn capacity(&self) -> usize {
        self.body.len()
    }

    pub fn num_slots(&self) -> usize {
        self.header.num_slots as usize
    }

    pub fn free_space(&self) -> usize {
        self.header.free_space_end as usize - self.size_of_pointers()
    }

    pub(crate) fn pointers(&self) -> Pointers<&[u8]> {
        Pointers::new_slice(&self.body[..self.size_of_pointers()]).unwrap()
    }

    fn size_of_pointers(&self) -> usize {
        Self::size_of_ptr() * self.header.num_slots as usize
    }
}

// - mutable operation on slotted table
//
// basic mutable operation consists of the following steps
// - mutate the buffer
//    - insert: add a new pointer and data
//    - remove: invalidate a pointer (and remove the pointed data?)
// - compact/fill the vacant spaces
impl<B: ByteSliceMut> Slotted<B> {
    pub fn initialize(&mut self) {
        self.header.num_slots = 0;
        self.header.free_space_end = self.body.len() as _;
    }
    fn pointers_mut(&mut self) -> Pointers<&mut [u8]> {
        let pointers_size = self.size_of_pointers();
        Pointers::new_slice(&mut self.body[..pointers_size]).unwrap()
    }
    // mutate the internal state such that
    // a new pointer(`pointers[index]`) points to a new slot of size = `size`.
    // At this point, the slot is not filled yet.
    pub fn insert(&mut self, index: usize, len: usize) -> Option<()> {
        if self.free_space() >= Self::size_of_ptr() + len {
            self.header.free_space_end -= len as u16;
            let free_space_offset = self.header.free_space_end;
            let num_slots = self.num_slots();
            // create a space to store a new pointer at index
            self.pointers_mut()
                .copy_within(index..num_slots as usize, index + 1);
            self.header.num_slots += 1;
            let pointer = &mut self.pointers_mut()[index];
            pointer.offset = free_space_offset;
            pointer.size = len as u16;
            self.header.num_slots += 1;
            Some(())
        } else {
            None
        }
    }
    fn data_mut(&mut self, pointer: Ptr) -> &mut [u8] {
        &mut self.body[pointer.range()]
    }

    // mutate the internal state such that
    // a pointer(`pointers[index]`) is dropped and no space is vacant
    pub fn remove(&mut self, index: usize) {
        self.resize(index, 0);
        self.pointers_mut().copy_within(index + 1.., index);
        self.header.num_slots -= 1;
    }

    // mutate the internal state such that
    // a pointers[index] points to a slot of non-negative size `len_new`.
    // If the new size is 0, it invalidates the pointer.
    fn resize(&mut self, index: usize, len_new: usize) -> Option<()> {
        let mut ptr = self.pointers()[index];
        let original_size = ptr.size as isize;
        let diff = len_new as isize - original_size;
        if original_size == len_new as isize {
            Some(())
        } else if diff > self.free_space() as isize {
            None
        } else {
            let next_free_space_end = self.header.free_space_end as isize - diff;
            let original_data_offset = ptr.offset as usize;
            self.body.as_bytes_mut().copy_within(
                self.header.free_space_end as usize..original_data_offset as usize,
                next_free_space_end as usize,
            );
            self.header.free_space_end = next_free_space_end as _;
            for p in self.pointers_mut().iter_mut() {
                if p.offset as usize <= original_data_offset {
                    p.offset -= diff as u16;
                }
            }
            ptr.size = len_new as u16;
            if len_new == 0 {
                // special case:
                // empty pointer points to the start of data slots section
                ptr.offset = self.header.free_space_end;
            }
            Some(())
        }
    }
}

// select a data by an index of pointer
impl<B: ByteSlice> Index<usize> for Slotted<B> {
    type Output = [u8];
    fn index(&self, index: usize) -> &Self::Output {
        self.data(self.pointers()[index])
    }
}

impl<B: ByteSliceMut> IndexMut<usize> for Slotted<B> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data_mut(self.pointers()[index])
    }
}

// It allows us to regard value of type B as a slice of Ptr without copying.
// For example
//
// ```rust
// let items: Pointers<&[u8]> = todo!();
// let ptr = items[0];
// ```
//
pub(crate) type Pointers<B> = LayoutVerified<B, [Ptr]>;

// a pointer to positive size bytes in a byte sequence
#[derive(Debug, FromZeroes, FromBytes, AsBytes, Clone, Copy)]
#[repr(C)]
pub(crate) struct Ptr {
    offset: u16,
    // if size is 0, it points to nothing
    size: u16,
}

impl Ptr {
    pub fn range(&self) -> Range<usize> {
        (self.offset as usize)..((self.offset + self.size) as usize)
    }
}

#[derive(Debug, FromZeroes, FromBytes, AsBytes)]
#[repr(C)]
pub struct Header {
    free_space_start: u16,
    free_space_end: u16,
    num_slots: u16,
}

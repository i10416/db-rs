Q: What is the difference between Disk I/O and Momory I/O?

- SRAM (Inside CPU): Like the tools in your hands. It is tiny but instantly accessible.
- DRAM (On Motherboard): Like a workbench nearby. It holds much more, but you have to reach for it, which takes slightly longer.
- Disk (SSD/HDD): Like a storage cabinet in another room. It holds everything, but getting something out requires a long walk.

Q: Why `page_table` is `HashMap<PageId, BufferId>` instad of `HashMap<PageId, Page>`?

It seems we want to store data in consecutive memory location for cache locality.

Q: How to store data in B+ tree?

B+tree determines where to store data and slotted data structure defines how to store the data.

B+tree identifies a page to store a key-value pair. As both key and value are in variable length,
we want to efficiently store variable length data into fixed size pages.
"Slot table" covers this uscase.

```
+---------------------------------------------------------------+
| PAGE HEADER (Checksum, Free Space Pointer, Slot Count)        |
+---------------------------------------------------------------+

| Slot 0 (Offset: 480, Len: 20) | Slot 1 (Offset: 450, Len: 30) |
+---------------------------------------------------------------+
| Slot 2 (Offset: 400, Len: 50) | ... (Empty Slots)             |
+---------------------------------------------------------------+
|                                                               |
|                         FREE SPACE                            |
|                                                               |
+---------------------------------------------------------------+
| <------ Data for Slot 2 ------- | <--- Data for Slot 1 ------ |
+---------------------------------------------------------------+
| <------- Data for Slot 0 ------------------------------------>|
+---------------------------------------------------------------+
```

The page header points to the free space start and the free space end.

> The header region stores metadata about the page and always start at the beginning of the page. It holds information like the page id, B-tree node type (root, interior, leaf), and the start and end of the free space region. It’s also common to include things like a magic number, version number, a checksum to protect against data corruption, bit flags to indicate things like compression, sibling page ids, etc.

- https://siemens.blog/posts/database-page-layout/
- https://rabbitfoot141.hatenablog.com/entry/2019/12/03/000000#Tuple-Oriented


In B+tree, we store `(key -> [u8] as PageId)` in branch nodes and `(key -> [u8])` in leaf nodes by slot_id, which is an index of pointer in slotted table looked up by key.
The signiture of look up operation is `key => slot_id => value in the slot`.

To be more precise, key is a non-empty list of non-empty byte sequences encoded in memcmpable format(`memcmpable::encode: [[u8]] => [u8] ^ ordering`)
where `^ ordering` indicates the resulting byte array is sensibly sortable
while the original list of byte sequences is not sensibly sortable.

Q: How zerocopy works?

```rs
pub struct Slotted<B> {
    // This creates a typed, read-only view to underlying bytes
    // without instantiating actual `Header` type
    header: LayoutVerified<B, Header>,
    body: B,
}
```

Q: Why Ptr has a `_pad` to fill 64 bytes?

???

Q: How search works?

search returns a sequence of buffers(page_id, page, etc.)

Data is not loaded until necessary. Node type, Leaf type and Branch type work as a typed-facade to
underlying data(zero copy abstraction).
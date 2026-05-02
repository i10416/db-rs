## Zero copy B+Tree Design

B+Tree is a kind of typed, zerocopy facade to underlying byte sequence.
It provides an API to traverse/manipulate the underlying data as if it forms a tree structure,
hiding the implementation details to compute offsets in the byte sequence.

It is quite similar to a Sudachi binary dictionary reader which reads a large trie data structure
serialized into a byte sequence. It does not load the entire dictionary on memory.
Instead, it computes an offset to a metadata set for a given prefix and builds a lattice(a kind of graph data structure) to compute the minimal cost path.
For example, if Sudachi encounters "tr", it computes the offset to a metadata set for "tr".
The metadata might contain "trie", "try", "train", etc. combined with statistics(e.g. occurance cost).

The difference is, trie dictionary for morpheme analysis is read only
while B+Tree might mutate underlying byte sequence on write.

pub mod disk;
// A kind of inmemory cache for disk I/O.
// The cache size is much smaller than disk, so it is necessary to
// invalidate cache.
pub mod inmemory;

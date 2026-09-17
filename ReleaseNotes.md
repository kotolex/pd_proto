# Release Notes

## 2026-09-17 (Version 1.1.0)

### Added
* **Checksum Support (`checksum` and `checksums`):** Introduced support for working with adler32 checksum.

### Changed
* **Internal architecture:** Make internal functions private to clarify public interface
* **Internal architecture:** Decrease default nesting limit to 500

### Fixed
* **Protocol internal:** Fix bug with big negative integer overflow



## 2026-09-16 (Version 1.0.2)

### Added
* **File I/O Support (`dump` and `load`):** Introduced full support for working with file-like objects, matching the standard `pickle` and `json` APIs.
* **Efficient Buffered I/O:** Serialized data is streamed directly to files using highly optimized buffered writing.
* **Zero-Copy Performance (`memmap`):** The `load` method utilizes memory mapping (`memmap2`) under the hood to parse files directly from disk without allocations.

### Changed
* **Internal I/O architecture:** Replaced raw byte buffers with stream-based reader and writer traits to support arbitrary files on disk.

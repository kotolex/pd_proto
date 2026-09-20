# Release Notes

## 2026-09-20 (Version 1.1.0)

### Added
* **Explain (`explain` and `explains`):** Introduced support for step-by-step parsing report for debugging.
* **Checksum Support (`checksum` and `checksums`):** Introduced support for working with adler32 checksum.
* Readme sections, examples and tests for new features

### Changed
* **Internal architecture:** Make internal functions private to clarify public interface
* **Internal architecture:** Decrease default nesting limit to 500
* **Internal architecture:** Move check for unparsed bytes to Rust
* **Internal architecture:** Move some common functions to utils

### Fixed
* **Protocol internal:** Fix bug with big negative integer overflow
* **Protocol internal:** Fix bug with string bytes (not enough to parse)
* **Protocol internal:** Fix bug with bytes (not enough to parse)
* **Protocol internal:** Remove unnecessary size parsing for optimized strings
* **Texts:** Fix a lot of texts (docs, errors, etc.)



## 2026-09-16 (Version 1.0.2)

### Added
* **File I/O Support (`dump` and `load`):** Introduced full support for working with file-like objects, matching the standard `pickle` and `json` APIs.
* **Efficient Buffered I/O:** Serialized data is streamed directly to files using highly optimized buffered writing.
* **Zero-Copy Performance (`memmap`):** The `load` method utilizes memory mapping (`memmap2`) under the hood to parse files directly from disk without allocations.

### Changed
* **Internal I/O architecture:** Replaced raw byte buffers with stream-based reader and writer traits to support arbitrary files on disk.

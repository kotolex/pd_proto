# pd_proto (Peace Data Protocol)

> All detailed technical specifications, internal byte structures, layout constraints, and type tags can be explored in the comprehensive [Protocol Specification](https://github.com/kotolex/pd_proto/blob/master/SPECIFICATION.md).

<p>
    <a href="https://pypi.org/project/pd-proto/"><img src="https://img.shields.io/pypi/status/pd-proto?style=flat-square"></a>
    &nbsp;
	<a href="https://pypi.org/project/pd-proto/"><img src="https://img.shields.io/pypi/v/pd-proto?style=flat-square"></a>
    &nbsp;
	<a href="https://pypi.org/project/pd-proto/"><img src="https://img.shields.io/pypi/pyversions/pd-proto?style=flat-square"></a>
    &nbsp;
	<a href="https://pypi.org/project/pd-proto/"><img src="https://img.shields.io/github/last-commit/kotolex/pd_proto/master?style=flat-square"></a>
</p>

## Introduction

Python has firmly established itself as the most popular and widely adopted programming language in the world. At the heart of virtually every Python application - ranging from microservices and web backends to data pipelines and machine learning infrastructure - lies the heavy utilization of standard built-in data types. 

Because standard applications spend the vast majority of their CPU cycles manipulating and transmitting these exact primitives, **`pd_proto` specializes exclusively in the ultra-fast serialization and deserialization of Python's native built-in types**. By focusing on data structures rather than complex object graphs, class inheritance, or custom behavior, `pd_proto` bypasses the systemic overhead found in traditional serialization frameworks.

**System Requirements:**
- OS: 64-bit Operating System (Windows, Linux, macOS)
- Python: Version 3.8 or higher

## Core Advantages

* **Zero Dependencies:** Built entirely with native Python C-API bindings and a highly optimized Rust core, requiring no third-party libraries or external runtimes.
* **Platform & Runtime Independent:** Fully decoupled from the underlying Operating System and specific Python version updates, ensuring absolute portability across 64-bit Linux, macOS, and Windows.
* **Minimal Binary Footprint:** Generates compiled payloads that are significantly smaller than equivalent byte streams produced by native `pickle` or `json`.
* **Blazing Fast Performance:** Drastically outperforms native CPython serializers by stripping away dynamic object reflection and memory allocation overhead.

## Key Architectural Enhancements

Knowing the practical realities of data transmission, `pd_proto` introduces several architectural mechanics to maximize efficiency:

* **Varint Length Encoding:** Length descriptors for collections, strings, and integers utilize variable-length integers (LEB128). Short data segments consume a single byte for length instead of being penalized by fixed 4-byte or 8-byte headers.
* **Optimized Floating-Point Structures:** Primitives with up to 6 decimal places (such as `12.22` or `3.14`) undergo an automated scaling routine that condenses standard 8-byte IEEE 754 floats into tightly packed Varints.
* **Intelligent Inline Tags:** Highly recurrent constants (`0`, `1`–`13`, `100`, `1000`) and standard short collection shapes (e.g., a tuple containing exactly 2 or 3 elements) utilize dedicated optimizing tags. This entirely removes the need to write separate size or value descriptors into the stream.
* **Localized String & Big Integers & Float Caching:** The processing pipeline uses isolated, bounded in-memory caches during execution. By avoiding repeated memory allocation in the Python heap for highly recurrent strings or numeric primitives, the parser maintains an incredibly low execution profile that easily fits into the CPU's L1 cache.

## Installation

Install the compiled library directly from PyPI using `pip`:

```bash
pip install pd_proto
```

## Supported Types

The protocol strictly and natively processes the following built-in types:
`None` | `bool` | `int` | `float` | `str` | `bytes` | `list` | `tuple` | `dict` | `set` | `datetime`

*Note: User-defined subclasses or structures containing application-specific logic must be sanitized and converted into a standard native schema (such as a dictionary or tuple) prior to serialization.*

## Usage

Just like with pickle and json, use dumps to serialize data and loads to deserialize it.

```python
from pd_proto import dumps, loads

data = {"text": "some text", "is_valid": True, "unique_tags": {"apple", "banana", "cherry"}}
bts = dumps(data)
print(bts)  # b'\x01\x11\x03,text1some text0is_valid\x013unique_tags\x10\x03.banana-apple.cherry'
parsed = loads(bts)
print(parsed)  # {'text': 'some text', 'is_valid': True, 'unique_tags': {'banana', 'apple', 'cherry'}}
assert data == parsed  # The protocol guarantees equality after deserialization
```

You can use any supported (built-in) types and collections composed of supported types. If an unsupported type is encountered in the data, you will receive a clear error message about it.

### Parameters

You can configure certain serialization parameters to boost speed at the cost of the resulting byte array size. Since optimal defaults are already selected, tweaking these settings is generally not recommended.

**max_depth** - Specifies the maximum allowed nesting depth for collections, throwing an exception if exceeded. Defaults to 1000. Setting it to a negative value or 0 disables the depth check, which may lead to stack overflow and application crashes.

**float_limit** - Specifies the threshold for float optimization. For details on how this optimization works, refer to the protocol specification. Defaults to 268_435_455.0. If set to a negative value or 0, no attempts will be made to optimize float sizes. This may boost performance but expands the result size since every float takes up 8 bytes.

**string_length_limit** - Specifies the string size threshold for compression. Strings larger than this value (in bytes) will be compressed. Defaults to 100 bytes. If set to a negative value or 0, no strings will be compressed - for instance, if you know the data is already incompressible.

### Errors

Every error has a clear, self-explanatory name and includes a message describing the issue. If you are unsure which specific exception might be raised, you can catch the base exception for all protocol errors(PDProtoError).

```python
from pd_proto import dumps, PDProtoError

data = frozenset([1, 2])
try:
    bts = dumps(data)
except PDProtoError:
    print("Cant use it")  # frozenset is not supported!
```
**Note on frozenset:** Despite being a built-in type, `frozenset` is seldom used and is identical to a standard `set` from a data perspective (ignoring behavior). If you need to serialize it, just use a regular `set`.

### Files

The library works with file-like objects exactly like the standard `pickle` and `json` modules. Simply use the standard `dump()` and `load()` methods.

* **Efficient Buffered I/O:** All operations utilize highly efficient buffered streaming under the hood.
* **Important for Reading:** The `load()` method requires a **real file present in the filesystem** (with a valid OS descriptor/handle) to enable zero-copy memory mapping. In-memory streams like `io.BytesIO` or network sockets are not supported for reading for now.

```python
from pd_proto import dump, load

data = {1: 1, "2": "2", 3: 3.14, 4: [1, 2, 3]}
# Note: The file must be opened in binary mode ('b') since the library works with bytes, not text.
with open("data.bin", "wb") as file_to_write:
    dump(file_to_write, data)  # Serializes the object and writes it directly to the file.

with open("data.bin", "rb") as file_to_read:
    parsed = load(file_to_read) # Deserializes the object from the file.

assert data == parsed  # The protocol guarantees full equality after deserialization.
```

## Comparison with JSON

The primary benefit of JSON over `pd_proto` is human-readability. Otherwise, JSON produces larger payloads and performs slower.

For obvious architectural reasons, the binary payloads generated by `pd_proto` are significantly more compact - often reducing data size by up to 50% compared to standard JSON text strings. `pd_proto` delivers substantially faster execution speeds while simultaneously maintaining a much smaller byte footprint. 

Furthermore, unlike JSON, `pd_proto` provides native, out-of-the-box support for complex types and states such as `datetime`, `set`, `tuple`, `bytes` as well as IEEE 754 special float values (`NaN`, `Inf`, and `-Inf`). 

A notorious limitation of JSON is its inability to serialize bytes and dates, forcing developers to convert it into text strings. This introduces the systemic overhead of string parsing on the receiving end, which requires strict prior coordination of the exact date format or bytes encoding. `pd_proto` completely eliminates this friction, packing and restoring directly into standard Python `datetime` or `bytes` objects.

* **Important Notice on Naive Datetimes:** Please note that naive `datetime` objects (those without an explicit timezone) are serialized as raw timestamps. If a naive datetime is packed on a machine in one geographic timezone and unpacked on a machine running in a different timezone, its absolute value will shift accordingly. This fully mirrors native CPython runtime behavior and must be accounted for during cross-region data transfers.

## Comparison with Pickle

While `pickle` is highly optimized and executes rapidly (particularly within Linux environments), `pd_proto` delivers matching or superior processing speeds depending on the specific volume and composition of the dataset. Besides, `pd_proto` consistently yields a more compact serialized byte footprint.

A distinct advantage of `pickle` is its inherent capacity to serialize user-defined class instances and custom subclasses derived from built-in types - a capability explicitly omitted from `pd_proto`. 
Instead, `pd_proto` maintains a strict, uncompromised focus on data structures, ensuring maximum throughput and minimal storage footprint. 

Furthermore, `pd_proto` is entirely decoupled from specific Python runtime versions and is uniformly optimized across all operating systems, whereas `pickle` exhibits a pronounced performance bias toward Linux environments.

## Benchmarks

The size of serialized data remains identical across different operating systems and Python versions. 
However, execution speed may vary depending on data volume, content, and the OS itself. 
For instance, pickle is faster on Linux but processes `datetime` slowly. Below are a few benchmarks on the simplest data across various operating systems.

If you add datetimes to this dataset, the performance gap with pickle becomes even more significant. As for JSON, you would have to convert datetimes to strings beforehand, since it does not support these data types natively. 

**Windows 10 (Python 3.13.1 [MSC v.1942 64 bit (AMD64)] on win32)**

```pycon
Python 3.13.1 >>> from pd_proto import dumps
Python 3.13.1 >>> import json, pickle
Python 3.13.1 >>> from timeit import timeit
Python 3.13.1 >>> data = {1:1, "2":"2", 3:3.14, 4:[1,2,3]}
Python 3.13.1 >>> dumps(data)
b'\x01\x11\x04==)2%\x00?\x16\xba\x02@S=>?'
Python 3.13.1 >>> json.dumps(data)
'{"1": 1, "2": "2", "3": 3.14, "4": [1, 2, 3]}'
Python 3.13.1 >>> pickle.dumps(data)
b'\x80\x04\x95&\x00\x00\x00\x00\x00\x00\x00}\x94(K\x01K\x01\x8c\x012\x94h\x01K\x03G@\t\x1e\xb8Q\xeb\x85\x1fK\x04]\x94(K\x01K\x02K\x03eu.'
Python 3.13.1 >>> timeit("dumps(data)", "from __main__ import data, dumps, pickle", number=1000_000)
0.6568191999976989
Python 3.13.1 >>> timeit("pickle.dumps(data)", "from __main__ import data, dumps, pickle", number=1000_000)
0.8377401000034297
Python 3.13.1 >>> timeit("json.dumps(data)", "from __main__ import data, dumps, pickle, json", number=1000_000)
1.9830707000000984
```

**MacOS Tahoe (Python 3.13.1 [Clang 15.0.0 (clang-1500.3.9.4)] on darwin)**

```pycon
>>> data = {1:1, "2":"2", 3:3.14, 4:[1,2,3]}
>>> import pickle, json
>>> from pd_proto import dumps
>>> from timeit import timeit
>>> dumps(data)
b'\x01\x11\x04==)2%\x00?\x16\xba\x02@S=>?'
>>> json.dumps(data)
'{"1": 1, "2": "2", "3": 3.14, "4": [1, 2, 3]}'
>>> pickle.dumps(data)
b'\x80\x04\x95&\x00\x00\x00\x00\x00\x00\x00}\x94(K\x01K\x01\x8c\x012\x94h\x01K\x03G@\t\x1e\xb8Q\xeb\x85\x1fK\x04]\x94(K\x01K\x02K\x03eu.'
>>> timeit("dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
0.7029794589616358
>>> timeit("pickle.dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
0.7053574579767883
>>> timeit("json.dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
1.838942875037901
```

**Linux Ubuntu 26 (Python 3.14.4 [GCC 15.2.0] on linux)**

```pycon
>>> import pickle, json
... from pd_proto import dumps
... from timeit import timeit
...
>>> data = {1:1, "2":"2", 3:3.14, 4:[1,2,3]}
>>> dumps(data)
b'\x01\x11\x04==)2%\x00?\x16\xba\x02@S=>?'
>>> json.dumps(data)
'{"1": 1, "2": "2", "3": 3.14, "4": [1, 2, 3]}'
>>> pickle.dumps(data)
b'\x80\x05\x95&\x00\x00\x00\x00\x00\x00\x00}\x94(K\x01K\x01\x8c\x012\x94h\x01K\x03G@\t\x1e\xb8Q\xeb\x85\x1fK\x04]\x94(K\x01K\x02K\x03eu.'
>>> timeit("dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
0.7428264559999889
>>> timeit("pickle.dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
0.8976360610000143
>>> timeit("json.dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
2.200423194999985
```

**Linux Debian 13 (Python 3.13.5 [GCC 14.2.0] on linux)**

```pycon
>>> import pickle, json
... from pd_proto import dumps
... from timeit import timeit
...
>>> data = {1:1, "2":"2", 3:3.14, 4:[1,2,3]}
>>> dumps(data)
b'\x01\x11\x04==)2%\x00?\x16\xba\x02@S=>?'
>>> json.dumps(data)
'{"1": 1, "2": "2", "3": 3.14, "4": [1, 2, 3]}'
>>> pickle.dumps(data)
b'\x80\x04\x95&\x00\x00\x00\x00\x00\x00\x00}\x94(K\x01K\x01\x8c\x012\x94h\x01K\x03G@\t\x1e\xb8Q\xeb\x85\x1fK\x04]\x94(K\x01K\x02K\x03eu.'
>>> timeit("dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
0.7543717089999973
>>> timeit("pickle.dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
0.8993470230000185
>>> timeit("json.dumps(data)", "from __main__ import dumps, data, pickle, json", number=1000_000)
2.2170975030000477
```
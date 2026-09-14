# pd_proto (Peace Data Protocol)

> All detailed technical specifications, internal byte structures, layout constraints, and type tags can be explored in the comprehensive [Protocol Specification](SPECIFICATION.md).

## Introduction

Python has firmly established itself as the most popular and widely adopted programming language in the world. At the heart of virtually every Python application - ranging from microservices and web backends to data pipelines and machine learning infrastructure - lies the heavy utilization of standard built-in data types. 

Because standard applications spend the vast majority of their CPU cycles manipulating and transmitting these exact primitives, **`pd_proto` specializes exclusively in the ultra-fast serialization and deserialization of Python's native built-in types**. By focusing on data structures rather than complex object graphs, class inheritance, or custom behavior, `pd_proto` bypasses the systemic overhead found in traditional serialization frameworks.

## Core Advantages

* **Zero Dependencies:** Built entirely with native Python C-API bindings and a highly optimized Rust core, requiring no third-party libraries or external runtimes.
* **Platform & Runtime Independent:** Fully decoupled from the underlying Operating System and specific Python version updates, ensuring absolute portability across Linux, macOS, and Windows.
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
pip install pd-proto
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

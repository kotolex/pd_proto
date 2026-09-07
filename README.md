## PDProto

PeaceData Protocol or just PDProto is a binary protocol for representing basic data types in Python

## _We fight for each byte!_

### ToDo
 - decrypt in Rust
 - datetimes encrypt/decrypt
 - lists and tuples of small length
 - try to return something even on failed parse?
 - if ascii then simplify string encoding (no need to encode utf-8) 


### Base points
- OS independent
- Python version independent
- order guarantee (except set)
- smaller than pickle or json
- faster than pickle on MacOS and Windows10, same speed on Linux
- no dependencies
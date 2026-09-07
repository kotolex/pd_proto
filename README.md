## PDProto

PeaceData Protocol or just PDProto is a binary protocol for representing basic data types in Python

## _We fight for each byte!_

### ToDo
 - decrypt in Rust
 - string compress option on encryption
 - float limit on encryption
 - datetimes encrypt/decrypt
 - depth limit option
 - lists and tuples of small length
 - try to return something even on failed parse?
 - if ascii then simplify string encoding (no need to encode utf-8) 


### Base point
- OS independent
- Python version independent
- order guarantee (except set)
- smaller than pickle or json
- faster than pickle
- no dependencies
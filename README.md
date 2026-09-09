## PDProto

PeaceData Protocol or just PDProto is a binary protocol for representing basic data types in Python

## _We fight for each byte!_

### ToDo
 - lists and tuples of small length
 - optimize decryption
 - depth for decypyt
 - rust docs
 - int 1



### Base points
- OS independent
- Python version independent
- order guarantee (except set)
- smaller than pickle or json
- faster than pickle on MacOS and Windows10, same speed on Linux
- faster than json for all OS (2-4 times faster)
- no dependencies
- can use nan, inf, -inf for float, can use datetimes (json cant)
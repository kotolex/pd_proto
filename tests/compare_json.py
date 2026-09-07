import json
from timeit import timeit

from pd_proto import encrypt, decrypt

data = ["text", [12569, [-1.234, 4.5678], {1: [[1, 2], [3, 4]]}], None, True, False]
py_data = encrypt(data)
json_data = json.dumps(data)
result =  100 - (len(py_data) / (len(json_data) / 100))
print(f"Size difference: {len(py_data) - len(json_data)} byte, {result:.2f}% better")

print("ENCRYPT JSON")
print(timeit("encrypt(data)", "from __main__ import encrypt, data, json", number=10000))
print(timeit("json.dumps(data)", "from __main__ import encrypt, data, json", number=10000))
print("DECRYPT JSON")
print(timeit("decrypt(py_data)", "from __main__ import decrypt, py_data, json, json_data", number=10000))
print(timeit("json.loads(json_data)", "from __main__ import decrypt, py_data, json, json_data", number=10000))
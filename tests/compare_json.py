import json
from timeit import timeit

from pd_proto import encrypt, decrypt

data = {
    "text": "Тестовая строка UTF-8",
    "text_ascii": "some text",
    "integer": 42,
    "float_coords": (55.7558, 37.6173),
    "boolean_true": True,
    "boolean_false": False,
    "none_value": None,
    "unique_tags": ["apple", "banana", "cherry"],
    "list_of_ints": [-1234124, 0, 123, 999, 123321445],
    "nested_dict": {"key": -3.14},
}
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
import json
from timeit import timeit

from pd_proto import loads, dumps

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
py_data = dumps(data)
json_data = json.dumps(data)
result =  100 - (len(py_data) / (len(json_data) / 100))
print(f"Size difference: {len(py_data) - len(json_data)} byte, {result:.2f}% better")

print("dumps JSON")
print(timeit("dumps(data)", "from __main__ import dumps, data, json", number=10000))
print(timeit("json.dumps(data)", "from __main__ import dumps, data, json", number=10000))
print("DECRYPT JSON")
print(timeit("loads(py_data)", "from __main__ import loads, py_data, json, json_data", number=10000))
print(timeit("json.loads(json_data)", "from __main__ import loads, py_data, json, json_data", number=10000))
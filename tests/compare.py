import pickle
from datetime import timedelta, timezone, datetime
from timeit import timeit
from zoneinfo import ZoneInfo

from pd_proto import encrypt, decrypt

data = {
    "text": "Тестовая строка UTF-8",
    "text_ascii": "some text",
    "integer": 42,
    "float_coords": (55.7558, 37.6173),
    "boolean_true": True,
    "boolean_false": False,
    "none_value": None,
    "unique_tags": {"apple", "banana", "cherry"},
    "list_of_ints": [-1234124, 0, 123, 999, 123321445],
    "nested_dict": {"key": -3.14},
    "datetime_naive": datetime(2026, 9, 8, 21, 12, 0),
    "datetime_aware": datetime(2026, 9, 8, 21, 12, 0, tzinfo=ZoneInfo("Europe/Moscow")),
    "datetime_offset": datetime(2026, 9, 8, 21, 12, 0, tzinfo=timezone(timedelta(hours=2))),
    # "bytes_data": b"\x00\x01\x02\x03"
}
py_data = encrypt(data)
pickle_data = pickle.dumps(data)
result =  100 - (len(py_data) / (len(pickle_data) / 100))
print(f"Size difference: {len(py_data) - len(pickle_data)} byte, {result:.2f}% better")

print("ENCRYPT")
print(timeit("encrypt(data)", "from __main__ import encrypt, data, pickle", number=10000))
print(timeit("pickle.dumps(data)", "from __main__ import encrypt, data, pickle", number=10000))
print("DECRYPT")
print(timeit("decrypt(py_data)", "from __main__ import decrypt, py_data, pickle, pickle_data", number=10000))
print(timeit("pickle.loads(pickle_data)", "from __main__ import decrypt, py_data, pickle, pickle_data", number=10000))

# At clean Python
# Size difference: -35 byte, 44.87% better
# ENCRYPT
# 14.861491874995409
# 0.8831491669989191
# DECRYPT
# 33.93260312502389
# 0.8927419999963604

# on Rust
# ENCRYPT (MacOS 25% faster)
# 0.006887415947858244
# 0.009223082975950092

# ENCRYPT (Windows 10, 25% faster)
# 0.008552699997380842
# 0.011504300000524381

# ENCRYPT (Debian same speed~)
# 0.012550916999771289
# 0.008764723000240338
# -----------
# 0.01072115599981771
# 0.01356410300013522

# on Rust (Windows 10, faster)
# DECRYPT
# 0.03864210000028834
# 0.04446980000648182
import pickle
from timeit import timeit

from pd_proto import encrypt, decrypt

data = ["text", [12569, (1.234, 4.5678), {1: [{1, 2}, {3, 4}]}], None, True, False]
py_data = encrypt(data)
pickle_data = pickle.dumps(data)
result =  100 - (len(py_data) / (len(pickle_data) / 100))
print(f"Size difference: {len(py_data) - len(pickle_data)} byte, {result:.2f}% better")

print("ENCRYPT")
print(timeit("encrypt(data)", "from __main__ import encrypt, data, pickle", number=10000))
print(timeit("pickle.dumps(data)", "from __main__ import encrypt, data, pickle", number=10000))
# print("DECRYPT")
# print(timeit("decrypt(py_data)", "from __main__ import decrypt, py_data, pickle, pickle_data"))
# print(timeit("pickle.loads(pickle_data)", "from __main__ import decrypt, py_data, pickle, pickle_data"))

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

# ENCRYPT (Debian same speed~)
# 0.012550916999771289
# 0.008764723000240338
# -----------
# 0.01072115599981771
# 0.01356410300013522
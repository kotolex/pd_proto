import pickle
from timeit import timeit

from pydata import decrypt, encrypt

data = [[12569, (1.234, 4.5678), {1: [{1, 2}, {3, 4}]}], None, True, False]
py_data = encrypt(data)
pickle_data = pickle.dumps(data)
result =  100 - (len(py_data) / (len(pickle_data) / 100))
print(f"Size difference: {len(py_data) - len(pickle_data)} byte, {result:.2f}% better")

print("ENCRYPT")
print(timeit("encrypt(data)", "from __main__ import encrypt, data, pickle"))
print(timeit("pickle.dumps(data)", "from __main__ import encrypt, data, pickle"))
print("DECRYPT")
print(timeit("decrypt(py_data)", "from __main__ import decrypt, py_data, pickle, pickle_data"))
print(timeit("pickle.loads(pickle_data)", "from __main__ import decrypt, py_data, pickle, pickle_data"))

# Size difference: -22 byte, 30.99% better
# ENCRYPT
# 14.861491874995409
# 0.8831491669989191
# DECRYPT
# 33.93260312502389
# 0.8927419999963604

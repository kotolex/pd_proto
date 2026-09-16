import math
import pickle
import random
import tempfile
from datetime import datetime, timezone, timedelta
from string import ascii_letters, digits, ascii_lowercase
from unittest import TestCase, main
from zoneinfo import ZoneInfo, available_timezones

from pd_proto.load import loads, load
from pd_proto.dump import dumps, dump

test_floats = (
    111.408802, 275.074313, 139.61, 676.7, 87.02144, 31.8763, 218.7, 601.99833,
    198.89788, 701.284709, 220.4965, 278.23507, 758.8, 698.1, 277.916, 957.12,
    102.29, 96.797, 603.705, 729.7, 124.9012, 78.8844, 829.339, 885.37466,
    577.337, 69.641238, 227.952696, 985.124, 866.41, 278.018, 834.043605,
    370.21, 670.14, 936.567257, 71.488229, 171.20442, 244.861963, 379.4796,
    688.124326, 684.58, 229.1, 805.0, 267.7873, 913.05, 876.29235, 212.684,
    395.6528, 458.860082, 139.702, 561.355861, 0.0, 0.1245, 1234.1, 3.141592,
    12349.0, 12.13020, -395.6528, -458.860082, -139.702, -561.355861, -0.1245, -1234.1, -3.141592,
)


def get_random_series(length: int) -> str:
    return "".join(random.choices(list(ascii_lowercase), k=length))


class TestUseCases(TestCase):
    def test_works_both_way(self):
        params = (
            {},
            [],
            "ЯЙË text",
            datetime.now(timezone.utc),
            datetime(2026, 9, 8, 21, 12, 0),
            {1, 2, None},
            [1000, 3.14, 3],
            [1000, -3.14, -3],
            [0, 0.0, True, False, None, [], tuple(), set(), {}],
            [1000, 3.14, [("1", "2"), {10, 121}]],
            [[12569, (1.234, 4.5678), {1: [{1, 2}, {3, 4}]}]],
            [{1: 1, 2: 2}, {3: {4: 4}}],
            [(ascii_letters + digits) * 3, 1234567890],
            9_223_372_036_854_775_807,
            -9_223_372_036_854_775_808,
        )
        for param in params:
            with self.subTest(f"test decrypt=encrypt ({param})"):
                self.assertEqual(loads(dumps(param)), param)

    def test_floats_more(self):
        for param in test_floats:
            with self.subTest(f"test floats ({param})"):
                res = dumps(param)
                self.assertEqual(loads(res), param)

    def test_diff_with_pickle(self):
        data = [[12569, (1.234, 4.5678), {1: [{1, 2}, {3, 4}]}], None, True, False]
        py_data = dumps(data)
        pickle_data = pickle.dumps(data)
        result = 100 - (len(py_data) / (len(pickle_data) / 100))
        self.assertGreater(result, 30)

    def test_diff_with_pickle_big_nesting(self):
        num = 0
        prev = {"name": "first", "age": num, "inner": []}
        for _ in range(100):
            num += 1
            prev = {"name": get_random_series(10), "level": num, "inner": [prev]}
        py_data = dumps(prev)
        pickle_data = pickle.dumps(prev)
        result = 100 - (len(py_data) / (len(pickle_data) / 100))
        self.assertGreater(result, 20)

    def test_floats(self):
        params = (
            3.14,
            15.0,
            12345.1,
            1234.123,
            567890.2345,
            5690.2345,
            123.12345,
            1.765432,
            -3.14,
            -15.0,
            -12345.1,
            -1234.123,
            -567890.2345,
            -5690.2345,
            -123.12345,
            -1.765432,
            275.074313,
            float("inf"),
            float("-inf"),
        )
        for param in params:
            with self.subTest(f"test opt floats ({param})"):
                res = dumps(param)
                self.assertEqual(loads(res), param)

    def test_strings(self):
        for param in ["a" * i for i in range(1, 16)]:
            with self.subTest(f"test opt strings ({param})"):
                res = dumps(param)
                self.assertEqual(loads(res), param)

    def test_empty_bytes(self):
        value = b''
        crypted = dumps(value)
        back = loads(crypted)
        self.assertEqual(value, back)

    def test_bytes(self):
        value = b'\x01\x13\x02\x01\x12'
        crypted = dumps(value)
        back = loads(crypted)
        self.assertEqual(value, back)

    def test_bytes_in_list(self):
        value = [b'1', b'', b'2']
        crypted = dumps(value)
        back = loads(crypted)
        self.assertEqual(value, back)

    def test_dt_no_tz(self):
        value = datetime.now()
        crypted = dumps(value)
        back = loads(crypted)
        self.assertEqual(value, back)

    def test_dt_offset(self):
        value = datetime.now(timezone.utc)
        crypted = dumps(value)
        back = loads(crypted)
        self.assertEqual(value, back)

    def test_dt_iana(self):
        value = datetime.now(tz=ZoneInfo("Europe/London"))
        crypted = dumps(value)
        back = loads(crypted)
        self.assertEqual(value, back)

    def test_tz_in_list(self):
        value = datetime.now()
        value2 = datetime.now(timezone(timedelta(hours=-2)))
        value3 = datetime.now(tz=ZoneInfo("Europe/London"))
        a_list = [value, value2, value3, value, value2, value3]
        crypted = dumps(a_list)
        back = loads(crypted)
        self.assertEqual(a_list, back)

    def test_all_timezones(self):
        for tz in available_timezones():
            with self.subTest(f"test timezone {tz}"):
                value = datetime.now(tz=ZoneInfo(tz))
                crypted = dumps(value)
                back = loads(crypted)
                self.assertEqual(value, back)

    def test_all_offsets(self):
        for tz in range(-12, 15):
            with self.subTest(f"test timezone offset {tz}"):
                value = datetime.now(tz=timezone(timedelta(hours=tz)))
                crypted = dumps(value)
                back = loads(crypted)
                self.assertEqual(value, back)

    def test_float_nan(self):
        value = float("nan")
        crypted = dumps(value)
        back = loads(crypted)
        self.assertTrue(math.isnan(back))

    def test_1000(self):
        params = (
            [e for e in range(1001)],
            [[1, 2] for _ in range(1001)],
            tuple(e for e in range(1001)),
            set(e for e in range(1001)),
            {e: str(e) for e in range(1001)},
            (ascii_letters * 20)[:1001],
        )
        for param in params:
            with self.subTest(f"test 1000-element collections ({type(param)})"):
                res = dumps(param)
                self.assertEqual(loads(res), param)

    def test_cache_ints(self):
        data = b"\x01X\x0b\xe8\x84\x01\n\xf0\xab\x01\x0b\x01='\x00'\x01\x0b\x01="
        result = loads(data)
        self.assertEqual(result, [-17000, 22000, -1, 1, -17000, 22000, -1, 1])

    def test_dump_file(self):
        data = {1: 1, "2": "2", 3: 3.14, 4: [1, 2, 3]}
        with tempfile.NamedTemporaryFile() as tmp:
            dump(tmp, data)
            tmp.seek(0)
            read_data = load(tmp)
        self.assertEqual(read_data, data)


if __name__ == '__main__':
    main()

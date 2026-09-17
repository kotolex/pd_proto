import random
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from unittest import TestCase, main

from pd_proto import encode_float, encode_varint, decode_varint, loads, BinaryFileError
from pd_proto.dump import dumps, dump
from pd_proto.errors import CycleLinksError, UnsupportedTypeError, IntegerOutOfBoundsError


class TestDumps(TestCase):

    def test_encode_varint(self):
        self.assertEqual(b'\x7f', bytes(encode_varint(127)))
        self.assertEqual(b'\xe8\x07', bytes(encode_varint(1000)))

    def test_decode_varint(self):
        self.assertEqual((0, 1), decode_varint(b'\x00', 0))
        self.assertEqual((127, 1), decode_varint(b'\x7f', 0))
        self.assertEqual((1000, 2), decode_varint(b'\xe8\x07', 0))

    def test_work_varint_many(self):
        for i in (42, 555, 100, 1000, 1000000):
            self.assertEqual(i, decode_varint(bytes(encode_varint(i)), 0)[0])

    def test_work_varint_random(self):
        for i in (random.randint(0, 1000000) for _ in range(1000)):
            self.assertEqual(i, decode_varint(bytes(encode_varint(i)), 0)[0])

    def test_dumps(self):
        params = (
            (b'\x01\x00', None),
            (b'\x01\x01', True),
            (b'\x01\x02', False),
            (b'\x01\x04', ""),
            (b'\x01\x12', b''),
            (b'\x01\t', 0),
            (b'\x01\x0bd', -100),
            (b'\x01\x0b\xff\xff\xff\xff\xff\xff\xff\xff\x7f', -9223372036854775807),
            (b'\x01\n\xff\xff\xff\xff\xff\xff\xff\xff\x7f', 9_223_372_036_854_775_807),
            (b'\x01\x16\x80\x02', 2.56),
            (b'\x01 \x80\x02', -2.56),
            (b'\x01\x03', 0.0),
            (b'\x01,test', "test"),
            (b'\x01\x13\x02\x01\x12', b'\x01\x12'),
            (b'\x01\x05', []),
            (b'\x01Q\x01', [True]),
            (b'\x01S=>R)a)b', [1, 2, ["a", "b"]]),
            (b'\x01S=>R=>', [1, 2, [1, 2]]),
            (b'\x01\x06', tuple()),
            (b'\x019\t\x03\x00', (0, 0.0, None)),
            (b'\x01\x07', set()),
            (b'\x01\x10\x02\t\x00', {0, None}),
            (b'\x01\x08', {}),
            (b'\x01\x11\x02=>?@', {1: 2, 3: 4}),
            (b'\x01\x1c\x0cA\xda\xa8\x1e\x88\x00\x00\x00\t', datetime(2026, 9, 8, 21, 12, 0, tzinfo=timezone.utc)),
            (b'\x01=', 1),
            (b'\x01>', 2),
            (b'\x01?', 3),
            (b'\x01@', 4),
            (b'\x01A', 5),
            (b'\x01B', 6),
            (b'\x01C', 7),
            (b'\x01D', 8),
            (b'\x01E', 9),
            (b'\x01F', 10),
            (b'\x01G', 11),
            (b'\x01H', 12),
            (b'\x01I', 13),
            (b'\x01J', 15),
            (b'\x01K', 20),
            (b'\x01L', 24),
            (b'\x01M', 50),
            (b'\x01N', 100),
            (b'\x01<', 1000),
            (b'\x018=>', (1, 2)),
            (b'\x019=>?', (1, 2, 3)),
            (b'\x01:=>?@', (1, 2, 3, 4)),
            (b'\x01;=>?@A', (1, 2, 3, 4, 5)),
            (b'\x01Q=', [1, ]),
            (b'\x01R=>', [1, 2]),
            (b'\x01S=>?', [1, 2, 3]),
            (b'\x01T=>?@', [1, 2, 3, 4]),
            (b'\x01U=>?@A', [1, 2, 3, 4, 5]),
            (b'\x01V=>?@AB', [1, 2, 3, 4, 5, 6]),
            (b'\x01W=>?@ABC', [1, 2, 3, 4, 5, 6, 7]),
            (b'\x01X=>?@ABCD', [1, 2, 3, 4, 5, 6, 7, 8, ]),
            (b'\x01Y=>?@ABCDE', [1, 2, 3, 4, 5, 6, 7, 8, 9, ]),
            (b'\x01Z=>?@ABCDEF', [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        )
        for expected, arg in params:
            with self.subTest(f"dumps({arg})"):
                self.assertEqual(expected, dumps(arg))

    def test_encode_float(self):
        self.assertEqual(bytearray(b'\x16\xba\x02'), encode_float(3.14))
        self.assertEqual(bytearray(b' \xba\x02'), encode_float(-3.14))
        self.assertEqual(bytearray(b'\x03'), encode_float(0.0))

    def test_dumps_raise_on_unsupported_type(self):
        with self.assertRaises(UnsupportedTypeError):
            dumps(self)

    def test_dumps_raise_on_recursion(self):
        a_l = [1, 2]
        a_l.append(a_l)
        with self.assertRaises(CycleLinksError):
            dumps(a_l)

    def test_dumps_raise_on_too_big_int(self):
        with self.assertRaises(IntegerOutOfBoundsError):
            dumps(9_223_372_036_854_775_810)

    def test_dumps_raise_on_too_small_int(self):
        with self.assertRaises(IntegerOutOfBoundsError):
            dumps(-9_223_372_036_854_775_810)

    def test_dump_file(self):
        data = {1: 1, "2": "2", 3: 3.14, 4: [1, 2, 3]}
        with tempfile.NamedTemporaryFile() as tmp:
            dump(tmp, data)
            tmp.seek(0)
            read_data = tmp.read()
        self.assertEqual(loads(read_data), data)

    def test_dump_raise_not_a_file(self):
        with self.assertRaises(BinaryFileError):
            dump(self, [])

    def test_dump_raise_not_a_binary(self):
        with self.assertRaises(BinaryFileError):
            with open(Path(__file__).parent / "compare.py") as file:
                dump(file, [])

    def test_dump_raise_not_for_write(self):
        with self.assertRaises(BinaryFileError):
            with open(Path(__file__).parent / "compare.py", "rb") as file:
                dump(file, [])


if __name__ == '__main__':
    main()

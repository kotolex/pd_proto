import random
from unittest import TestCase, main

from src.pydata.decrypt import decode_varint
from src.pydata.encrypt import encode_float, encode_varint, encrypt


class TestEncodeEncrypt(TestCase):

    def test_encode_varint(self):
        self.assertEqual(b'\x00', bytes(encode_varint(0)))
        self.assertEqual(b'\x7f', bytes(encode_varint(127)))
        self.assertEqual(b'\xe8\x07', bytes(encode_varint(1000)))

    def test_decode_varint(self):
        self.assertEqual((0, 1), decode_varint(b'\x00'))
        self.assertEqual((127, 1), decode_varint(b'\x7f'))
        self.assertEqual((1000, 2), decode_varint(b'\xe8\x07'))

    def test_work_varint_many(self):
        for i in (0, 42, 555, 100, 1000, 1000000):
            self.assertEqual(i, decode_varint(bytes(encode_varint(i)))[0])

    def test_work_varint_random(self):
        for i in (random.randint(0, 1000000) for _ in range(1000)):
            self.assertEqual(i, decode_varint(bytes(encode_varint(i)))[0])

    def test_encrypt(self):
        params = (
            (b'\x01\x00', None),
            (b'\x01\x01', True),
            (b'\x01\x02', False),
            (b'\x01\x04', ""),
            (b'\x01\nd', 100),
            (b'\x01\x0bd', -100),
            (b'\x01\x0c@\x04z\xe1G\xae\x14{', 2.56),
            (b'\x01\x03', 0.0),
            (b'\x01\r\x04test', "test"),
        )
        for expected, arg in params:
            with self.subTest(f"encrypt{arg}"):
                self.assertEqual(expected, encrypt(arg))

    def test_encode_float(self):
        self.assertEqual(b'@\t\x1e\xb8Q\xeb\x85\x1f', encode_float(3.14))
        self.assertEqual(b'\x00\x00\x00\x00\x00\x00\x00\x00', encode_float(0.0))


if __name__ == '__main__':
    main()

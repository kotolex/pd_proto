import random
from unittest import TestCase, main
from src.pydata.encrypt import encode_varint, encrypt
from src.pydata.decrypt import decode_varint


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
        self.assertEqual(b'\x01\x00', encrypt(None))
        self.assertEqual(b'\x01\x01', encrypt(True))
        self.assertEqual(b'\x01\x02', encrypt(False))
        self.assertEqual(b'\x01\x03d', encrypt(100))
        self.assertEqual(b'\x01\x04d', encrypt(-100))


if __name__ == '__main__':
    main()

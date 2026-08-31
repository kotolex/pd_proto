from unittest import TestCase, main

from src.pydata.decrypt import decrypt
from src.pydata.encrypt import encrypt
from src.pydata.errors import EmptyDataError, ProtocolError


class TestDecrypt(TestCase):

    def test_decrypt(self):
        self.assertEqual(-910, decrypt(encrypt(-910)))
        self.assertEqual(910, decrypt(encrypt(910)))
        self.assertEqual(None, decrypt(encrypt(None)))
        self.assertEqual(True, decrypt(encrypt(True)))
        self.assertEqual(False, decrypt(encrypt(False)))
        self.assertEqual("", decrypt(encrypt("")))
        self.assertEqual(0, decrypt(encrypt(0)))
        self.assertEqual([], decrypt(encrypt([])))
        # self.assertEqual({}, decrypt(encrypt({})))
        # self.assertEqual(tuple(), decrypt(encrypt(tuple())))
        # self.assertEqual(set(), decrypt(encrypt(set())))

    def test_decrypt_string(self):
        self.assertEqual("stop", decrypt(b'\x01\r\x04stop'))
        self.assertEqual("", decrypt(b'\x01\x04'))

    def test_decrypt_list(self):
        self.assertEqual([1], decrypt(b'\x01\x0e\x01\n\x01'))
        self.assertEqual([True], decrypt(b'\x01\x0e\x01\x01'))
        self.assertEqual([1, 2], decrypt(b'\x01\x0e\x02\n\x01\n\x02'))
        self.assertEqual([1, 2, [1, 2]], decrypt(b'\x01\x0e\x03\n\x01\n\x02\x0e\x02\n\x01\n\x02'))

    def test_decrypt_float(self):
        self.assertEqual(3.14, decrypt(b'\x01\x0c@\t\x1e\xb8Q\xeb\x85\x1f'))
        self.assertEqual(0.0, decrypt(b'\x01\x03'))

    def test_decrypt_fail_on_empty(self):
        with self.assertRaises(EmptyDataError):
            decrypt(b'')

    def test_decrypt_fail_on_empty2(self):
        with self.assertRaises(EmptyDataError):
            decrypt(b'1')

    def test_decrypt_fail_on_wrong_protocol(self):
        with self.assertRaises(ProtocolError):
            decrypt(b'\x02\x03')

    def test_decrypt_fail_on_corrupt_data(self):
        with self.assertRaises(ProtocolError):
            decrypt(b'\x01\x010101')


if __name__ == '__main__':
    main()

from unittest import TestCase, main
from src.pydata.decrypt import decrypt


class TestDecrypt(TestCase):

    def test_decrypt(self):
        self.assertEqual(-910, decrypt(b'\x01\x04\x8e\x07'))
        self.assertEqual(910, decrypt(b'\x01\x03\x8e\x07'))
        self.assertEqual(None, decrypt(b'\x01\x00'))
        self.assertEqual(True, decrypt(b'\x01\x01'))
        self.assertEqual(False, decrypt(b'\x01\x02'))


if __name__ == '__main__':
    main()

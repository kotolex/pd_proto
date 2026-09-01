import pickle
from unittest import TestCase, main

from src.pydata.encrypt import encrypt
from src.pydata.utils import exponent


class TestUtils(TestCase):
    def test_exponent(self):
        self.assertEqual(exponent(3.14), 2)
        self.assertEqual(exponent(2.0), 0)
        self.assertEqual(exponent(1.1234005), 7)

if __name__ == '__main__':
    main()
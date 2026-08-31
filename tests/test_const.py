from unittest import TestCase, main

from src.pydata.const import int_by_type, obj_by_code
from src.pydata.errors import UnsupportedTypeError, WrongCodeError


class TestConst(TestCase):

    def test_type_by_int_raise(self):
        with self.assertRaises(WrongCodeError):
            obj_by_code(-1)

    def test_int_by_type_raise(self):
        with self.assertRaises(UnsupportedTypeError):
            int_by_type(self)

    def test_int_by_type(self):
        params = (
            (0, None),
            (1, True),
            (2, False),
            (4, ""),
            (10, 100),
            (11, -100),
            (12, 2.56),
            (3, 0.0),
        )
        for expected, arg in params:
            with self.subTest(f"int_by_type{arg}"):
                self.assertEqual(expected, int_by_type(arg))



if __name__ == '__main__':
    main()

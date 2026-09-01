import pickle
from string import ascii_letters, digits
from unittest import TestCase, main

from src.pd_proto.decrypt import decrypt
from src.pd_proto.encrypt import encrypt


class TestUseCases(TestCase):
    def test_works_both_way(self):
        params = (
            {},
            [],
            "ЯЙË text",
            {1, 2, None},
            [1000, 3.14, 3],
            [0, 0.0, True, False, None, [], tuple(), set(), {}],
            [1000, 3.14, [("1", "2"), {10, 121}]],
            [[12569, (1.234, 4.5678), {1: [{1, 2}, {3, 4}]}]],
            [{1: 1, 2: 2}, {3: {4: 4}}],
            [(ascii_letters+digits)*3, 1234567890],
        )
        for param in params:
            with self.subTest(f"test decrypt=encrypt ({param})"):
                self.assertEqual(decrypt(encrypt(param)), param)

    def test_diff_with_pickle(self):
        data = [[12569, (1.234, 4.5678), {1: [{1, 2}, {3, 4}]}], None, True, False]
        py_data = encrypt(data)
        pickle_data = pickle.dumps(data)
        result = 100 - (len(py_data) / (len(pickle_data) / 100))
        self.assertGreater(result, 30)

    def test_floats(self):
        params = (
            3.14,
            15.0,
            12345.1,
            1234.123,
            567890.2345,
            123.12345,
            1.765432,
        )
        for param in params:
            with self.subTest(f"test opt floats ({param})"):
                res = encrypt(param)
                self.assertEqual(decrypt(res), param)

    def test_strings(self):
        for param in ["a"* i for i in range(1,16)]:
            with self.subTest(f"test opt strings ({param})"):
                res = encrypt(param)
                self.assertEqual(decrypt(res), param)


if __name__ == '__main__':
    main()

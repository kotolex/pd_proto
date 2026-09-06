import pickle
from string import ascii_letters, digits
from unittest import TestCase, main

from pd_proto import encrypt
from pd_proto.decrypt import decrypt

test_floats = (
    111.408802, 275.074313, 139.61, 676.7, 87.02144, 31.8763, 218.7, 601.99833,
    198.89788, 701.284709, 220.4965, 278.23507, 758.8, 698.1, 277.916, 957.12,
    102.29, 96.797, 603.705, 729.7, 124.9012, 78.8844, 829.339, 885.37466,
    577.337, 69.641238, 227.952696, 985.124, 866.41, 278.018, 834.043605,
    370.21, 670.14, 936.567257, 71.488229, 171.20442, 244.861963, 379.4796,
    688.124326, 684.58, 229.1, 805.0, 267.7873, 913.05, 876.29235, 212.684,
    395.6528, 458.860082, 139.702, 561.355861, 0.0, 0.1245, 1234.1, 3.141592,
    12349.0, 12.13020
)


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
            [(ascii_letters + digits) * 3, 1234567890],
        )
        for param in params:
            with self.subTest(f"test decrypt=encrypt ({param})"):
                self.assertEqual(decrypt(encrypt(param)), param)

    def test_floats_more(self):
        for param in test_floats:
            with self.subTest(f"test floats ({param})"):
                res = encrypt(param)
                self.assertEqual(decrypt(res), param)

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
            5690.2345,
            123.12345,
            1.765432,
        )
        for param in params:
            with self.subTest(f"test opt floats ({param})"):
                res = encrypt(param)
                self.assertEqual(decrypt(res), param)

    def test_strings(self):
        for param in ["a" * i for i in range(1, 16)]:
            with self.subTest(f"test opt strings ({param})"):
                res = encrypt(param)
                self.assertEqual(decrypt(res), param)


if __name__ == '__main__':
    main()

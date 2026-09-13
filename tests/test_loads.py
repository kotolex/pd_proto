from unittest import TestCase, main

from pd_proto.load import loads
from pd_proto.dump import dumps
from pd_proto.errors import (BytesLeftError, ParseFloatError, DataCorruptionError,
                             ParseStringError, EmptyDataError,
                             ProtocolError)


class TestLoads(TestCase):

    def test_loads(self):
        self.assertEqual(-910, loads(dumps(-910)))
        self.assertEqual(910, loads(dumps(910)))
        self.assertEqual(-91.02, loads(dumps(-91.02)))
        self.assertEqual(91.0434, loads(dumps(91.0434)))
        self.assertEqual(None, loads(dumps(None)))
        self.assertEqual(True, loads(dumps(True)))
        self.assertEqual(False, loads(dumps(False)))
        self.assertEqual("", loads(dumps("")))
        self.assertEqual(0, loads(dumps(0)))
        self.assertEqual([], loads(dumps([])))
        self.assertEqual({}, loads(dumps({})))
        self.assertEqual(tuple(), loads(dumps(tuple())))
        self.assertEqual(set(), loads(dumps(set())))

    def test_loads_string(self):
        self.assertEqual("stop", loads(b'\x01\r\x04stop'))
        self.assertEqual("", loads(b'\x01\x04'))

    def test_loads_list(self):
        self.assertEqual([1], loads(b'\x01\x0e\x01\n\x01'))
        self.assertEqual([True], loads(b'\x01\x0e\x01\x01'))
        self.assertEqual([1, 2], loads(b'\x01\x0e\x02\n\x01\n\x02'))
        self.assertEqual([1, 2, [1, 2]], loads(b'\x01\x0e\x03\n\x01\n\x02\x0e\x02\n\x01\n\x02'))

    def test_loads_float(self):
        self.assertEqual(3.14, loads(b'\x01\x0c@\t\x1e\xb8Q\xeb\x85\x1f'))
        self.assertEqual(-3.14, loads(b'\x01 \xba\x02'))
        self.assertEqual(0.0, loads(b'\x01\x03'))

    def test_loads_fail_on_empty(self):
        with self.assertRaises(EmptyDataError):
            loads(b'')

    def test_loads_fail_on_empty2(self):
        with self.assertRaises(EmptyDataError):
            loads(b'1')

    def test_loads_fail_on_wrong_protocol(self):
        with self.assertRaises(ProtocolError):
            loads(b'\x02\x03')

    def test_loads_fail_on_corrupt_data(self):
        with self.assertRaises(BytesLeftError):
            loads(b'\x01\x010101')

    def test_loads_list_full(self):
        params = (
            ([1, -1, 0, 1], b'\x01\x0e\x04\n\x01\x0b\x01\t\n\x01'),
            ([1, 2, [1, 2], 1, 2], b'\x01\x0e\x05\n\x01\n\x02\x0e\x02\n\x01\n\x02\n\x01\n\x02'),
            (['1', [], '2', ['1'], '3'], b'\x01\x0e\x05\r\x011\x05\r\x012\x0e\x01\r\x011\r\x013'),
            ([1.23, 0.0, 3.14], b'\x01\x0e\x03\x0c?\xf3\xae\x14z\xe1G\xae\x03\x0c@\t\x1e\xb8Q\xeb\x85\x1f'),
            ([None, False, True, [], 0, 0.0, ''], b'\x01\x0e\x07\x00\x02\x01\x05\t\x03\x04'),
            ([(1, 2), 3, {'4', '5'}, 6.789], b"\x01\x0e\x04\x0f\x02\n\x01\n\x02\n\x03\x10\x02\r\x014\r\x015\x0c@\x1b'\xef\x9d\xb2-\x0e"),
            ([100, 10.1, [100, 10.1, [100, 10.1]]], b'\x01\x0e\x03\nd\x0c@$333333\x0e\x03\nd\x0c@$333333\x0e\x02\nd\x0c@$333333'),
        )
        for expected, arg in params:
            with self.subTest(f"loads_list({arg})"):
                self.assertEqual(expected, loads(arg))

    def test_loads_fail_on_corrupt_float(self):
        with self.assertRaises(ParseFloatError):
            loads(b'\x01\x0c@$33333')

    def test_loads_fail_on_corrupt_string(self):
        with self.assertRaises(ParseStringError):
            loads(b'\x01\r\x031')

    def test_loads_tuple(self):
        params = (
            ((1, 2, None), b'\x01\x0f\x03\n\x01\n\x02\x00'),
            ((0, 0.0, None, ''), b'\x01\x0f\x04\t\x03\x00\x04'),
            ((1, (1.0, ()), '1'), b'\x01\x0f\x03\n\x01\x0f\x02\x0c?\xf0\x00\x00\x00\x00\x00\x00\x06\r\x011'),
        )
        for expected, arg in params:
            with self.subTest(f"loads_list({arg})"):
                self.assertEqual(expected, loads(arg))


    def test_loads_set(self):
        params = (
            ({0, 1, None}, b'\x01\x10\x03\t\n\x01\x00'),
            ({0.12, '', 100}, b'\x01\x10\x03\x0c?\xbe\xb8Q\xeb\x85\x1e\xb8\x04\nd'),
            ({(1, 2), 3}, b'\x01\x10\x02\n\x03\x0f\x02\n\x01\n\x02'),
        )
        for expected, arg in params:
            with self.subTest(f"loads_list({arg})"):
                self.assertEqual(expected, loads(arg))

    def test_loads_fail_list_end_stream(self):
        with self.assertRaises(DataCorruptionError):
            loads(b'\x01\x0e\x03\n\x01\n\x02')

    def test_loads_fail_tuple_end_stream(self):
        with self.assertRaises(DataCorruptionError):
            loads(b'\x01\x0f\x02\n\x01\n')

    def test_loads_fail_string_end_stream(self):
        with self.assertRaises(ParseStringError):
            loads(b'\x01*\xd1')

    def test_loads_fail_varint(self):
        with self.assertRaises(DataCorruptionError):
            loads(b'\x01\n' + b'\x80'*20)

    def test_loads_fail_no_cache_index(self):
        with self.assertRaises(DataCorruptionError):
            loads(b"\x01R\n\xf0\xab\x01'")

    def test_loads_fail_nothing_in_cache(self):
        with self.assertRaises(DataCorruptionError):
            loads(b"\x01R\n\xf0\xab\x01'\x05")

    def test_loads_fail_no_element_for_dict(self):
        with self.assertRaises(DataCorruptionError):
            loads(b'\x01\x11\x02==>')

    # def test_loads_fail_not_enough_bytes(self):
    #     with self.assertRaises(ValueError):
    #         loads(b'\x01\x13\x051234')


if __name__ == '__main__':
    main()

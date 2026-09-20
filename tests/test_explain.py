import tempfile
from pathlib import Path
from unittest import TestCase, main

from pd_proto.load import explains, explain
from pd_proto.dump import dump


class TestExplain(TestCase):

    def test_extra_bytes(self):
        expected ="""Parsing started at offset 1, total length 3
-------------- [Offset 1] [Tag 0 / 0x0] [Nesting level 1]---------------
None object parsed.
Parsing stopped at offset 2
[ERROR] Corrupt data, finished at offset 2, but still have 1 bytes unparsed, total data length 3
"""
        result = explains(b'\x01\x00\x00')
        self.assertEqual(expected, result)

    def test_empty_cache(self):
        expected = """Parsing started at offset 1, total length 3
-------------- [Offset 1] [Tag 38 / 0x26] [Nesting level 1]---------------
Float cache tag found; expecting a 1-byte cache index at offset 2
Float cache requested index 0
[ERROR] Parsing stopped: Unexpected float cache failure - no data at index 0
"""
        result = explains(b'\x01&\x00')
        self.assertEqual(expected, result)

    def test_no_cache_index(self):
        expected = """Parsing started at offset 1, total length 2
-------------- [Offset 1] [Tag 38 / 0x26] [Nesting level 1]---------------
Float cache tag found; expecting a 1-byte cache index at offset 2
[ERROR] Parsing stopped: Expect float cache index, but nothing to read at offset 2
"""
        result = explains(b'\x01&')
        self.assertEqual(expected, result)

    def test_no_tag(self):
        expected = """Parsing started at offset 1, total length 2
-------------- [Offset 1] [Tag 121 / 0x79] [Nesting level 1]---------------
Unknown tag 121
[ERROR] Parsing stopped: Unknown tag 121
"""
        result = explains(b'\x01y')
        self.assertEqual(expected, result)

    def test_var_int_no_data(self):
        expected = """Parsing started at offset 1, total length 4
-------------- [Offset 1] [Tag 10 / 0xA] [Nesting level 1]---------------
Positive integer tag found, attempting to parse it
Parsing of a var_int started at offset 2
Var_int parsing, offset 2
Var_int parsing, offset 3
Var_int parsing, offset 4
[ERROR] Parsing stopped: While parsing var_int - no data to read at offset 4
"""
        result = explains(b'\x01\n\xa2\xb2')
        self.assertEqual(expected, result)

    def test_var_int_too_big(self):
        expected = """Parsing started at offset 1, total length 16
-------------- [Offset 1] [Tag 10 / 0xA] [Nesting level 1]---------------
Positive integer tag found, attempting to parse it
Parsing of a var_int started at offset 2
Var_int parsing, offset 2
Var_int parsing, offset 3
Var_int parsing, offset 4
Var_int parsing, offset 5
Var_int parsing, offset 6
Var_int parsing, offset 7
Var_int parsing, offset 8
Var_int parsing, offset 9
Var_int parsing, offset 10
Var_int parsing, offset 11
[ERROR] Parsing stopped: While parsing var_int - int is too long or data corrupted at offset: 12
"""
        result = explains(b'\x01\n\xa2\xb2\xa2\xb2\xa2\xb2\xa2\xb2\xa2\xb2\xa2\xb2\xa2\xb2')
        self.assertEqual(expected, result)

    def test_float_not_enough_bytes(self):
        expected = """Parsing started at offset 1, total length 9
-------------- [Offset 1] [Tag 12 / 0xC] [Nesting level 1]---------------
Float tag found, attempting to parse it
Parsing of an 8-byte float started at offset 2
[ERROR] Parsing stopped: Not enough bytes, need 8, but have only 7 bytes left at offset 2
"""
        result = explains(b'\x01\x0c\x7f\xf8\x00\x00\x00\x00\x00')
        self.assertEqual(expected, result)

    def test_string_not_enough_bytes(self):
        expected = """Parsing started at offset 1, total length 18
-------------- [Offset 1] [Tag 13 / 0xD] [Nesting level 1]---------------
String tag found, attempting to parse it
Parsing of a string started at offset 2
Expected integer (string size) at offset 2
Parsing of a var_int started at offset 2
Var_int parsing, offset 2
Var_int parsed 16, 1 bytes read
Expected string size: 16
[ERROR] Parsing stopped: Not enough bytes, need 16, but have only 15 bytes left at offset 3
"""
        result = explains(b'\x01\r\x10123456789012345')
        self.assertEqual(expected, result)

    def test_opt_string_not_enough_bytes(self):
        expected = """Parsing started at offset 1, total length 6
-------------- [Offset 1] [Tag 45 / 0x2D] [Nesting level 1]---------------
String tag found, attempting to parse it
Parsing of a string started at offset 2
It is an optimized string, no need to parse its size; expected string size is 5
[ERROR] Parsing stopped: Not enough bytes, need 5, but have only 4 bytes left
"""
        result = explains(b'\x01-1234')
        self.assertEqual(expected, result)

    def test_list_not_enough_elements(self):
        expected = """Parsing started at offset 1, total length 2
-------------- [Offset 1] [Tag 81 / 0x51] [Nesting level 1]---------------
List tag found, attempting to parse it
	It is an optimized list, no need to parse its size
	List expects 1 elements
	Processed List element 0
No data at offset 2
[ERROR] Parsing stopped: No data at offset 2
"""
        result = explains(b'\x01Q')
        self.assertEqual(expected, result)

    def test_dict_no_value(self):
        expected = """Parsing started at offset 1, total length 4
-------------- [Offset 1] [Tag 17 / 0x11] [Nesting level 1]---------------
Dict tag found, attempting to parse it
	Expect dict size, parse int at 2
	Parsing of a var_int started at offset 2
	Var_int parsing, offset 2
	Var_int parsed 1, 1 bytes read
	Dict expects 1 elements(pairs)
	Start parsing 0 element (pair) for dict
-------------- [Offset 3] [Tag 61 / 0x3D] [Nesting level 2]---------------
	Optimized int tag found, attempting to parse it
	Integer=1 parsed
No data at offset 4
[ERROR] Parsing stopped: No data at offset 4
"""
        result = explains(b'\x01\x11\x01=')
        self.assertEqual(expected, result)

    def test_bytes_not_enough_size(self):
        expected = """Parsing started at offset 1, total length 5
-------------- [Offset 1] [Tag 19 / 0x13] [Nesting level 1]---------------
Bytes tag found, attempting to parse it
Expect bytes size, parse int at 2
Parsing of a var_int started at offset 2
Var_int parsing, offset 2
Var_int parsed 3, 1 bytes read
Bytes block expect size 3
[ERROR] Parsing stopped: Not enough bytes, need 3, but have only 2 bytes left at offset 3
"""
        result = explains(b'\x01\x13\x0312')
        self.assertEqual(expected, result)

    def test_dt_naive_wrong_type(self):
        expected = """Parsing started at offset 1, total length 10
-------------- [Offset 1] [Tag 27 / 0x1B] [Nesting level 1]---------------
Datetime without timezone tag found. A float (timestamp) is now expected.
-------------- [Offset 2] [Tag 0 / 0x0] [Nesting level 1]---------------
None object parsed.
[ERROR] Parsing stopped: Unexpected type while parsing DateTime, expected Float
"""
        result = explains(b'\x01\x1b\x00\xda\xa8\x0c\xf4\x00\x00\x00')
        self.assertEqual(expected, result)

    def test_dt_offset_wrong_type(self):
        expected = """Parsing started at offset 1, total length 14
-------------- [Offset 1] [Tag 28 / 0x1C] [Nesting level 1]---------------
Datetime with offset tag found. A float (timestamp) is now expected.
-------------- [Offset 2] [Tag 12 / 0xC] [Nesting level 1]---------------
Float tag found, attempting to parse it
Parsing of an 8-byte float started at offset 3
An 8-byte float (1788894720) parsed; 8 bytes read
Float=1788894720 parsed, pushing to cache
Timestamp=1788894720 parsed. An integer (offset) is now expected.
-------------- [Offset 11] [Tag 0 / 0x0] [Nesting level 1]---------------
None object parsed.
[ERROR] Parsing stopped: Unexpected type while parsing DateTime, expected Int
"""
        result = explains(b'\x01\x1c\x0cA\xda\xa8\x17\x80\x00\x00\x00\x00\xa08')
        self.assertEqual(expected, result)

    def test_dt_iana_wrong_type(self):
        expected = """Parsing started at offset 1, total length 25
-------------- [Offset 1] [Tag 29 / 0x1D] [Nesting level 1]---------------
Datetime with IANA tag found. A float (timestamp) is now expected.
-------------- [Offset 2] [Tag 12 / 0xC] [Nesting level 1]---------------
Float tag found, attempting to parse it
Parsing of an 8-byte float started at offset 3
An 8-byte float (1788891120) parsed; 8 bytes read
Float=1788891120 parsed, pushing to cache
Timestamp=1788891120 parsed. A string (IANA) is now expected.
-------------- [Offset 11] [Tag 0 / 0x0] [Nesting level 1]---------------
None object parsed.
[ERROR] Parsing stopped: Unexpected type while parsing DateTimeIana, expected String
"""
        result = explains(b'\x01\x1d\x0cA\xda\xa8\x13\xfc\x00\x00\x00\x00Europe/Moscow')
        self.assertEqual(expected, result)

    def test_exceed_nesting_limit(self):
        expected = """Parsing started at offset 1, total length 9
-------------- [Offset 1] [Tag 82 / 0x52] [Nesting level 1]---------------
List tag found, attempting to parse it
	It is an optimized list, no need to parse its size
	List expects 2 elements
	Processed List element 0
-------------- [Offset 2] [Tag 82 / 0x52] [Nesting level 2]---------------
	List tag found, attempting to parse it
		It is an optimized list, no need to parse its size
		List expects 2 elements
		Processed List element 0
-------------- [Offset 3] [Tag 61 / 0x3D] [Nesting level 3]---------------
		Optimized int tag found, attempting to parse it
		Integer=1 parsed
		Processed List element 1
-------------- [Offset 4] [Tag 82 / 0x52] [Nesting level 3]---------------
		List tag found, attempting to parse it
			It is an optimized list, no need to parse its size
			List expects 2 elements
			Processed List element 0
Nesting depth 4 exceeded maximum of 3 at offset 5
[ERROR] Parsing stopped: Nesting depth 4 exceeded maximum of 3 at offset 5
"""
        result = explains(b'\x01RR=R=Q>?', max_depth=3)
        self.assertEqual(expected, result)

    def test_explain_file(self):
        data = True
        expected="""Parsing started at offset 1, total length 2
-------------- [Offset 1] [Tag 1 / 0x1] [Nesting level 1]---------------
Boolean True parsed.
Parsing stopped at offset 2
"""
        with tempfile.NamedTemporaryFile() as tmp:
            dump(tmp, data)
            tmp.seek(0)
            with open("some.txt", "wt", encoding="utf-8") as tmp2:
                explain(tmp, tmp2)
            read_data = (Path(__file__).parent / "some.txt").read_text()
        self.assertEqual(read_data, expected)

if __name__ == '__main__':
    main()
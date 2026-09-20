import tempfile
from pathlib import Path
from unittest import TestCase, main

from pd_proto.errors import BinaryFileError
from pd_proto.dump import dumps
from pd_proto.utils import checksums, checksum, check_binary_file_for_reading


class TestUtils(TestCase):
    def test_checksums(self):
        data = {1: 1, "2": "2", 3: 3.14, 4: [1, 2, 3]}
        self.assertEqual(444138351, checksums(dumps(data)))

    def test_checksum(self):
        fl = Path(__file__).parent / "first.raw"
        with open(fl, "rb") as file:
            crc = checksum(file)
        self.assertEqual(3170987896, crc)

    def test_raise_not_for_read(self):
        with tempfile.NamedTemporaryFile(delete=False) as tmp:
            tmp.write(b"123")
            tmp.flush()
            temp_path = tmp.name
        with self.assertRaises(BinaryFileError):
            with open(temp_path, "wb") as file_to_read:
                check_binary_file_for_reading(file_to_read)

    def test_raise_text_mode(self):
        with tempfile.NamedTemporaryFile(delete=False) as tmp:
            tmp.write(b"123")
            tmp.flush()
            temp_path = tmp.name
        with self.assertRaises(BinaryFileError):
            with open(temp_path, "rt") as file_to_read:
                check_binary_file_for_reading(file_to_read)

    def test_raise_not_a_real_file(self):
        class NotAFile:
            def read(self, n):
                pass

        with self.assertRaises(BinaryFileError):
            nf = NotAFile()
            nf.readable = lambda: True
            check_binary_file_for_reading(nf)


if __name__ == '__main__':
    main()

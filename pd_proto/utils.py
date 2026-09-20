import os
import sys

from pd_proto import _checksum, _checksum_file
from pd_proto.const import SupportsRead, READABLE, FILENO, ENCODING, PROTOCOL_VERSION
from pd_proto.errors import BinaryFileError, EmptyDataError, ProtocolError


def get_sys_handle(file) -> int:
    """
    Get the OS file descriptor or handle.

    :param file: A real file-like object present in the filesystem.
    :return: The system descriptor or handle as an integer.
    """
    fd = file.fileno()
    if sys.platform == "win32":
        import msvcrt  # pylint: disable=import-outside-toplevel
        return msvcrt.get_osfhandle(fd)
    return fd


def check_binary_file_for_reading(file: SupportsRead):
    """
    Verifies that the file is opened for reading bytes and represents an actual filesystem object (not a stream).
    :param file: binary file opened for reading
    :raises BinaryFileError: if it is not a real file, file is not readable, or opened for read text
    """
    if not getattr(file, READABLE, lambda: False)():
        raise BinaryFileError("The 'file' parameter must be real file opened for reading bytes.")
    if not hasattr(file, FILENO):
        raise BinaryFileError("Expected a real file on disk, not an in-memory stream.")
    if hasattr(file, ENCODING):
        raise BinaryFileError("Param 'file' must be a binary file opened for reading (unexpected 'encoding' attribute)")

def check_binary_and_get_descriptor(file: SupportsRead):
    """
    Verifies that the file is opened for reading bytes and represents an actual filesystem object (not a stream).
    :param file: binary file opened for reading
    :raises BinaryFileError: if it is not a real file, file is not readable, or opened for read text
    :raises EmptyDataError: If no data can be decoded.
    :raises ProtocolError: If the protocol version does not match the current one.
    :return: The system descriptor or handle as an integer.
    """
    check_binary_file_for_reading(file)
    fd = get_sys_handle(file)
    file_size = os.fstat(file.fileno()).st_size
    if file_size <= 1:
        raise EmptyDataError("Nothing to decrypt")
    if file.read(1)[0] > PROTOCOL_VERSION:
        raise ProtocolError(f"Supported protocol version is less or equal {PROTOCOL_VERSION}")
    return fd


def checksums(bts: bytes) -> int:
    """
    Calculates the Adler-32 checksum of the provided byte data.

    :param bts: The input binary data to be checksummed.
    :return: An integer representing the calculated checksum.
    :raises TypeError: If the input data is not of type bytes.
    """
    if not isinstance(bts, bytes):
        raise TypeError("Input data must be of type bytes")
    return _checksum(bts)


def checksum(file: SupportsRead) -> int:
    """
    Calculates the Adler-32 checksum of the provided binary file.

    :param file: A file opened to read bytes.
    :return: An integer representing the calculated checksum.
    :raises BinaryFileError: if it is not a real file, file is not readable, or opened for read text
    :raises PDProtoError: For any other error in the Rust backend.
    """
    check_binary_file_for_reading(file)
    fd = get_sys_handle(file)
    return _checksum_file(fd)

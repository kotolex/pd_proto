from src.pydata.const import AllowedTypes, PROTOCOL_VERSION, type_by_int, Type


def decode_varint(buffer: bytes) -> tuple[int, int]:
    """
    Декодирует VarInt-представление в число.
    :param buffer: набор байтов
    :return: целое положительное число(или 0) и количество прочитанных байтов
    """
    number = 0
    shift = 0
    bytes_read = 0

    for byte in buffer:
        bytes_read += 1
        # Забираем 7 полезных бит (маска 0x7F убирает старший бит-флаг)
        number |= (byte & 0x7F) << shift
        # Если старший бит равен 0 (проверка через И с маской 0x80), чтение окончено
        if not (byte & 0x80):
            break
        # Сдвигаем позицию для следующей семерки битов
        shift += 7
    return number, bytes_read


def decrypt(bts: bytes) -> AllowedTypes:
    if not bytes or bts[0] != PROTOCOL_VERSION or len(bts) == 1:
        raise AttributeError("Empty data or unsupported protocol version")
    result = []
    index = 1
    continuation = False
    while True:
        b = bts[index]
        next_token = type_by_int(b)
        if b >= Type.INT_POSITIVE.value:
            if b in (Type.INT_POSITIVE.value, Type.INT_NEGATIVE.value):
                val, read = decode_varint(bts[index + 1:])
                result.append(next_token*val)
                index+=read
        else:
            result.append(next_token)
        if not continuation:
            break
    if index + 1 >= len(bts):
        return result[0]
    raise AttributeError(f"Corrupt data, parse bytes {index}, total length is {len(bts)}")

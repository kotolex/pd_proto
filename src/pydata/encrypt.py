from src.pydata.const import AllowedTypes, int_by_type, PROTOCOL_VERSION


def _encrypt_base(data: AllowedTypes, result: bytearray | None = None) -> bytearray:
    if result is None:
        result = bytearray()
    match data:
        case bool() as x:
            result.append(int_by_type(x))
        case int() as y:
            result.append(int_by_type(y))
            if y < 0:
                y = (-1) * y
            result.extend(encode_varint(y))
        case _:
            result.append(int_by_type(data))
    return result


def encode_varint(number: int) -> bytearray:
    """
    Преобразует положительное целое число в байты, используя VarInt-формат.
    :param number: целое положительное число (или 0)
    :return: байты представления числа
    """
    if number < 0:
        raise ValueError("Positive numbers only!")
    if number == 0:
        return bytearray([0])
    result = bytearray()
    while number > 0:
        # Извлекаем младшие 7 бит с помощью маски 127 (0b01111111)
        byte = number & 0x7F
        # Сдвигаем число вправо на 7 бит для следующей итерации
        number >>= 7
        # Если после сдвига число не обнулилось, значит будут еще байты
        if number > 0:
            # Устанавливаем старший 8-й бит в 1 с помощью маски 128 (0b10000000)
            byte |= 0x80
        result.append(byte)
    return result


def encrypt(data: AllowedTypes) -> bytes:
    """
    Конвертируем допустимый python тип в набор байтов
    :param data: объект любого из допустимых типов
    :return: байты представления
    """
    final = bytearray()
    final.append(PROTOCOL_VERSION)
    tail = _encrypt_base(data)
    final.extend(tail)
    return bytes(final)

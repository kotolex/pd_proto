from enum import IntEnum

PROTOCOL_VERSION = 1

AllowedTypes = None | bool | int


class Type(IntEnum):
    """
    Допустимые типы и их код в результирующем кодировании
    """
    NULL = 0
    BOOL_TRUE = 1
    BOOL_FALSE = 2
    INT_POSITIVE = 3
    INT_NEGATIVE = 4

    # FLOAT = 5,
    # STRING=6,
    # LIST=7,
    # TUPLE=8,
    # SET=9
    # DICT=10


def int_by_type(value: AllowedTypes):
    """
    Вернет целое число - код каждого допустимого для конвертирования типа данных
    :param value: объект допустимого типа
    :return: код для конвертирования
    :raise ValueError если тип недопустим
    """
    if value is None:
        return 0
    if value is True:
        return 1
    if value is False:
        return 2
    if isinstance(value, int):
        if value >= 0:
            return 3
        return 4
    raise ValueError(f"Value of unsupported type {value}: {type(value)}")


def type_by_int(code: int) -> AllowedTypes:
    """
    Вернет объект по его коду, важно что вернется не тип, а именно объект данного типа
    :param code: код объекта
    :return: объект данного типа
    :raise ValueError если тип недопустим
    """
    if code == Type.NULL.value:
        return None
    if code == Type.BOOL_TRUE.value:
        return True
    if code == Type.BOOL_FALSE.value:
        return False
    if code == Type.INT_POSITIVE.value:
        return 1
    if code == Type.INT_NEGATIVE.value:
        return -1
    raise ValueError("Corrupt data")

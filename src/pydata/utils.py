from decimal import Decimal


def exponent(number: float) -> int:
    """
    Returns the number of decimal places
    :param number: float number
    :return: int
    """
    d = Decimal(str(number))
    exp = d.normalize().as_tuple().exponent
    return abs(exp)

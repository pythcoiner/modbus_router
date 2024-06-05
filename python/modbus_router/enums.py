import math
from enum import Enum


def crc16(data):
    """Compute the MODBUS CRC-16 of a given list of integers."""
    crc_register = 0xFFFF

    for byte in data:
        crc_register ^= byte
        for _ in range(8):
            if crc_register & 0x0001:
                crc_register >>= 1
                crc_register ^= 0xA001
            else:
                crc_register >>= 1

    # Split the CRC into two bytes (low and high byte)
    low_byte = crc_register & 0x00FF
    high_byte = (crc_register & 0xFF00) >> 8

    return [low_byte, high_byte]


# decorator implementing `to_int()` and `from_int()` methods
def into_int(enum_class):
    def to_int(self):
        return self.value

    @staticmethod
    def from_int(value):
        try:
            return enum_class(value)
        except ValueError:
            return None

    enum_class.to_int = to_int
    enum_class.from_int = from_int
    return enum_class


@into_int
class RequestType(Enum):
    VFD_REQUEST = 1
    VFD_RESPONSE = 2
    JOYSTICK_REQUEST = 3
    JOYSTICK_RESPONSE = 4
    
    
@into_int
class VfdFnCode(Enum):
    RUN = 1
    STOP = 2
    STATUS = 3


@into_int
class JoystickFnCode(Enum):
    X_POS = 1
    Y_POS = 2
    BUTTON = 3
    X_THUMB = 4
    Y_THUMB = 5


@into_int
class ModbusId:
    BROADCAST = 0
    RESERVED_START = 248
    RESERVED = 255
    _value = None
    
    def __init__(self, value):
        if 0 <= value <= 247:  # Device ID range
            self.value = value
        else:
            self.value = None
            
            
class ButtonState(Enum):
    PRESSED = 1
    RELEASED = 0
    
    def __init__(self, *args):
        self.id = None
        
        
class JoystickAxis(Enum):
    X = 1
    Y = 2
    X_THUMB = 4
    Y_THUMB = 5
    
    def __init__(self, *args):
        self.position = None
        self.id = None
    
    
class VfdStatus(Enum):
    RUN = 1
    STOP = 2
    
    def __init__(self, *args):
        self.id = None
        self.speed = None



        
        


from .enums import *


class Response:
    
    def __init__(self,
                 id: ModbusId = None,
                 type: RequestType = None,
                 function: VfdFnCode | JoystickFnCode = None,
                 value: int = None,
                 ):
        
        if type in [RequestType.VFD_REQUEST, RequestType.JOYSTICK_REQUEST]:
            self.invalid = True
        else:
            self.invalid = False
        
        self.id = id
        self.type = type
        self.function = function
        self.value = value
    
    def is_valid(self):
        if not isinstance(self.id, ModbusId):
            print(" Id is not valid")
            return False
        
        if not isinstance(self.function, (VfdFnCode, JoystickFnCode)):
            print("function is not valid")
            return False
        
        if self.type not in [RequestType.VFD_RESPONSE, RequestType.JOYSTICK_RESPONSE]:
            print("Response type cannot be Request")
            return False
        
        if self.id is None or self.function is None:
            print(" Id or Function is None")
            return False
        else:
            return True
    
    @staticmethod
    def from_frame(frame):
        print(f"from_frame({frame})")
        if not isinstance(frame, list):
            print("Frame is not a list")
            return None
        
        if len(frame) != 8:
            print("Wrong frame length")
            return None
        
        for i in frame:
            if not isinstance(i, int):
                print("Wrong data type")
                return None
            if i < 0:
                print("Negative value")
                return None
            if i > 0xff:
                print("Value > 0xff")
                return None
        
        # check crc
        crc = crc16(frame[:-2])
        if crc != frame[-2:]:
            print(f"{frame=}")
            print(f"Wrong CRC {crc=} vs {frame[-2:]=}")
            return None
        
        id = ModbusId(frame[0])
        type = RequestType(frame[1])
        
        if type == RequestType.VFD_RESPONSE:
            fn_code = VfdFnCode(frame[2])
        
        elif type == RequestType.JOYSTICK_RESPONSE:
            fn_code = JoystickFnCode(frame[2])
        else:
            print("Response cannot be Request type")
            return None
        
        # deserializing data
        value = 0
        match fn_code:
            case VfdFnCode.RUN:
                print("RUN response are not expected")
                return None
            
            case VfdFnCode.STOP:
                print("STOP response are not expected")
                return None
            
            case _:
                if frame[3] not in [0, 1]:
                    print("Invalid sign value")
                    return None
                value = (frame[4] << 8) + frame[5]
                if frame[3] == 1:
                    value = -value
        
        out = Response(id, type, fn_code, value)
        if out.is_valid():
            return out
        else:
            print("Response.is_valid == False")
            return None
                    
            
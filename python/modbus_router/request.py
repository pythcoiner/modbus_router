from .enums import *


class Request:
    
    def __init__(self,
                 id: ModbusId = None,
                 type: RequestType = None,
                 function: VfdFnCode | JoystickFnCode = None,
                 data1: int = None,
                 data2: int = None,
                 data3: int = None, ):
        
        if type in [RequestType.VFD_RESPONSE, RequestType.JOYSTICK_RESPONSE]:
            self.invalid = True
        else:
            self.invalid = False
        
        self.id = id
        self.type = type
        self.function = function
        self.data1 = data1
        self.data2 = data2
        self.data3 = data3

    def __repr__(self):
        return str(f"Request({self.to_frame()})")
        
    def to_frame(self):
        if None in [self.id, self.type, self.function]:
            return None
        
        if not isinstance(self.id, ModbusId):
            return None
        
        if not isinstance(self.type, RequestType):
            return None
        
        if not isinstance(self.function, (VfdFnCode, JoystickFnCode)):
            return None
        
        if type in [RequestType.VFD_RESPONSE, RequestType.JOYSTICK_RESPONSE]:
            return None
        
        frame = [0] * 8
        
        frame[0] = self.id.to_int()
        frame[1] = self.type.to_int()
        frame[2] = self.function.to_int()
        
        if self.type == RequestType.VFD_REQUEST:
            match self.function:
                case VfdFnCode.RUN:
                    frame[3] = self.data1
                    frame[4] = self.data2
                    frame[5] = self.data3
                case _:
                    pass
        
        (frame[6], frame[7]) = crc16(frame[:-2])
        if self.is_frame_valid(frame):
            return frame
    
    def is_frame_valid(self, frame: []):
        if self.invalid:
            return False
        for i in frame:
            if i is None:
                return False
        return True
    
    def is_valid(self):
        if self.type in [RequestType.VFD_RESPONSE, RequestType.JOYSTICK_RESPONSE]:
            self.invalid = True
        else:
            self.invalid = False
        
        if not isinstance(self.id, ModbusId):
            return False
        
        if not isinstance(self.function, (VfdFnCode, JoystickFnCode)):
            return False
        
        if self.type in [RequestType.VFD_RESPONSE, RequestType.JOYSTICK_RESPONSE]:
            return False
        
        if self.invalid or self.id is None or self.function is None :
            return False
        else:
            return True
        
    @staticmethod
    def vfd_run(id: int, hertz: float):
        # convert float to i16
        ref = round(hertz * 100)
        ref = max(-32767, min(ref, 32767))
        if ref < 0:
            data1 = 1
        else:
            data1 = 0
        ref = abs(ref)
        data2 = (ref & 0xff00) >> 8
        data3 = ref & 0x00ff
        
        return Request(ModbusId(id),
                       RequestType.VFD_REQUEST,
                       VfdFnCode.RUN,
                       data1,
                       data2,
                       data3)
    
    @staticmethod
    def vfd_stop(id: int):
        return Request(ModbusId(id),
                       RequestType.VFD_REQUEST,
                       VfdFnCode.STOP)
    
    @staticmethod
    def vfd_status(id: int):
        return Request(ModbusId(id),
                       RequestType.VFD_REQUEST,
                       VfdFnCode.STATUS)
    
    

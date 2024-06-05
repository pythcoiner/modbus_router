# this line should be replaced in pymodbus/server/async_io.py::ModbusServerRequestHandler._async_execute()
# context = self.server.context[request.slave_id]
# by
# if request.slave_id in self.server.context[request.slave_id].keys():
#     context = self.server.context[request.slave_id][request.slave_id]
#     ~
import logging
import random

from pymodbus.server import StartSerialServer
from pymodbus.datastore import ModbusServerContext, ModbusSlaveContext, ModbusSequentialDataBlock, ModbusSparseDataBlock
from pymodbus.transaction import ModbusRtuFramer

logging.basicConfig()
log = logging.getLogger()
log.setLevel(logging.INFO)

port = '/dev/ttyUSB1'


class MegmeetDataBlock(ModbusSparseDataBlock):
    STATUS = 0x6505
    CMD = 0x6400
    REF = 0x6401
    
    STOP = 0x35
    FW = 0x34
    RV = 0x3c
    
    def __init__(self, id, *args, **kwargs):
        super(MegmeetDataBlock, self).__init__(*args, **kwargs)
        self.values = {self.CMD: 0, self.REF: 0, self.STATUS: 4964}
        self.stop = False
        self.speed = 0
        self.name = "MEGMEET"
        self.id = id

    def getValues(self, address, count=1):
        data = super(MegmeetDataBlock, self).getValues(address, count)
        if address == self.STATUS and count == 1:
            data[0] = data[0] - 100 + random.randint(0, 140)
            # log.info(f"{self.name} {self.id} read status {data}")
        return data

    def setValues(self, address, values, use_as_default=False):
        match address:
            case self.CMD:
                match values[0]:
                    case self.FW:
                        log.info(f"{self.name} {self.id} FW")
                        speed = 5000
                        super(MegmeetDataBlock, self).setValues(self.STATUS, [speed])
                    case self.RV:
                        log.info(f"{self.name} {self.id} RV")
                        speed = 5000
                        speed = speed | int(b'10000000')
                        super(MegmeetDataBlock, self).setValues(self.STATUS, [speed])
                    case self.STOP:
                        log.info(f"{self.name} {self.id} STOP")
                        speed = 0
                        super(MegmeetDataBlock, self).setValues(self.STATUS, [speed])
            case self.REF:
                log.info(f"{self.name} {self.id} write ref: {values[0]}")
        super(MegmeetDataBlock, self).setValues(address, values)


class FreconDataBlock(ModbusSparseDataBlock):
    STATUS = 0x3000
    CMD = 0x2000
    REF = 0x2001

    STOP = 0x05
    FW = 0x01
    RV = 0x02

    def __init__(self, id, *args, **kwargs):
        super(FreconDataBlock, self).__init__(*args, **kwargs)
        self.values = {self.CMD: 0, self.REF: 0, self.STATUS: 4964}
        self.stop = False
        self.speed = 0
        self.name = "FRECON"
        self.id = id

    def getValues(self, address, count=1):
        data = super(FreconDataBlock, self).getValues(address, count)
        if address == self.STATUS and count == 1:
            data[0] = data[0] - 100 + random.randint(0, 140)
            # log.info(f"{self.name} {self.id} read status {data}")
        return data

    def setValues(self, address, values, use_as_default=False):
        match address:
            case self.CMD:
                match values[0]:
                    case self.FW:
                        log.info(f"{self.name} {self.id} FW")
                        speed = 5000
                        super(FreconDataBlock, self).setValues(self.STATUS, [speed])
                    case self.RV:
                        log.info(f"{self.name} {self.id} RV")
                        speed = 5000
                        speed = speed | int(b'10000000')
                        super(FreconDataBlock, self).setValues(self.STATUS, [speed])
                    case self.STOP:
                        log.info(f"{self.name} {self.id} STOP")
                        speed = 0
                        super(FreconDataBlock, self).setValues(self.STATUS, [speed])
            case self.REF:
                log.info(f"{self.name} {self.id} write ref: {values[0]}")
        super(FreconDataBlock, self).setValues(address, values)


def megmeet(id: int):
    vfd = MegmeetDataBlock(id)
    return ModbusSlaveContext(di=None, co=None, hr=vfd, ir=None, zero_mode=True)


def frecon(id: int):
    vfd = FreconDataBlock(id)
    return ModbusSlaveContext(di=None, co=None, hr=vfd, ir=None, zero_mode=True)


vfd10 = megmeet(10)
vfd11 = megmeet(11)
vfd12 = frecon(12)

vfd22 = megmeet(22)
vfd20 = frecon(20)
vfd21 = frecon(21)
vfd26 = frecon(26)
vfd27 = frecon(27)

vfd30 = frecon(30)
vfd31 = frecon(31)

vfd40 = megmeet(40)

vfd43 = frecon(43)
vfd50 = frecon(50)
vfd51 = megmeet(51)
vfd60 = megmeet(60)
vfd61 = megmeet(61)

context = ModbusServerContext({10: vfd10,
                               11: vfd11,
                               12: vfd12,
                               20: vfd20,
                               21: vfd21,
                               22: vfd22,
                               26: vfd26,
                               27: vfd27,
                               30: vfd30,
                               31: vfd31,
                               40: vfd40,
                               43: vfd43,
                               50: vfd50,
                               51: vfd51,
                               60: vfd60,
                               61: vfd61})

StartSerialServer(context=context,
                  framer=ModbusRtuFramer,
                  port=port,
                  baudrate=115200,
                  ignore_missing_slaves=True
                  )

import sys
from copy import deepcopy as copy
from PySide6.QtCore import QCoreApplication, Signal, QTimer
from modbus_router.modbus import *


class FakeDevice(QObject):
    emit_request = Signal(object)

    def __init__(self, id: int, request: Request):
        QObject.__init__(self)
        self.request = request
        self.id = ModbusId(id)

    def poll(self):
        self.emit_request.emit(copy(self.request))

    def handle_response(self, response: Response):
        print(f"FakeDevice<{self.id.to_int()}>.handle_request({response})")


app = QCoreApplication()

router = ModbusRouter(app,
                      "/dev/ttyUSB0",
                      "error",
                      "error",
                      '/home/pyth/rust/modbus_router/python/modbus')

device1 = FakeDevice(10, Request(
    ModbusId(10),
    RequestType.VFD_REQUEST,
    VfdFnCode.STATUS,
))

device1.emit_request.connect(router.handle_request)
router.register(device1.id.to_int(), device1.handle_response)

device1_timer = QTimer()
device1_timer.timeout.connect(device1.poll)

router_timer = QTimer()
router_timer.timeout.connect(router.read_router_stdout)
router_timer.timeout.connect(router.read_router_stderr)

router.start()
router_timer.start(1)
time.sleep(3)
device1_timer.start(500)

sys.exit(app.exec())


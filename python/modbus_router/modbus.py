import _io
import io
import os
import select
import subprocess
import sys
import time

from PySide6.QtCore import QObject, Signal, QThread, QCoreApplication
from .enums import *
from .request import Request
from .response import Response


class Connector(QObject):
    signal = Signal(object)

    def __init__(self):
        QObject.__init__(self)

    def send(self, response: Response):
        if not isinstance(response, Response):
            return
        self.signal.emit(response)


FRAME_LENGTH = 8


class StdinDecoder:

    def __init__(self, stdin: _io.BufferedReader):
        self.stdin = stdin
        self.buff = []

    def try_read(self) -> [int]:
        # Try to read & push 1 byte to buffer
        readable, _, _ = select.select([self.stdin], [], [], 0)
        if readable:
            self.buff.append(int.from_bytes(self.stdin.read(1), byteorder='big'))
            while len(self.buff) >= FRAME_LENGTH:
                # Check if the first FRAME_LENGTH bytes have a valid CRC
                if self.check_crc(self.buff[:FRAME_LENGTH]):
                    frame = self.buff[:FRAME_LENGTH]
                    # Remove the processed frame from the buffer
                    self.buff = self.buff[FRAME_LENGTH:]
                    return frame
                # Remove the first byte to check the next window
                self.buff.pop(0)

    @staticmethod
    def check_crc(frame):
        if len(frame) == FRAME_LENGTH:
            crc = crc16(frame[:FRAME_LENGTH-2])
            return crc == frame[FRAME_LENGTH-2:]
        return False


class ModbusRouter(QThread):

    def __init__(self, parent: QCoreApplication, port, level_router, level_serial, binary='modbus'):
        super().__init__(parent=parent)

        self.connectors = {}
        self.process = None
        self.port = port
        self.level_router = level_router
        self.level_serial = level_serial
        self.binary = binary
        self.stdin = None

    def run(self):
        self.command = os.path.abspath(self.binary)
        command = [self.command, self.port, self.level_router, self.level_serial]
        print(command)

        env_vars = os.environ.copy()
        env_vars['RUST_BACKTRACE'] = '1'

        time.sleep(0.2)
        self.process = subprocess.Popen(command,
                                        stdin=subprocess.PIPE,
                                        stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE,
                                        env=env_vars,)

        self.stdin = StdinDecoder(self.process.stdout)
        print(f"{self.command} is running")
        # start polling modbus stdout
        print(f"ModbusRouter start polling {self.command}")

    def register(self, id: int, slot):
        connector = Connector()
        connector.signal.connect(slot)
        if id not in self.connectors.keys():
            self.connectors[str(id)] = connector
        else:
            raise ConnectionError(f"Connector with id {id} is already registered!")

    # incoming request should arrive at this slot
    def handle_request(self, request: Request):
        if not isinstance(request, Request):
            # TODO: do not return silently
            return

        print(f"ModbusRouter.handle_request({request})")

        frame = request.to_frame()
        if frame:
            self.send_frame(frame)

    def send_frame(self, frame: []):
        if self.process:
            print(type(self.process.stdin))
            self.process.stdin.write(bytes(frame))
            self.process.stdin.flush()
        else:
            print("modbus server have crashed!!")

    def read_router_stdout(self):
        if self.process:
            output = self.stdin.try_read()
            if output:
                print(f"ModbusRouter message from {self.command}: {output}")
                self.handle_response(output)

    def read_router_stderr(self):
        if self.process:
            ready, _, _ = select.select([self.process.stderr], [], [], 0)
            if ready:
                error_message = self.process.stderr.readline()
                msg = error_message.decode().replace("\n", "")
                print(msg)

    def handle_response(self, frame):
        response = Response.from_frame(frame)
        if response and response.is_valid():
            key = str(response.id.to_int())
            if key not in self.connectors.keys():
                print(f"No device registered with id {id}!")
            connector = self.connectors[key]
            print(f"ModbusRouter transmit response to device {key}")
            connector.send(response)

        else:
            print("Cannot parse error")

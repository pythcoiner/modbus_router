from enums import *
from request import Request
from response import Response


def test_request_type_to_from_int():
    assert RequestType.VFD_REQUEST.to_int() == 1
    assert RequestType.from_int(1) == RequestType.VFD_REQUEST
    assert RequestType.from_int(999) is None


def test_vfd_fn_code_to_from_int():
    assert VfdFnCode.RUN.to_int() == 1
    assert VfdFnCode.from_int(1) == VfdFnCode.RUN
    assert VfdFnCode.from_int(999) is None


def test_joystick_fn_code_to_from_int():
    assert JoystickFnCode.X_POS.to_int() == 1
    assert JoystickFnCode.from_int(1) == JoystickFnCode.X_POS
    assert JoystickFnCode.from_int(999) is None


def test_modbus_id_valid():
    assert ModbusId(10).value == 10
    assert ModbusId(0).value == 0
    assert ModbusId(247).value == 247
    assert ModbusId(3).to_int() == 3
    assert ModbusId.from_int(5).value == ModbusId(5).value


def test_modbus_id_invalid():
    assert ModbusId(248).value is None
    assert ModbusId(999).value is None


def test_request():
    assert Request(ModbusId(3),
                   RequestType.VFD_REQUEST,
                   VfdFnCode.RUN,
                   data1=1,
                   data2=0x13,
                   data3=0x88).to_frame() == [3, 1, 1, 0x01, 0x13, 0x88, 96, 130]
    
    assert Request(ModbusId(3),
                   RequestType.VFD_REQUEST,
                   VfdFnCode.STOP).to_frame() == [3, 1, 2, 0, 0, 0, 60, 80]
    
    assert Request(ModbusId(3),
                   RequestType.VFD_REQUEST,
                   VfdFnCode.STOP,
                   data1=1,
                   data2=2,
                   data3=3).to_frame() == [3, 1, 2, 0, 0, 0, 60, 80]
    
    assert Request.vfd_stop(3).to_frame() == [3, 1, 2, 0, 0, 0, 60, 80]
    
    assert Request.vfd_run(3, 50.00).to_frame() == [3, 1, 1, 0, 19, 136, 49, 66]
    assert Request.vfd_run(3, -50.00).to_frame() == [3, 1, 1, 1, 19, 136, 96, 130]
    
    assert Request.vfd_status(3).to_frame() == [3, 1, 3, 0, 0, 0, 61, 172]
    
    assert Request(3,
                   RequestType.VFD_REQUEST,
                   VfdFnCode.STOP).is_valid() is False
    
    assert Request(3,
                   RequestType.VFD_REQUEST,
                   VfdFnCode.STOP).to_frame() is None


def frame_response(frame: []):
    crc = crc16(frame)
    frame += crc
    return frame


def test_response_valid():
    # Valid VFD Response Frame
    frame = frame_response([3, 2, 3, 0, 0x13, 0x88])  # STATUS / 50.00Hz
    response = Response.from_frame(frame)
    assert response is not None
    assert response.id.value == 3
    assert response.type == RequestType.VFD_RESPONSE
    assert response.function == VfdFnCode.STATUS
    assert response.value == 5000  # Assuming this is the correct interpretation of the data
    
    # Valid Joystick Response Frame
    frame = frame_response([5, 4, 1, 0, 0x10, 0x20])
    response = Response.from_frame(frame)
    assert response is not None
    assert response.id.value == 5
    assert response.type == RequestType.JOYSTICK_RESPONSE
    assert response.function == JoystickFnCode.X_POS
    assert response.value == 4128  # Assuming correct data interpretation


def test_response_invalid():
    # Invalid due to incorrect CRC
    frame = [3, 2, 1, 0, 0x13, 0x88, 0x00, 0x00]
    assert Response.from_frame(frame) is None
    
    # Invalid due to incorrect frame length
    frame = [3, 2, 1, 0, 0x13, 0x88]
    assert Response.from_frame(frame) is None
    
    # Invalid due to non-integer elements
    frame = [3, 2, 1, 0, "0x13", 0x88, 0xA5, 0xB4]
    assert Response.from_frame(frame) is None
    
    # Invalid due to out-of-bounds integer
    frame = [3, 2, 1, 0, 0x13, 0x88, 0x1FFFF, 0xB4]
    assert Response.from_frame(frame) is None


def test_response_negative_value():
    # Testing negative value handling
    frame = frame_response([4, 2, 3, 1, 0x01, 0xf4])
    response = Response.from_frame(frame)
    assert response is not None
    assert response.value == -500  # Assuming correct interpretation for negative values


def test_response_unexpected_function_code():
    # Frame with an unexpected function code
    frame = [3, 2, 99, 0, 0x13, 0x88, 0xA5, 0xB4]  # Invalid function code
    assert Response.from_frame(frame) is None

#!/usr/bin/env python3

# A mock backend for the NUT server.
# Linted with flake8.

import socket
import sys

# Server endpoint
SERVER_HOST = ""
SERVER_PORT = 3493
# Max request length in bytes (avoid DoS)
RECV_BUFFER_MAX_BYTES = 4096
UPS_EXPECTED = "alpha"

DATA_VER = """\
Network UPS Tools upsd 2.7.4 "yolo" - http://www.networkupstools.org/
"""
COMMAND_UPS_LIST = "list ups"
DATA_UPS_LIST = """\
BEGIN LIST UPS
UPS alpha "desc 1"
END LIST UPS
"""
COMMAND_VAR_LIST = "list var"  # Plus UPS name
DATA_VAR_LIST = """\
BEGIN LIST VAR alpha
VAR alpha battery.charge "100"
VAR alpha battery.charge.low "10"
VAR alpha battery.charge.warning "20"
VAR alpha battery.mfr.date "1"
VAR alpha battery.runtime "1320"
VAR alpha battery.runtime.low "300"
VAR alpha battery.type "PbAcid"
VAR alpha battery.voltage "260.0"
VAR alpha battery.voltage.nominal "120"
VAR alpha device.mfr "1"
VAR alpha device.model "2200R"
VAR alpha device.serial "HIDDEN"
VAR alpha device.type "ups"
VAR alpha driver.name "usbhid-ups"
VAR alpha driver.parameter.offdelay "60"
VAR alpha driver.parameter.ondelay "120"
VAR alpha driver.parameter.pollfreq "30"
VAR alpha driver.parameter.pollinterval "2"
VAR alpha driver.parameter.port "auto"
VAR alpha driver.parameter.synchronous "no"
VAR alpha driver.version "2.7.4"
VAR alpha driver.version.data "CyberPower HID 0.4"
VAR alpha driver.version.internal "0.41"
VAR alpha input.transfer.high "290"
VAR alpha input.transfer.low "165"
VAR alpha input.voltage "238.7"
VAR alpha input.voltage.nominal "230"
VAR alpha output.voltage "237.2"
VAR alpha ups.beeper.status "enabled"
VAR alpha ups.delay.shutdown "60"
VAR alpha ups.delay.start "120"
VAR alpha ups.load "21"
VAR alpha ups.mfr "1"
VAR alpha ups.model "2200R"
VAR alpha ups.productid "0601"
VAR alpha ups.realpower.nominal "2200"
VAR alpha ups.serial "HIDDEN"
VAR alpha ups.status "OL CHRG"
VAR alpha ups.timer.shutdown "-60"
VAR alpha ups.timer.start "-60"
VAR alpha ups.vendorid "0764"
END LIST VAR alpha
"""


class EmptyObject:
    pass


def main():
    print(f"Starting mock NUT server on {SERVER_HOST}:{SERVER_PORT}")
    server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    server.bind((SERVER_HOST, SERVER_PORT))
    server.listen()

    # Accept clients
    while True:
        client = EmptyObject()
        try:
            client.connection, (client.address, client.port) = server.accept()
        except OSError:
            # Socket closed (probably)
            break
        with client.connection:
            try:
                log("New client", client)
                handleClient(client)
                log("Closing client", client)
            except Exception as err:
                log(f"Error {type(err).__name__} during request: {err}", client, error=True)


def handleClient(client):
    # Use wrapper so we can easily reassign it for modifications
    lineBufferPtr = EmptyObject()
    lineBufferPtr.value = bytearray()
    while True:
        line = readRequestLine(lineBufferPtr, client)
        if not line:
            break
        handleRequest(line, client)

def readRequestLine(lineBufferPtr, client):
    while True:
        ok, line = readRequestLineInner(lineBufferPtr, client)
        if not ok:
            return None
        if line:
            log(f"New request: {line}", client)
            return line

        # Fail if max length reached (without any complete lines)
        bufferSize = len(lineBufferPtr.value)
        if bufferSize >= RECV_BUFFER_MAX_BYTES:
            log("Error: Too long request", client, error=True)
            return None


def readRequestLineInner(lineBufferPtr, client):
    bufferSize = len(lineBufferPtr.value)
    data = client.connection.recv(RECV_BUFFER_MAX_BYTES - bufferSize)
    # Check EOF
    if not data:
        return False, None
    lineBufferPtr.value.extend(data)
    bufferSize += len(data)

    # Check for line ending (using simple LF)
    i = -1
    while True:
        i += 1
        if i >= bufferSize:
            break

        # Strip leading spaces
        if i == 0 and lineBufferPtr.value[i] == ord(' '):
            lineBufferPtr.value = lineBufferPtr.value[:i] + lineBufferPtr.value[i+1:]
            bufferSize -= 1
            i -= 1
            continue

        # Strip CR
        if lineBufferPtr.value[i] == ord('\r'):
            lineBufferPtr.value = lineBufferPtr.value[:i] + lineBufferPtr.value[i+1:]
            bufferSize -= 1
            i -= 1
            continue

        # Check if complete line, extract from buffer and return early if found
        if lineBufferPtr.value[i] == ord('\n'):
            line = lineBufferPtr.value[:i].decode()
            lineBufferPtr.value = lineBufferPtr.value[i+1:]
            return True, line

    # No lines were found (yet)
    return True, None


def handleRequest(line, client):
    def sendText(message):
        client.connection.sendall(message.encode())
    lowerLine = line.lower()
    lineParts = line.split()
    numLineParts = len(lineParts)
    if numLineParts == 1 and lowerLine.startswith("ver"):
        sendText(DATA_VER)
    elif numLineParts == 2 and lowerLine.startswith("list ups"):
        sendText(DATA_UPS_LIST)
    elif numLineParts == 3 and lowerLine.startswith("list var"):
        if lineParts[2] == UPS_EXPECTED:
            sendText(DATA_VAR_LIST)
        else:
            sendText("ERR UPS not found\n")
    elif numLineParts == 1 and lowerLine.startswith("logout"):
        sendText(DATA_VER)
    else:
        sendText("ERR Unknown command\n")


def log(message, client=None, error=False):
    output = sys.stderr if error else sys.stdout
    if client:
        print(f"[{client.address}:{client.port}] " + message, file=output)
    else:
        print(message, file=output)


if __name__ == "__main__":
    main()

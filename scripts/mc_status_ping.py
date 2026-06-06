#!/usr/bin/env python3
import json
import socket
import struct
import sys


def varint(value: int) -> bytes:
    out = bytearray()
    while True:
        temp = value & 0x7F
        value >>= 7
        if value:
            temp |= 0x80
        out.append(temp)
        if not value:
            return bytes(out)


def read_varint(sock: socket.socket) -> int:
    value = 0
    shift = 0
    for _ in range(5):
        b = sock.recv(1)
        if not b:
            raise EOFError("socket closed while reading varint")
        byte = b[0]
        value |= (byte & 0x7F) << shift
        if not (byte & 0x80):
            return value
        shift += 7
    raise ValueError("varint too long")


def read_exact(sock: socket.socket, length: int) -> bytes:
    data = bytearray()
    while len(data) < length:
        chunk = sock.recv(length - len(data))
        if not chunk:
            raise EOFError(f"socket closed while reading {length} bytes")
        data.extend(chunk)
    return bytes(data)


def packet(packet_id: int, payload: bytes = b"") -> bytes:
    body = varint(packet_id) + payload
    return varint(len(body)) + body


def main() -> int:
    host = sys.argv[1] if len(sys.argv) > 1 else "127.0.0.1"
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 25565
    protocol = int(sys.argv[3]) if len(sys.argv) > 3 else 773
    timeout = float(sys.argv[4]) if len(sys.argv) > 4 else 5.0
    host_bytes = host.encode("utf-8")
    handshake = (
        varint(protocol)
        + varint(len(host_bytes))
        + host_bytes
        + struct.pack(">H", port)
        + varint(1)
    )
    with socket.create_connection((host, port), timeout=timeout) as sock:
        sock.sendall(packet(0, handshake))
        sock.sendall(packet(0))
        _length = read_varint(sock)
        packet_id = read_varint(sock)
        if packet_id != 0:
            raise RuntimeError(f"unexpected status packet id {packet_id}")
        json_length = read_varint(sock)
        data = read_exact(sock, json_length)
        print(json.dumps(json.loads(data.decode("utf-8")), ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

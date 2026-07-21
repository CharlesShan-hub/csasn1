# csasn1_example.py — Python 通过 ctypes 调用 Rust ASN.1 编解码
#
# 使用方法:
#   cargo build  # 先编译 Rust 生成 csasn1.dll
#   python csasn1_example.py

import ctypes
import json
import os

# ─── 加载 DLL ───
dll_path = os.path.join("target", "debug", "csasn1.dll")
csasn1 = ctypes.CDLL(dll_path)


# ─── Buffer 结构体（对应 Rust 端的 #[repr(C)]） ───
class Buffer(ctypes.Structure):
    _fields_ = [
        ("data", ctypes.POINTER(ctypes.c_ubyte)),
        ("len", ctypes.c_size_t),
        ("capacity", ctypes.c_size_t),
    ]


# ─── 函数签名声明 ───
csasn1.csasn1_encode.argtypes = [
    ctypes.c_char_p,  # type_name
    ctypes.c_char_p,  # encoding ("ber"/"der"/"aper"/"uper")
    ctypes.c_char_p,  # json
]
csasn1.csasn1_encode.restype = Buffer

csasn1.csasn1_decode.argtypes = [
    ctypes.c_char_p,  # type_name
    ctypes.c_char_p,  # encoding
    ctypes.POINTER(ctypes.c_ubyte),  # data
    ctypes.c_size_t,  # len
]
csasn1.csasn1_decode.restype = Buffer

csasn1.csasn1_free_buffer.argtypes = [Buffer]
csasn1.csasn1_free_buffer.restype = None


def encode(type_name: str, encoding: str, data: dict) -> bytes:
    """Python dict → JSON → ASN.1 编码 → 字节流"""
    json_str = json.dumps(data)
    buf = csasn1.csasn1_encode(
        type_name.encode(),
        encoding.encode(),
        json_str.encode(),
    )
    result = bytes(ctypes.cast(
        buf.data, ctypes.POINTER(ctypes.c_ubyte * buf.len)
    ).contents)
    csasn1.csasn1_free_buffer(buf)
    return result


def decode(type_name: str, encoding: str, data: bytes) -> dict:
    """字节流 → ASN.1 解码 → JSON → Python dict"""
    buf = csasn1.csasn1_decode(
        type_name.encode(),
        encoding.encode(),
        (ctypes.c_ubyte * len(data))(*data),
        len(data),
    )
    json_str = bytes(ctypes.cast(
        buf.data, ctypes.POINTER(ctypes.c_ubyte * buf.len)
    ).contents).decode("utf-8")
    csasn1.csasn1_free_buffer(buf)

    if json_str.startswith("ERROR:"):
        raise RuntimeError(json_str)
    return json.loads(json_str)


# ══════════════════════════════════════════
if __name__ == "__main__":
    print("=== Python → Rust ASN.1 编解码 ===\n")

    # ─── 构造 Apdu ───
    apdu = {
        "apch": {
            "cc": {"next": False, "resp": True, "err": False, "pi": 0},
            "sc": 0x51,
            "fl": 0,
        },
        "asdu": "01020304",
    }

    # ─── 测试不同编码方式 ───
    for enc in ["ber", "der", "aper", "uper"]:
        encoded = encode("Apdu", enc, apdu)
        decoded = decode("Apdu", enc, encoded)
        ok = "✓" if decoded == apdu else "✗"
        print(f"  {enc.upper():<6} {len(encoded):>4} bytes  {ok}")

    print("\nPython 用法总结:")
    print("  encoded = encode('Apdu', 'aper', {'apch': {...}, 'asdu': '...'})")
    print("  decoded = decode('Apdu', 'aper', binary_data)")

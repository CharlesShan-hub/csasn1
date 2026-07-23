use super::helpers;

/// Generate _native.py — ctypes bridge
pub fn gen_native(_prefix: &str, _package: &str, default_enc: &str) -> String {
    let mut c = String::new();
    c.push_str(&helpers::gen_header_comment());
    c.push_str("# ctypes bridge to native asn1.dll\n\n");
    c.push_str("import ctypes\nimport json\nimport os\nimport sys\n\n");
    c.push_str("_lib = None\n\n");
    c.push_str("def _load_lib():\n    global _lib\n");
    c.push_str("    if _lib is not None:\n        return _lib\n");
    c.push_str("    dll_name = \"asn1.dll\" if sys.platform == \"win32\" else \"libasn1.so\"\n");
    c.push_str("    search_paths = [\n");
    c.push_str("        os.path.dirname(__file__),\n");
    c.push_str("        os.path.join(os.path.dirname(__file__), \"resources\"),\n");
    c.push_str("        os.path.join(os.path.dirname(__file__), \"..\", \"resources\"),\n");
    c.push_str("    ]\n");
    c.push_str("    for p in search_paths:\n");
    c.push_str("        full = os.path.join(p, dll_name)\n");
    c.push_str("        if os.path.exists(full):\n");
    c.push_str("            _lib = ctypes.cdll.LoadLibrary(full)\n");
    c.push_str("            _lib.csasn1_encode.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_char_p]\n");
    c.push_str("            _lib.csasn1_encode.restype = ctypes.c_void_p\n");
    c.push_str("            _lib.csasn1_decode.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.POINTER(ctypes.c_ubyte), ctypes.c_int]\n");
    c.push_str("            _lib.csasn1_decode.restype = ctypes.c_void_p\n");
    c.push_str("            _lib.csasn1_free_string.argtypes = [ctypes.c_void_p]\n");
    c.push_str("            _lib.csasn1_free_string.restype = None\n");
    c.push_str("            return _lib\n");
    c.push_str("    raise RuntimeError(f\"Cannot find {dll_name}\")\n\n");

    c.push_str("def _read_and_free(ptr) -> str:\n");
    c.push_str("    try:\n        return ctypes.cast(ptr, ctypes.c_char_p).value.decode(\"utf-8\")\n");
    c.push_str("    finally:\n        _load_lib().csasn1_free_string(ptr)\n\n");

    c.push_str(&format!("_ENCODING = \"{}\"\n\n", default_enc));

    c.push_str("def encode(json_str: str, type_name: str | None = None) -> bytes:\n");
    c.push_str("    lib = _load_lib()\n");
    c.push_str("    tn = (type_name or \"UNKNOWN\").encode()\n    enc = _ENCODING.encode()\n    js = json_str.encode()\n");
    c.push_str("    resp = _read_and_free(lib.csasn1_encode(tn, enc, js))\n");
    c.push_str("    result = json.loads(resp)\n");
    c.push_str("    if result.get(\"ok\"):\n        return bytes(result[\"bytes\"])\n");
    c.push_str("    raise RuntimeError(f\"encode failed: {result.get('error')}\")\n\n");

    c.push_str("def decode_raw(data: bytes, type_name: str | None = None) -> str:\n");
    c.push_str("    lib = _load_lib()\n");
    c.push_str("    tn = (type_name or \"UNKNOWN\").encode()\n    enc = _ENCODING.encode()\n");
    c.push_str("    buf = (ctypes.c_ubyte * len(data)).from_buffer_copy(data)\n");
    c.push_str("    resp = _read_and_free(lib.csasn1_decode(tn, enc, buf, len(data)))\n");
    c.push_str("    result = json.loads(resp)\n");
    c.push_str("    if result.get(\"ok\"):\n        return json.dumps(result[\"value\"])\n");
    c.push_str("    raise RuntimeError(f\"decode failed: {result.get('error')}\")\n");
    c
}

/// Generate _base.py — serialization helpers
pub fn gen_base(_prefix: &str, _package: &str) -> String {
    let mut c = String::new();
    c.push_str(&helpers::gen_header_comment());
    c.push_str("# Base utilities\n\n");
    c.push_str("import json\nfrom dataclasses import dataclass, field\nfrom typing import Any\n\n");
    c.push_str("from ._native import encode, decode_raw\n\n");

    // --- Lenient serialization (skips None, used by encode_test) ---
    c.push_str("def to_json(obj) -> str:\n    return json.dumps(_to_dict(obj))\n\n");
    c.push_str("def _to_dict(obj) -> Any:\n");
    c.push_str("    if obj is None:\n        return None\n");
    c.push_str("    if isinstance(obj, bytes):\n        return obj.hex()\n");
    c.push_str("    if isinstance(obj, list):\n        return [_to_dict(x) for x in obj]\n");
    c.push_str("    # BIT STRING with _bit_count: convert int to hex string\n");
    c.push_str("    if hasattr(obj, '_bit_count') and hasattr(obj, 'value'):\n");
    c.push_str("        return bit_string_hex(obj.value, obj._bit_count)\n");
    c.push_str("    # Newtype wrapper: unwrap to raw value\n");
    c.push_str("    if hasattr(obj, '__dataclass_fields__'):\n");
    c.push_str("        fields = obj.__dataclass_fields__\n");
    c.push_str("        if set(fields.keys()) == {'value'}:\n");
    c.push_str("            return _to_dict(obj.value)\n");
    c.push_str("    # CHOICE: only serialize the selected variant\n");
    c.push_str("    if hasattr(obj, '_choice') and obj._choice:\n");
    c.push_str("        v = getattr(obj, obj._choice, None)\n");
    c.push_str("        if v is not None:\n            return {obj._choice: _to_dict(v)}\n");
    c.push_str("        return {}\n");
    c.push_str("    # Dataclass: skip None values (lenient mode)\n");
    c.push_str("    if hasattr(obj, '__dataclass_fields__'):\n");
    c.push_str("        d = {}\n");
    c.push_str("        for k in obj.__dataclass_fields__:\n");
    c.push_str("            if k in ('_set', '_choice', '_bit_count'):\n                continue\n");
    c.push_str("            v = getattr(obj, k)\n");
    c.push_str("            if v is None:\n                continue\n");
    c.push_str("            d[k] = _to_dict(v)\n");
    c.push_str("        return d\n");
    c.push_str("    return obj\n\n");
    // --- Strict serialization (checks _set, used by encode) ---
    c.push_str("def to_json_strict(obj) -> str:\n    return json.dumps(_to_dict_strict(obj))\n\n");
    c.push_str("def _to_dict_strict(obj) -> Any:\n");
    c.push_str("    if obj is None:\n        return None\n");
    c.push_str("    if isinstance(obj, bytes):\n        return obj.hex()\n");
    c.push_str("    if isinstance(obj, list):\n        return [_to_dict_strict(x) for x in obj]\n");
    c.push_str("    # BIT STRING with _bit_count: convert int to hex string\n");
    c.push_str("    if hasattr(obj, '_bit_count') and hasattr(obj, 'value'):\n");
    c.push_str("        return bit_string_hex(obj.value, obj._bit_count)\n");
    c.push_str("    # Newtype wrapper\n");
    c.push_str("    if hasattr(obj, '__dataclass_fields__'):\n");
    c.push_str("        fields = obj.__dataclass_fields__\n");
    c.push_str("        if set(fields.keys()) == {'value'}:\n");
    c.push_str("            return _to_dict_strict(obj.value)\n");
    c.push_str("    # CHOICE: only serialize the selected variant\n");
    c.push_str("    if hasattr(obj, '_choice') and obj._choice:\n");
    c.push_str("        v = getattr(obj, obj._choice, None)\n");
    c.push_str("        if v is not None:\n            return {obj._choice: _to_dict_strict(v)}\n");
    c.push_str("        return {}\n");
    c.push_str("    # Has _set: only include explicitly set fields\n");
    c.push_str("    if hasattr(obj, '_set'):\n");
    c.push_str("        d = {}\n");
    c.push_str("        for k in obj.__dataclass_fields__:\n");
    c.push_str("            if k in ('_set', '_choice', '_bit_count'):\n                continue\n");
    c.push_str("            if k in obj._set:\n");
    c.push_str("                d[k] = _to_dict_strict(getattr(obj, k))\n");
    c.push_str("        return d\n");
    c.push_str("    # No _set nor _choice: include all non-None\n");
    c.push_str("    if hasattr(obj, '__dataclass_fields__'):\n");
    c.push_str("        d = {}\n");
    c.push_str("        for k in obj.__dataclass_fields__:\n");
    c.push_str("            if k in ('_set', '_choice', '_bit_count'):\n                continue\n");
    c.push_str("            v = getattr(obj, k)\n");
    c.push_str("            if v is not None:\n                d[k] = _to_dict_strict(v)\n");
    c.push_str("        return d\n");
    c.push_str("    return obj\n\n");
    // --- Decode ---
    c.push_str("def _convert_bytes(val, target_type) -> Any:\n");
    c.push_str("    if val is None:\n        return None\n");
    c.push_str("    if target_type is bytes and isinstance(val, str):\n");
    c.push_str("        try:\n            return bytes.fromhex(val)\n");
    c.push_str("        except (ValueError, AttributeError):\n            pass\n");
    c.push_str("    return val\n\n");
    // --- BIT STRING hex conversion (matching Java CmsBase.bitStringHex) ---
    c.push_str("def bit_string_hex(value: int, num_bits: int) -> str:\n");
    c.push_str("    num_bytes = (num_bits + 7) // 8\n    result = 0\n");
    c.push_str("    for i in range(num_bits):\n");
    c.push_str("        if (value >> i) & 1:\n");
    c.push_str("            byte_pos = i // 8\n            bit_pos = 7 - (i % 8)\n");
    c.push_str("            result |= 1 << (byte_pos * 8 + bit_pos)\n");
    c.push_str("    return format(result, '0{}x'.format(num_bytes * 2))\n\n");
    c.push_str("def parse_bit_string_hex(hex_str: str, num_bits: int) -> int:\n");
    c.push_str("    raw = int(hex_str, 16)\n    result = 0\n");
    c.push_str("    for i in range(num_bits):\n");
    c.push_str("        byte_pos = i // 8\n        bit_pos = 7 - (i % 8)\n");
    c.push_str("        if (raw >> (byte_pos * 8 + bit_pos)) & 1:\n");
    c.push_str("            result |= 1 << i\n");
    c.push_str("    return result\n\n");
    c.push_str("def from_json(json_str: str, cls) -> Any:\n");
    c.push_str("    data = json.loads(json_str)\n    return _from_dict(data, cls)\n\n");
    c.push_str("def _from_dict(data, cls) -> Any:\n");
    c.push_str("    if data is None:\n        return None\n");
    c.push_str("    if hasattr(cls, '__dataclass_fields__'):\n");
    c.push_str("        fields = cls.__dataclass_fields__\n");
    c.push_str("        # BIT STRING: parse hex string back to int\n");
    c.push_str("        if hasattr(cls, '_bit_count') and 'value' in fields:\n");
    c.push_str("            hex_val = data['value'] if isinstance(data, dict) and 'value' in data else data\n");
    c.push_str("            return cls(value=parse_bit_string_hex(str(hex_val) if hex_val is not None else '0', cls._bit_count))\n");
    c.push_str("        # Newtype: always recursively wrap, even when data is scalar\n");
    c.push_str("        if set(fields.keys()) == {'value'}:\n");
    c.push_str("            val = data['value'] if isinstance(data, dict) and 'value' in data else data\n");
    c.push_str("            result = _from_dict(val, fields['value'].type)\n");
    c.push_str("            result = _convert_bytes(result, fields['value'].type)\n");
    c.push_str("            return cls(value=result)\n");
    c.push_str("        # Regular dataclass\n");
    c.push_str("        if isinstance(data, dict):\n");
    c.push_str("            kwargs = {}\n");
    c.push_str("            for name, field_type in fields.items():\n");
    c.push_str("                if name in ('_set', '_choice', '_bit_count'):\n                    continue\n");
    c.push_str("                if name in data:\n");
    c.push_str("                    val = _from_dict(data[name], field_type.type)\n");
    c.push_str("                    kwargs[name] = _convert_bytes(val, field_type.type)\n");
    c.push_str("            return cls(**kwargs)\n");
    c.push_str("    if isinstance(data, list):\n        return data\n");
    c.push_str("    return data\n");
    c
}

/// Generate __init__.py
pub fn gen_init(prefix: &str, package: &str) -> String {
    let _pkg = if package.is_empty() { prefix.to_lowercase() } else { package.replace('.', "_") };
    let mut c = String::new();
    c.push_str(&helpers::gen_header_comment());
    c.push_str("\nfrom ._native import encode, decode_raw\n");
    c.push_str("from ._base import to_json, to_json_strict, from_json, bit_string_hex, parse_bit_string_hex\n");
    c.push_str("\n__all__ = []\n");
    c
}

/// Generate pixi.toml
pub fn gen_pixi_toml(prefix: &str, package: &str) -> String {
    let pkg_name = if package.is_empty() {
        format!("{}-data", prefix.to_lowercase())
    } else {
        package.to_string()
    };
    format!(r#"[project]
name = "{pkg_name}"
version = "0.1.0"
channels = ["conda-forge"]
platforms = ["win-64", "linux-64", "osx-64"]

[tasks.test]
cmd = "pytest -v tests"
env = {{ PYTHONPATH = "src" }}

[dependencies]
python = ">=3.11"
pytest = ">=8"
pytest-xdist = "*"
"#)
}

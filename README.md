# Charles Shan's ASN.1 Generator (csasn1)

> [中文](./README_CN.md) | **English**

![ber](https://img.shields.io/badge/ber-Basic%20Encoding%20Rules-orange) ![der](https://img.shields.io/badge/der-Distinguished%20Encoding%20Rules-violet) ![aper](https://img.shields.io/badge/aper-Aligned%20Packed%20Encoding-green) ![uper](https://img.shields.io/badge/uper-Unaligned%20Packed%20Encoding-red)

Generate a Rust encode/decode dynamic library (`asn1.dll` / `libasn1.so`) and Java / Python bean classes from a single ASN.1 specification. All languages exchange data through one unified JSON intermediate representation with the native library — the business layer never touches bit-level operations.

> Notes
>
> - **Edit only the** **`.asn`**: do not hand-edit the generated `generated.rs` / `ffi_auto.rs`; the next generation overwrites them.
>
> - **Cross-platform dynamic library**: the generator copies `asn1.dll` (Windows) / `libasn1.so` (Linux/Mac) based on the current platform; on the Java side, `Native.load("asn1")` locates it by OS/arch automatically.
>
> - **`encode`** **returns a byte array, not hex (deliberate)**: `csasn1_encode`'s binary result is a JSON numeric array (`{"bytes": [1,2,3,...]}`). Do **not** "optimize" it back to hex — Jackson deserializes a JSON string into `byte[]` using **base64** by default (not hex), and hex characters happen to fall inside the base64 alphabet, silently decoding to wrong bytes (no error, wrong result). The numeric array is a `byte[]` representation natively supported by both Jackson and ctypes, with no ambiguity. Decode goes through JER and uses hex only because that is the Rust-side JER output, which is unaffected by this pitfall.

***

## 1. Commands

### 1.1 Quick start

Inside the `csasn1/` directory, put your spec in `specs/` (default `specs/dlt2811.asn`):

```powershell
just build          # build Rust: DLL + CLI (generates Rust types + FFI dispatch at build time)
just gen-java       # generate Java bean classes into assets/java
just gen-python     # generate Python package into assets/python
just gen-all        # build + generate all languages
```

> **Edit only your** **`.asn`** **file; everything else is automatic.** The generated `src/generated.rs` / `src/ffi_auto.rs` are overwritten on the next build — don't hand-edit.

***

### 1.2 Command line

```powershell
# Java (must specify --lang java)
cargo run --release -- --lang java --src specs/dlt2811.asn --dest assets/java --prefix Cms --enc aper --package com.example

# Python
cargo run --release -- --lang python --src specs/dlt2811.asn --dest assets/python --prefix Cms --enc aper --package com.example
```

Running with no arguments enters interactive mode, prompting for each parameter one by one.

| Parameter   | Default             | Description                                                                      |
| ----------- | ------------------- | -------------------------------------------------------------------------------- |
| `--lang`    | required            | Target language: `java` / `python` (alias `--bin`); must be specified explicitly |
| `--src`     | `specs/dlt2811.asn` | ASN.1 spec path; passing a `.asn` auto-maps to `src/generated.rs`                |
| `--dest`    | `java/src`          | Output directory (alias `--out`)                                                 |
| `--prefix`  | `Inner`             | Class-name prefix                                                                |
| `--enc`     | `ber`               | Encoding baked into generated code (`ber`/`der`/`aper`/`uper`)                   |
| `--package` | empty               | Java package name                                                                |

### 1.3 `just` commands

| Command                              | Purpose                                                                           |
| ------------------------------------ | --------------------------------------------------------------------------------- |
| `just build`                         | Build Rust (DLL + CLI)                                                            |
| `just gen-java`                      | Generate Java classes into `assets/java` (see justfile for `--prefix Inner` etc.) |
| `just test-java`                     | Generate Java + run Maven tests                                                   |
| `just test-java-one <Class>`         | Run a single Java test class                                                      |
| `just gen-python`                    | Generate Python package into `assets/python`                                      |
| `just test-python`                   | Generate Python + pixi tests                                                      |
| `just gen-all`                       | Build + generate all languages                                                    |
| `just rust-all`                      | List all ASN.1 type names                                                         |
| `just jer <type> <json>`             | Print a type's JER JSON via the example binary                                    |
| `just json <type>` / `just json-all` | Print JER "god format" using the preset test                                      |

***

## 2. How it works

### 2.1 Data flow

```
specs/dlt2811.asn ────────────────────（only input; edit only this）
        │ ① build.rs · rasn-compiler
        ▼
src/generated.rs ─────────────────────（common artifact: Rust types）
        │
        ├── ② build.rs · scan type names ──▶ src/ffi_auto.rs ──▶ asn1.dll
        │
        └── ③ main.rs · syn parse AST ────▶ Java / Python bean classes
```

The diagram annotations are in Chinese (`①`, `②`, `③` match the steps below; `·` separates two words). At runtime (Java example): encode = Java object → JSON → `csasn1_encode` → ASN.1 binary; decode is the reverse. **All languages exchange data through JSON as the intermediate representation and never touch bit operations.**

### 2.2 Generation steps

**① build.rs** — at build time runs two steps automatically: `rasn-compiler` compiles the `.asn` into `generated.rs`; then it scans type names and generates `csasn1_encode/decode` `match` branches for each type into `ffi_auto.rs` (including the Jackson↔JER adapter).

**② lib.rs → dynamic library** — exports 4 C functions (see `asn1.def`):

```c
char* csasn1_ping(void);                                          // returns "pong"
char* csasn1_encode(const char* type_name, const char* encoding, const char* json);
char* csasn1_decode(const char* type_name, const char* encoding, const uint8_t* data, size_t len);
void  csasn1_free_string(char* s);                                // frees the returned JSON string
```

Returned JSON: encode → `{"ok": true, "bytes": [...]}`, decode → `{"ok": true, "value": ...}`.

**③ main.rs** — CLI code-generator entry: parses the `generated.rs` AST → `extract` produces `Vec<TypeInfo>` (collecting `*_default` function bodies, analyzing newtype/struct/enum, parsing rasn attributes); it also reads the `.asn` text to extract type definitions and BIT STRING/ENUMERATED named constants to aid generation; then dispatches by `TypeKind` to the Java / Python generators.

### 2.3 Directory structure

```
csasn1/
├── specs/xxx.asn         ← your ASN.1 spec (edit only this)
├── build.rs              ← generates generated.rs + ffi_auto.rs at build time
├── asn1.def              ← Windows DLL export table (4 functions)
├── src/
│   ├── main.rs           ← CLI (code-generator entry: syn parse → extract → dispatch)
│   ├── lib.rs            ← library entry (cdylib, builds as asn1.dll); #[cfg(test)] hosts tests/
│   ├── generated.rs      ← auto-generated (Rust types, don't hand-edit)
│   ├── ffi_auto.rs       ← auto-generated (FFI dispatch, don't hand-edit)
│   ├── generator/
│   │   ├── mod.rs        ← generator module tree declaration + public re-exports
│   │   ├── asn1_parse.rs ← parse .asn text: type definitions + named constants
│   │   ├── extract.rs    ← syn parse generated.rs → Vec<TypeInfo>
│   │   ├── model.rs      ← data model TypeInfo/FieldInfo/VariantInfo/TypeKind
│   │   ├── java/         ← Java generator (multi-layer + JSON-driven + templated, see below)
│   │   └── python/       ← Python generator (assembled by gen_type.rs)
│   └── tests/            ← pure-Rust APER/JER round-trip tests
├── examples/jer_god.rs   ← JER "god format" debug example
├── scripts/list_types.ps1← list all generated types
└── assets/               ← standalone test artifacts (java / python, can be regenerated)
```

### 2.4 Generated output — Java (`generator/java/`)

Each type gets a `<Prefix>*` POJO (data kept in `_v`) + `<Prefix>Base` (base class, `encode()`/`decode()`) + `<Prefix>Native` (JNA calls into the DLL) + `V` (`_v` helper) + built-in default wrapper classes + auto JUnit tests + `pom.xml`. `encode()` encodes the whole `_v`; `encodeTest()` prints the intermediate JSON for debugging. Runtime deps: Jackson + JNA.

> The generator itself was refactored from "single-file dispatch" into "**top-level dispatch → per-type generators → shared reuse layer + JSON-driven + templated**":

| Layer                   | File                                            | Responsibility                                                                                                                                          |
| ----------------------- | ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Top-level orchestration | `java/mod.rs`                                   | `JavaConfig` + `generate()`: create Maven layout, iterate types into class\_gen + test\_gen, write `Native/Base/V`, built-in default wrappers, copy DLL |
| Dispatch                | `java/class_gen.rs`                             | Assemble the class header by `TypeKind`, then fan out to the three generators                                                                           |
| Per-type generator      | `gen_newtype.rs`                                | Newtype (bitstring / unsigned-u32 / regular three branches)                                                                                             |
| <br />                  | `gen_struct.rs`                                 | SEQUENCE/STRUCT: per-field default-vs-sample, fill `_v` in constructor, `sample()`                                                                      |
| <br />                  | `gen_choice.rs`                                 | CHOICE: named\_consts, no-arg constructor, isChoice guard, `@JsonSetter`, decode                                                                        |
| Shared reuse layer      | `gen_newtype_common.rs`                         | Shared by all three: encode/decode strategy, template filling, cycle detection, default/sample value dispatch                                           |
| JSON-driven             | `type_map.json`                                 | Declarative mapping: Rust type → Java type + encode/decode + the two default sets (default/sample)                                                      |
| <br />                  | `type_registry.rs`                              | Loads JSON at compile time (`include_str!`), longest-prefix match for exact + prefix                                                                    |
| <br />                  | `type_map.rs`                                   | Rust→Java type resolution (unwraps Option/SequenceOf/Vec/Box; checks registry first, then falls back to Newtype)                                        |
| Templates               | `templates/*.txt`                               | 12 fixed skeletons (constructor / encode / decode / sample, etc.)                                                                                       |
| Test generation         | `test_gen.rs` + `test_newtype/struct/choice.rs` | Generate JUnit round-trip tests per type                                                                                                                |

**Two sets of defaults (default vs sample)** — `TypeSpec.default` / `TypeSpec.sample` respectively:

- `default`: the "unset" state initialized by the constructor; Jackson-safe (`""`/`0`/`null`).

- `sample`: the **non-zero, SIZE-compliant** fill used by the `sample()` test factory (FixedBitString filled with `1`, length taken from SIZE).

- **Template side**: `ctor_bitstring` / `ctor_unsigned` = special constructor blocks; `encode_plain` (no try/catch) vs `encode_wrapped` (try/catch to shield MAPPER errors); `decode` = static decode skeleton; `sample_factory` (single value) / `sample_factory_puts` (multiple fields) / `sample_factory_choice` (with `_choice`) — three sample factories.

**type\_map.json exact / prefix** — the `exact` table maps simple built-in types (bool/u8/i32/String/()…); the `prefix` table matches constrained container types by **longest prefix** (Empty/VisibleString/OctetString/FixedBitString/Integer…). Extracting fixed mappings into data means changing a mapping doesn't require recompiling Rust.

### 2.5 Generated output — Python (`generator/python/`)

`_native.py` (ctypes) + `_base.py` + `_types.py` (dataclasses, topologically sorted) + `__init__.py` + auto tests + `pixi.toml`. The generator is assembled by a single `gen_type.rs` and is structurally simpler than the Java side.

### 2.6 Testing

Rust side (`src/tests/`): `lib.rs` hosts test submodules under `#[cfg(test)]`; once `build.rs` has produced `generated.rs`, run them from `csasn1/`:

| Command                                    | Contents                                                                                                         |
| ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| `cargo test data_scalars`                  | Round-trip of assorted scalars (float, utctime, binarytime, octet/visible/unicode/bit string, boolean, int8..64) |
| `cargo test data_compound`                 | Compound/array round-trip + FFI-adapter pipeline (Jackson JSON→JER→APER→JER) + DEFAULT survival check            |
| `cargo test complex_types`                 | Complex-type round-trip (notably FixedBitString, JER→APER→JER)                                                   |
| `cargo test jer_god_format -- --nocapture` | Print the JER "god format" reference output for each type                                                        |

> These are **pure-Rust** assertions; the Java/Python round-trips are driven independently by each `assets/*` Maven / pixi setup.

Generated per-language tests

```powershell
just test-java     # assets/java mvn test
just test-python   # assets/python pixi run test
```

### 2.7 Integration with an existing project

Java

1. Copy the generated `assets/java/src/main/java/<your package>/*.java` into your source tree
2. Deploy the native dynamic library (Windows `asn1.dll`, Linux/Mac `libasn1.so`) to a classpath resource (including the JNA platform directory)
3. Add Jackson + Lombok + JNA dependencies
4. Encode/decode with `<Prefix>*.encode()` / `.decode()`

Python

```bash
pip install -e assets/python          # or
cd assets/python && pixi install && pixi run test
```

***

## 3. Tech stack

- [rasn](https://github.com/librasn/rasn) — Rust ASN.1 codec framework

- [rasn-compiler](https://github.com/librasn/compiler) — ASN.1 → Rust code generator

- [syn](https://github.com/dtolnay/syn) — Rust code parsing

- [JNA](https://github.com/java-native-access/jna) — Java native access

- [Jackson](https://github.com/FasterXML/jackson) — Java JSON serialization

- [Lombok](https://projectlombok.org/) — Java boilerplate reduction

- [pixi](https://pixi.sh/) — Python package manager

- [ctypes](https://docs.python.org/3/library/ctypes.html) — Python native calls


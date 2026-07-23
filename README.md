# csasn1 — ASN.1 编解码工具链

从 ASN.1 规约文件自动生成 Rust 编解码库 + 多语言绑定。

## 架构

```
你的 .asn1 文件
      │
      ▼  cargo build
┌──────────────────────┐
│  build.rs            │
│  ① rasn-compiler     │──→ src/generated.rs (Rust 类型)
│  ② 类型扫描          │──→ src/ffi_auto.rs   (FFI 分发)
│                      │──→ target/debug/csasn1.dll
└──────────────────────┘
      │
      ▼  cargo run --bin csasn1
┌──────────────────────┐
│  csasn1             │──→ java/src/Cms*.java (Java 类)
│  用 syn 解析         │    每个 ASN.1 类型一个类
│  生成 Jackson POJO   │    含 encode() / decode()
└──────────────────────┘
```

## 快速开始

```powershell
# 编译全部（DLL + CLI + 代码生成器）
cargo build

# 运行 CLI 演示
cargo run --bin cscli

# 生成 Java 类（默认 Cms 前缀、ber 编码）
cargo run --bin csasn1
```

```powershell
cargo run --bin csasn1 -- --src ./specs/dlt2811.asn --dest ./java/src --prefix Asn --enc per --package com.example.asn1
```

编译成命令
```powershell
# 编译并生成软连接
cargo build --release; $env:PATH = "$pwd\target\release;$env:PATH"

# 之后就能直接用了
./csasn1 --src ./specs/dlt2811.asn --dest ./java/src --prefix Asn --enc per

# 拷到任何地方都能跑
copy target\release\csasn1.exe D:\tools\
D:\tools\csasn1.exe --prefix MyPfx
```

```bash
csasn1.exe --src ./specs/dlt2811.asn --dest "D:\project\work\standard\dlt2811bean\cms\jcms\jcms-data\src\main\java\com\ysh\jcms\data" --prefix Cms --enc per --package com.ysh.jcms.data
```

## 项目结构

```
csasn1/
├── specs/dlt2811.asn      ← 你的 ASN.1 规约文件（只改这个）
├── build.rs               ← 编译时自动生成 Rust 类型 + FFI
├── Cargo.toml
├── src/
│   ├── main.rs            ← CLI 演示入口
│   ├── lib.rs             ← 库入口（编译为 csasn1.dll）
│   ├── ffi_auto.rs        ← 自动生成的 FFI 分发代码
│   ├── generated.rs       ← 自动生成的 Rust 类型
│   └── bin/csasn1.rs    ← Java 类生成器
├── java/src/              ← 生成的 Java 类（cargo run --bin csasn1 产出）
│   ├── CmsNative.java     ← JNA 桥接（加载 DLL，encode/decode）
│   └── Cms*.java          ← 每个 ASN.1 类型一个类（304+ 个）
└── python/
    └── csasn1_example.py  ← Python ctypes 调用示例
```

## 工作流

### 当你修改 .asn1 文件后

```powershell
# 第 1 步：编译 DLL
cargo build
# → src/generated.rs 自动更新
# → src/ffi_auto.rs 自动更新
# → csasn1.dll 重新生成

# 第 2 步：生成 Java 类
cargo run --bin csasn1 -- --prefix Asn --enc per --src ./specs/dlt2811.asn --package com.example.asn1
# → java/src/Asn*.java 全部重新生成，包名 com.example.asn1
```

**只改 `.asn1` 文件，其余全自动。**

## csasn1 — Code 代码生成器

从 `src/generated.rs`（由 `build.rs` 从 ASN.1 规约自动生成）生成 Java POJO 类。

```powershell
# 完整参数示例
cargo run --bin csasn1 -- --prefix Asn --enc per --src ./specs/dlt2811.asn --out java/src --package com.example.asn1

# 最短用法（全部走默认值）
cargo run --bin csasn1
```

### 参数说明

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--src <path>` | `specs/dlt2811.asn` | ASN.1 规约文件路径，传 `.asn` 自动映射到 `src/generated.rs` |
| `--out <dir>` | `java/src` | Java 源码输出目录 |
| `--prefix <str>` | `Cms` | Java 类名前缀，如 `Asn` → `AsnBoolean.java` |
| `--enc <str>` | `ber` | 生成时固定的编码方式（`ber`/`der`/`aper`/`uper`），Java 类不再需要手动传编码参数 |
| `--package <str>` | （空） | Java 包名，如 `com.example.asn1` → 文件头生成 `package com.example.asn1;` |

### 生成的 Java 类

每个 ASN.1 类型一个文件，格式统一：

```java
// Auto-generated. ASN.1 type: Boolean

package com.example.asn1;

public class AsnBoolean {
    public int value;

    public byte[] encode();                  // 严格编码（生产用）
    public byte[] encodeTest();              // 宽松编码（测试用）
    public static AsnBoolean decode(byte[] data);  // 解码（编码方式由生成时固定）
}
```

## FFI 接口

DLL 导出 3 个 C 函数，所有语言共用：

```c
// JSON → 编码 → 二进制
Buffer csasn1_encode(const char* type_name,
                     const char* encoding,
                     const char* json);

// 二进制 → 解码 → JSON
Buffer csasn1_decode(const char* type_name,
                     const char* encoding,
                     const uint8_t* data,
                     size_t len);

// 释放 Rust 分配的内存
void csasn1_free_buffer(Buffer buf);
```

### 支持的编码方式

| 参数 | 编码规则 | 特点 |
|------|----------|------|
| `"ber"` | Basic Encoding Rules | 通用，可读 |
| `"der"` | Distinguished Encoding Rules | 确定性编码 |
| `"aper"` | Aligned Packed Encoding Rules | 紧凑，对齐 |
| `"uper"` | Unaligned Packed Encoding Rules | 最紧凑 |

## Java 使用

```java
// 构造 Java 对象
CmsApdu apdu = new CmsApdu();
apdu.apch = new CmsApch();
apdu.apch.cc = new CmsControlCode();
apdu.apch.cc.resp = true;
apdu.asdu = hexToBytes("01020304");

// 严格编码（生产用）— 只编码显式设过的 OPTIONAL 字段
byte[] per = apdu.encode();

// 宽松编码（测试用）— 全部 OPTIONAL 字段都编码
byte[] perTest = apdu.encodeTest();

// 解码（编码方式由生成时固定，无需指定）
CmsApdu recv = CmsApdu.decode(per);
```

需要 Jackson 依赖：
```xml
<dependency>
    <groupId>com.fasterxml.jackson.core</groupId>
    <artifactId>jackson-databind</artifactId>
    <version>2.17.0</version>
</dependency>
```

## Python 使用

```python
from csasn1_example import encode, decode

encoded = encode("Apdu", "aper", {
    "apch": {"cc": {"resp": True}, "sc": 0x51, "fl": 0},
    "asdu": "01020304"
})
decoded = decode("Apdu", "aper", encoded)
```

## 你的现有项目集成

如果你已有 Java 项目（如 `dlt2811bean/cms`）：

1. 复制 `java/src/*.java` 到你的项目源码目录
2. 复制 `target/debug/csasn1.dll` 到 `resources/win32-x86-64/`
3. 添加 Jackson 依赖
4. 用 `CmsApdu.encode()` / `CmsApdu.decode()` 替代旧的编解码调用

## 技术栈

- [rasn](https://github.com/librasn/rasn) — Rust ASN.1 编解码框架
- [rasn-compiler](https://github.com/librasn/compiler) — ASN.1 → Rust 代码生成器
- [syn](https://github.com/dtolnay/syn) — Rust 代码解析
- [JNA](https://github.com/java-native-access/jna) — Java 原生调用
- [Jackson](https://github.com/FasterXML/jackson) — Java JSON 序列化

## TODO / 改进方向

### Java 类质量提升

- [ ] **Builder 模式** — 为复杂类型生成 Builder，代替多个 setter 调用
- [ ] **精确 OPTIONAL presence 控制** — 每个 POJO 内建 `_set` 追踪（记录哪些字段通过 setter 设过值）。`encode()` 为严格模式，只编码 `_set` 中的 OPTIONAL 字段；`encodeTest()` 为宽松模式，全部字段都编码，方便测试向后兼容
- [ ] **不可变对象变体** — 可选生成 record 或不可变类，适合线程安全场景
- [ ] **泛化编解码接口** — `Codec.aper().encode(obj)` 代替每个类的静态方法
- [ ] **Jackson 注解** — 生成 `@JsonProperty("fc")`、`@JsonInclude` 等注解，开箱即用与 REST API 集成

### 性能

- [ ] **批量 FFI 调用** — 多个 PDU 一次 JNI 调用，减少跨语言开销
- [ ] **直接 ByteBuffer** — 跳过 JSON 中间表示，直接传二进制 buffer

### 测试

- [ ] **约束感知的随机数据生成** — 根据 SIZE 范围、permitted alphabet 自动生成合法的随机测试数据
- [ ] **边界值测试** — 自动生成 MIN/MAX/空/超长等边界用例

### 文档与可追溯性

- [ ] **ASN.1 注释 → Javadoc** — 将 ASN.1 规约中的注释携带到生成的 Java 类中
- [ ] **生成版本标记** — 标明由哪个 ASN.1 文件、什么时间、哪个生成器版本生成

# csasn1 — ASN.1 编解码工具链

从 ASN.1 规约文件自动生成 Rust 编解码库 + 多语言绑定（Java / Python）。

## 架构

```
你的 .asn1 文件
      │
      ▼  cargo build
┌──────────────────────┐
│  build.rs            │
│  ① rasn-compiler     │──→ src/generated.rs (Rust 类型)
│  ② 类型扫描          │──→ src/ffi_auto.rs   (FFI 分发)
│                      │──→ target/release/asn1.dll
└──────────────────────┘
      │
      ▼  csasn1 (代码生成器)
┌──────────────────────┐
│  --lang java         │──→ assets/java/   (Java POJO + JNA)
│  --lang python       │──→ assets/python/ (Python dataclass + ctypes)
│  统一 JSON 中间表示   │    所有语言通过 JSON ↔ 原生 DLL 交换数据
└──────────────────────┘
```

## 快速开始

```powershell
# 编译 Rust DLL + CLI
just build

# 生成 Java 类
just gen-java

# 生成 Python 包
just gen-python

# 全部一起
just gen-all
```

## just 命令

| 命令 | 作用 |
|------|------|
| `just build` | 编译 Rust（DLL + CLI） |
| `just gen-java` | 生成 Java 类（assets/java/） |
| `just gen-python` | 生成 Python 包（assets/python/） |
| `just gen-all` | 编译 + 生成全部语言 |
| `just test-java` | 生成 Java + 运行 Maven 测试 |
| `just test-python` | 生成 Python + 运行 pytest |
| `just test-java-one <TestClass>` | 运行单个 Java 测试 |

## 项目结构

```
csasn1/
├── specs/dlt2811.asn      ← 你的 ASN.1 规约文件（只改这个）
├── build.rs               ← 编译时自动生成 Rust 类型 + FFI
├── justfile                ← 常用命令
├── src/
│   ├── main.rs             ← CLI 入口（代码生成器）
│   ├── lib.rs              ← 库入口（编译为 asn1.dll）
│   ├── ffi_auto.rs         ← 自动生成的 FFI 分发代码
│   ├── generated.rs        ← 自动生成的 Rust 类型
│   └── generator/
│       ├── java/           ← Java 代码生成器
│       └── python/         ← Python 代码生成器
├── assets/
│   ├── java/               ← 生成的 Java 类（JNA + Jackson）
│   │   ├── pom.xml
│   │   └── src/main/java/.../Cms*.java
│   └── python/             ← 生成的 Python 包（ctypes + dataclass）
│       ├── pixi.toml
│       ├── src/cms_data/
│       └── tests/
└── patches/                ← （已删除，修复已合入 rasn fork）
```

## 工作流

### 当你修改 .asn1 文件后

```powershell
# 编译 DLL
cargo build
# → src/generated.rs 自动更新
# → src/ffi_auto.rs 自动更新
# → asn1.dll 重新生成

# 重新生成 Java 类
just gen-java

# 重新生成 Python 包
just gen-python
```

**只改 `.asn1` 文件，其余全自动。**

## csasn1 — 代码生成器

从 `src/generated.rs`（由 `build.rs` 从 ASN.1 规约自动生成）生成 Java/Python 绑定。

```powershell
# Java（默认）
cargo run --release -- --lang java --src specs/dlt2811.asn --dest assets/java --prefix Cms --enc aper --package com.ysh.jcms.data

# Python
cargo run --release -- --lang python --src specs/dlt2811.asn --dest assets/python --prefix Cms --enc aper --package com.ysh.jcms.data
```

### 参数说明

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--lang <str>` | `java` | 目标语言（`java` / `python`） |
| `--src <path>` | `specs/dlt2811.asn` | ASN.1 规约文件路径，传 `.asn` 自动映射到 `src/generated.rs` |
| `--dest <dir>` | `java/src` | 输出目录 |
| `--prefix <str>` | `Cms` | 类名前缀 |
| `--enc <str>` | `ber` | 生成时固定的编码方式 |
| `--package <str>` | （空） | 包名 |

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
import com.ysh.jcms.data.*;

// 构造 Java 对象（链式赋值）
CmsApdu apdu = new CmsApdu();
apdu.apch(new CmsApch().cc(new CmsControlCode().resp(true)));

// 严格编码（生产用）— 只编码 _set 中标记过的 OPTIONAL 字段
byte[] per = apdu.encode();

// 宽松编码（测试用）— 全部字段都编码
byte[] perTest = apdu.encodeTest();

// 解码（编码方式由生成时固定，无需指定）
CmsApdu recv = CmsApdu.decode(per);
```

需要 Jackson + Lombok + JNA 依赖：
```xml
<dependency>
    <groupId>com.fasterxml.jackson.core</groupId>
    <artifactId>jackson-databind</artifactId>
    <version>2.17.0</version>
</dependency>
<dependency>
    <groupId>org.projectlombok</groupId>
    <artifactId>lombok</artifactId>
    <version>1.18.36</version>
    <scope>provided</scope>
</dependency>
<dependency>
    <groupId>net.java.dev.jna</groupId>
    <artifactId>jna</artifactId>
    <version>5.14.0</version>
</dependency>
```

## Python 使用

```python
from cms_data import CmsApdu, CmsApch

# 构造 Python 对象
apdu = CmsApdu()
apdu.apch = CmsApch()
apdu.apch.cc.resp = True

# 编码为 APER 字节
data = apdu.encode()

# 解码
decoded = CmsApdu.decode(data)
```

需要安装生成的 Python 包（pixi 或 pip）：
```bash
# 使用 pixi
cd assets/python
pixi install
pixi run test

# 使用 pip
pip install -e assets/python
```

## 与现有项目集成

### Java 项目（如 `dlt2811bean/cms`）

1. 复制 `assets/java/src/main/java/com/ysh/jcms/data/*.java` 到你的项目
2. 复制 `target/release/asn1.dll` 到 `resources/win32-x86-64/`
3. 添加 Jackson + Lombok + JNA 依赖
4. 用 `CmsApdu.encode()` / `CmsApdu.decode()` 替代旧的编解码调用

### Python 项目

```bash
pip install -e assets/python
# 或
cd assets/python && pixi install
```

## 技术栈

- [rasn](https://github.com/librasn/rasn) — Rust ASN.1 编解码框架
- [rasn-compiler](https://github.com/librasn/compiler) — ASN.1 → Rust 代码生成器
- [syn](https://github.com/dtolnay/syn) — Rust 代码解析
- [JNA](https://github.com/java-native-access/jna) — Java 原生调用
- [Jackson](https://github.com/FasterXML/jackson) — Java JSON 序列化
- [Lombok](https://projectlombok.org/) — Java 样板代码消除
- [pixi](https://pixi.sh/) — Python 包管理
- [ctypes](https://docs.python.org/3/library/ctypes.html) — Python 原生调用

## TODO / 改进方向

### Java 类质量提升

- [x] **精确 OPTIONAL presence 控制** — 每个 POJO 内建 `_set` 追踪。`encode()` 为严格模式，`encodeTest()` 为宽松模式
- [x] **Fluent setter（链式调用）** — 通过 Lombok `@Accessors(fluent=true, chain=true)` 支持 `obj.foo(val).bar(val2)`
- [ ] **泛化编解码接口** — `Codec.aper().encode(obj)` 代替每个类的静态方法
- [ ] **Jackson 注解** — 生成 `@JsonProperty("fc")`、`@JsonInclude` 等注解

### Python 生成器

- [ ] **OPTIONAL presence 控制** — Python dataclass 的 `_set` 追踪完善
- [ ] **CHOICE 类型的 JSON 序列化** — 正确处理变体选择
- [ ] **pixi 环境集成** — 自动复制 asn1.dll 到包目录

### 性能

- [ ] **批量 FFI 调用** — 多个 PDU 一次 JNI/ctypes 调用，减少跨语言开销
- [ ] **直接 ByteBuffer** — 跳过 JSON 中间表示，直接传二进制 buffer

### 测试

- [ ] **约束感知的随机数据生成** — 根据 SIZE 范围、permitted alphabet 自动生成合法的随机测试数据
- [ ] **边界值测试** — 自动生成 MIN/MAX/空/超长等边界用例

### 文档与可追溯性

- [x] **生成版本标记** — 每个生成的文件包含 `csasn1 vX.Y.Z at YYYY-MM-DD HH:MM UTC`

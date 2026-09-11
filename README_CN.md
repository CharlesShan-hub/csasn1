# Charles Shan's ASN.1 Generator (csasn1)

> **[English](./README.md) | 中文**

![ber](https://img.shields.io/badge/ber-Basic%20Encoding%20Rules-orange) ![der](https://img.shields.io/badge/der-Distinguished%20Encoding%20Rules-violet) ![aper](https://img.shields.io/badge/aper-Aligned%20Packed%20Encoding-green) ![uper](https://img.shields.io/badge/uper-Unaligned%20Packed%20Encoding-red)

从一份 ASN.1 规约自动产出 **Rust 编解码动态库（`asn1.dll` / `libasn1.so`）** 及 **Java / Python 的 Bean 类**。所有语言通过统一 JSON 中间表示与原生库交换数据，业务侧完全不碰位操作。

> 注意事项
>
> - **只改** **`.asn`**：生成的 `generated.rs` / `ffi_auto.rs` 不要手动编辑，下次生成会被覆盖。
>
> - **跨平台动态库**：生成器按当前平台拷贝 `asn1.dll`（Windows）/ `libasn1.so`（Linux/Mac）；Java 侧 `Native.load("asn1")` 按 OS/架构自动定位。
>
> - **`encode`** **返回字节数组而非 hex（刻意设计）**：`csasn1_encode` 的二进制结果用 JSON 数字数组（`{"bytes": [1,2,3,...]}`），**不要**"优化"回 hex——Jackson 把 JSON 字符串反序列化成 `byte[]` 时默认按 **base64** 解析（非 hex），而 hex 字符恰好落在 base64 字母表内，会**静默解出错误字节**（不报错但结果错）。数字数组是 Jackson / ctypes 都原生支持的 `byte[]` 表示，无歧义。decode 方向走 JER 用 hex 是因为那是 Rust 侧 JER 编码输出，不受此坑影响。

***

## 1. 命令介绍

### 1.1 快速开始

在 `csasn1/` 目录内，把要用的规约放到 `specs/`（默认是 `specs/dlt2811.asn`；本项目把规约放在生成器身边——说明见 `docs/asn1.md`）：

```powershell
just build          # 编译 Rust：DLL + CLI（构建期自动生成 Rust 类型 + FFI 分发）
just gen-java       # 生成 Java Bean 类到 assets/java
just gen-python     # 生成 Python 包到 assets/python
just gen-all        # build + 生成全部语言
```

> **只改你的** **`.asn`** **文件，其余全自动。** 生成的 `src/generated.rs` / `src/ffi_auto.rs` 由下次构建覆盖，勿手动编辑。

***

### 1.2 命令行

```powershell
# Java（需指定 --lang java）
cargo run --release -- --lang java --src specs/dlt2811.asn --dest assets/java --prefix Cms --enc aper --package com.example

# Python
cargo run --release -- --lang python --src specs/dlt2811.asn --dest assets/python --prefix Cms --enc aper --package com.example
```

不传参数进入交互式模式，逐项提问参数。

| 参数          | 默认                  | 说明                                           |
| ----------- | ------------------- | -------------------------------------------- |
| `--lang`    | 必填                 | 目标语言：`java` / `python`（别名 `--bin`），必须显式指定   |
| `--src`     | `specs/dlt2811.asn` | ASN.1 规约路径；传 `.asn` 自动映射到 `src/generated.rs` |
| `--dest`    | `java/src`          | 输出目录（别名 `--out`）                             |
| `--prefix`  | `Inner`             | 生成类名前缀                                       |
| `--enc`     | `ber`               | 固化进生成代码的编码方式（`ber`/`der`/`aper`/`uper`）      |
| `--package` | 空                   | Java 包名                                      |

### 1.3 just 命令

| 命令                                 | 作用                                                       |
| ---------------------------------- | -------------------------------------------------------- |
| `just build`                       | 编译 Rust（DLL + CLI）                                       |
| `just gen-java`                    | 生成 Java 类到 `assets/java`（`--prefix Inner` 等参数见 justfile） |
| `just test-java`                   | 生成 Java + 跑 Maven 测试                                     |
| `just test-java-one <Class>`       | 跑单个 Java 测试类                                             |
| `just gen-python`                  | 生成 Python 包到 `assets/python`                             |
| `just test-python`                 | 生成 Python + pixi 测试                                      |
| `just gen-all`                     | 编译 + 生成全部语言                                              |
| `just rust-all`                    | 列出所有 ASN.1 类型名                                           |
| `just jer <类型> <json>`             | 用 example 二进制打印某类型的 JER JSON                             |
| `just json <类型>` / `just json-all` | 用预置测试打印 JER "上帝格式"                                       |

***

## 2. 工作原理

### 2.1 数据流

```
specs/dlt2811.asn ────────────────────（唯一输入，只改这个）
        │ ① build.rs · rasn-compiler
        ▼
src/generated.rs ─────────────────────（Rust 类型，公共产物）
        │
        ├── ② build.rs · 扫描类型名 ────▶ src/ffi_auto.rs ──▶ asn1.dll
        │
        └── ③ main.rs · syn 解析 AST ────▶ Java / Python Bean 类
```

运行时（以 Java 为例）：编码 = Java 对象 → JSON → `csasn1_encode` → ASN.1 二进制；解码相反。**所有语言通过 JSON 作为中间表示交换数据，完全不碰位操作。**

### 2.2  生成环节

**① build.rs** —— 编译期自动两步：`rasn-compiler` 把 `.asn` 编译成 `generated.rs`；再扫描类型名，为每个类型生成 `csasn1_encode/decode` 的 match 分支到 `ffi_auto.rs`（含 Jackson↔JER 适配）。

**② lib.rs → 动态库** —— 导出 4 个 C 函数（见 `asn1.def`）：

```c
char* csasn1_ping(void);                                          // 返回 "pong"
char* csasn1_encode(const char* type_name, const char* encoding, const char* json);
char* csasn1_decode(const char* type_name, const char* encoding, const uint8_t* data, size_t len);
void  csasn1_free_string(char* s);                                // 释放返回的 JSON 字符串
```

返回 JSON：encode → `{"ok": true, "bytes": [...]}`，decode → `{"ok": true, "value": ...}`。

**③ main.rs** —— CLI 代码生成器入口：解析 `generated.rs` 的 AST → `extract` 产出 `Vec<TypeInfo>`（收集 `*_default` 函数体、分析 newtype/struct/enum、parse rasn 属性）；同时从 `.asn` 文本提取类型定义与 BIT STRING/ENUMERATED 命名常量辅助生成；按 `TypeKind` 分派到 Java / Python 生成器。

### 2.3 目录结构

```
csasn1/
├── specs/xxx.asn         ← 你的 ASN.1 规约（只改这个）
├── build.rs              ← 编译期自动生成 generated.rs + ffi_auto.rs
├── asn1.def              ← Windows DLL 导出符号表（4 个函数）
├── src/
│   ├── main.rs           ← CLI（代码生成器入口：syn 解析 → extract → 分发）
│   ├── lib.rs            ← 库入口（cdylib，编译为 asn1.dll）；#[cfg(test)] 收纳 tests/
│   ├── generated.rs      ← 自动生成（Rust 类型，勿手动编辑）
│   ├── ffi_auto.rs       ← 自动生成（FFI 分发，勿手动编辑）
│   ├── generator/
│   │   ├── mod.rs        ← 生成器模块树声明 + 公共 re-export
│   │   ├── asn1_parse.rs ← 解析 .asn 文本：类型定义 + 命名常量
│   │   ├── extract.rs    ← syn 解析 generated.rs → Vec<TypeInfo>
│   │   ├── model.rs      ← 数据模型 TypeInfo/FieldInfo/VariantInfo/TypeKind
│   │   ├── java/         ← Java 生成器（多层 + JSON 驱动 + 模板化，见下）
│   │   └── python/       ← Python 生成器（gen_type.rs 拼接）
│   └── tests/            ← 纯 Rust 端 APER/JER 往返测试
├── examples/jer_god.rs   ← JER "上帝格式" 调试示例
├── scripts/list_types.ps1← 列出所有生成类型
└── assets/               ← 独立测试产物（java / python，可重新生成）
```

### 2.4 生成产物 - Java（`generator/java/`）

每个类型一个 `<前缀>*` POJO（数据统一存 `_v`）+ `<前缀>Base`（基类，`encode()`/`decode()`）+ `<前缀>Native`（JNA 调 DLL）+ `V`（`_v` 助手）+ 内建默认包装类 + 自动 JUnit 测试 + `pom.xml`。`encode()` 全量编码 `_v`，`encodeTest()` 打印中间 JSON 供调试。运行依赖 Jackson + JNA。

> 生成器本身由「单文件派发」重构为「**顶层派发 → 按类型生成器 → 共享复用层 + JSON 驱动 + 模板化**」：

| 层级      | 文件                                              | 职责                                                                                                   |
| ------- | ----------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| 顶层编排    | `java/mod.rs`                                   | `JavaConfig` + `generate()`：建 Maven 目录、遍历类型调 class\_gen + test\_gen、写 `Native/Base/V`、内建默认包装类、拷贝 DLL |
| 派发层     | `java/class_gen.rs`                             | 按 `TypeKind` 拼类头后分发给三个生成器                                                                            |
| 专属生成器   | `gen_newtype.rs`                                | Newtype（bitstring / unsigned-u32 / 常规 三分支）                                                           |
| <br />  | `gen_struct.rs`                                 | SEQUENCE/STRUCT：字段级 default-vs-sample、构造器填 `_v`、`sample()`                                           |
| <br />  | `gen_choice.rs`                                 | CHOICE：named\_consts、无参构造器、isChoice 守卫、`@JsonSetter`、decode                                          |
| 共享复用层   | `gen_newtype_common.rs`                         | 三方复用：encode/decode 策略、模板填充、循环引用检测、default/sample 值分发                                                 |
| JSON 驱动 | `type_map.json`                                 | 声明性映射：Rust类型 → Java类型 + 编解码 + default/sample 两套默认值                                                   |
| <br />  | `type_registry.rs`                              | 编译期加载 JSON（`include_str!`），exact + prefix 最长优先匹配                                                     |
| <br />  | `type_map.rs`                                   | Rust类型→Java类型解析（Option/SequenceOf/Vec/Box 解包，先查 registry 再回退 Newtype）                                |
| 模板      | `templates/*.txt`                               | 12 个固定骨架（构造器 / encode / decode / sample 等）                                                           |
| 测试生成    | `test_gen.rs` + `test_newtype/struct/choice.rs` | 按类型生成 JUnit 往返测试                                                                                     |

**两套默认值（default vs sample）** —— `TypeSpec.default` / `TypeSpec.sample` 分别承载：

- `default`：构造器初始化的"未设置"状态，Jackson 安全（`""`/`0`/`null`）。

- `sample`：`sample()` 测试工厂的**非零、SIZE 合规**填充（FixedBitString 铺满 `1`、长度取 SIZE）。

- **模板侧**：`ctor_bitstring` / `ctor_unsigned` = 特殊构造器块；`encode_plain`（无 try/catch） vs `encode_wrapped`（try/catch 兜 MAPPER 异常）；`decode` = 静态 decode 骨架；`sample_factory`（单值）/ `sample_factory_puts`（多字段）/ `sample_factory_choice`（带 `_choice`）三种 sample 工厂。

**type\_map.json 的 exact / prefix** —— `exact` 表映射简单内建类型（bool/u8/i32/String/()…），`prefix` 表按**最长前缀**匹配带约束的容器类型（Empty/VisibleString/OctetString/FixedBitString/Integer…）。把固定映射抽成数据，改映射不必重编译 Rust。

### 2.5 生成产物 - Python（`generator/python/`）

`_native.py`（ctypes）+ `_base.py` + `_types.py`（dataclass，按拓扑排序）+ `__init__.py` + 自动测试 + `pixi.toml`。生成器由单个 `gen_type.rs` 拼接，结构比 Java 侧朴素。

### 2.6 测试

Rust 端（`src/tests/`），`lib.rs` 在 `#[cfg(test)]` 下收纳测试子模块，`build.rs` 先编出 `generated.rs` 即可跑（在 `csasn1/` 下）：

| 命令                                         | 内容                                                                                       |
| ------------------------------------------ | ---------------------------------------------------------------------------------------- |
| `cargo test data_scalars`                  | 各类标量 APER 往返（float、utctime、binarytime、octet/visible/unicode/bit string、boolean、int8..64） |
| `cargo test data_compound`                 | 复合/数组往返 + FFI 适配器流程（Jackson JSON→JER→APER→JER）+ DEFAULT 存活验证                             |
| `cargo test complex_types`                 | 复杂类型往返（尤其 FixedBitString、JER→APER→JER）                                                   |
| `cargo test jer_god_format -- --nocapture` | 打印各类型 JER "上帝格式" 参考输出                                                                    |

> 这些是**纯 Rust 端**断言；Java/Python 侧往返由各 `assets/*` 的 Maven / pixi 独立驱动。

生成的各语言测试

```powershell
just test-java     # assets/java mvn test
just test-python   # assets/python pixi run test
```

### 2.7 与现有项目集成

Java

1. 把生成的 `assets/java/src/main/java/<你的包>/*.java` 复制到项目源码
2. 部署原生动态库（Windows 的 `asn1.dll`、Linux/Mac 的 `libasn1.so`）到 classpath 资源（含 JNA 平台目录）
3. 添加 Jackson + Lombok + JNA 依赖
4. 用 `<前缀>*.encode()` / `.decode()` 完成编解码

Python

```bash
pip install -e assets/python          # 或
cd assets/python && pixi install && pixi run test
```

***

## 3. 技术栈

- [rasn](https://github.com/librasn/rasn) — Rust ASN.1 编解码框架

- [rasn-compiler](https://github.com/librasn/compiler) — ASN.1 → Rust 代码生成器

- [syn](https://github.com/dtolnay/syn) — Rust 代码解析

- [JNA](https://github.com/java-native-access/jna) — Java 原生调用

- [Jackson](https://github.com/FasterXML/jackson) — Java JSON 序列化

- [Lombok](https://projectlombok.org/) — Java 样板代码消除

- [pixi](https://pixi.sh/) — Python 包管理

- [ctypes](https://docs.python.org/3/library/ctypes.html) — Python 原生调用
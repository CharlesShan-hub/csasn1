# 纯 `_v` 统一数据模型 — 完整迁移方案

## 核心理念

所有数据全部存在 InnerBase 的 `_v` (LinkedHashMap<String, Object>) 中。
**`_v` 里只有 Map / List / 基本类型（String、Integer 等），没有任何 Inner\* Java 对象。**

Inner\* 类只有构造器、encode/decode 和一些工具方法，没有 Java 数据字段。
序列化/反序列化全走 `_v`，`@JsonValue toJsonValue()` 统一出口。

## 数据模型

| 类型 | `_v` 内容 | 序列化结果 |
|------|----------|-----------|
| **Scalar** (InnerInt32) | `{"_": 42}` | `42` (数字) |
| **Scalar** (InnerVisibleString) | `{"_": "hello"}` | `"hello"` (字符串) |
| **CHOICE** (InnerData) | `{"_choice": "error", "error": 0}` | `{"error": 0}` |
| **SEQUENCE** (InnerXxx) | `{"field1": {map...}, "field2": 42}` | `{"field1": {...}, "field2": 42}` |

`@JsonValue toJsonValue()` 统一处理：
- 如果 `_v` 有 `_choice` → 返回 `{"variant": value}`
- 如果 `_v` 只有一个 `_` → 返回展开的值（适配标量）
- 否则 → 直接返回 `_v`

## 核心原则：`_v` 里只有 Map，没有 Inner\* 对象

### 不这样做（存 Java 对象）
```java
// 构造器中
_v.put("child", new InnerChild());  // child 是 InnerChild Java 对象
// 取用
((InnerChild) obj._v.get("child"))._v.put("field", 42);
// 序列化时 Jackson 调 InnerChild 的 @JsonValue → 返回其 _v
```

### 这样做（纯 Map）
```java
// 构造器中
_v.put("child", new InnerChild()._v);  // child 是 InnerChild 的 _v map!
// 取用
((Map<String, Object>) obj._v.get("child")).put("field", 42);
// 序列化时就是 Map 序列化，零歧义
```

### 为什么纯 Map 更好

1. **序列化没有歧义** — 纯 Map 序列化就是 Jackson 的标准 Map 序列化，不需要 `@JsonValue` 递归
2. **反序列化没有歧义** — JSON 直接转成 Map 套 Map，不需要 `@JsonAnySetter` 做类型转换
3. **equals 不需要定制** — 两个 `_v` Map 的比较就是数据内容的比较
4. **赋值更简洁** — 没有 cast 到 Inner\* 类型的步骤

## 例子

### SEQUENCE A 包含 SEQUENCE B

```java
// InnerB.java — 纯 _v
public class InnerB extends InnerBase {
    public InnerB() {
        _v.put("x", 0);
        _v.put("y", "hello");
    }
    // encode/decode...
}

// InnerA.java — 包含 InnerB
public class InnerA extends InnerBase {
    public InnerA() {
        _v.put("name", "test");
        _v.put("child", new InnerB()._v);  // ← 只存 Map！
    }
    // encode/decode...
}
```

### 赋值操作

```java
// 建根对象
InnerA a = new InnerA();

// 直接设标量字段
a._v.put("name", "newName");

// 取子 Map 设子字段
((Map<String, Object>) a._v.get("child")).put("x", 42);

// 多级嵌套
((Map<String, Object>) ((Map<String, Object>) a._v.get("child"))
    .get("grandchild")).put("z", true);

// 改 CHOICE variant
Map<String, Object> data = (Map<String, Object>) a._v.get("data");
data.put("_choice", "int32");
data.put("int32", 100);
```

### 用 @JsonAnySetter 反序列化

```java
// SEQUENCE 的反序列化 — Jackson 逐字段调 setField
@JsonAnySetter
public void setField(String key, Object value) {
    // value 可能是 Map（对应子 SEQUENCE/CHOICE）或基本类型
    // 直接存 _v，不做类型转换
    _v.put(key, value);
    _set.add(key);
}
```

### 序列化结果

```java
InnerA a = new InnerA();
a._v.put("name", "foo");
a._v.put("child", new InnerB()._v);
((Map<String, Object>) a._v.get("child")).put("x", 99);

String json = MAPPER.writeValueAsString(a._v);
// → {"name":"foo","child":{"x":99,"y":"hello"}}
```

## 各生成器改动

### 1. native_gen.rs (InnerBase) — 已改完

- `@JsonValue` 已加回 `toJsonValue()`
- 不再需要 `getValue()`/`setValue()` 之外的修改

### 2. gen_struct.rs (SEQUENCE) — 完全重写

**之前：**
```java
public class InnerXxx extends InnerBase {
    public InnerType1 field1 = new InnerType1();
    public int field2 = 0;
    public byte[] encode() { MAPPER.writeValueAsString(this); }
    public static InnerXxx decode(byte[] d) { MAPPER.readValue(..., InnerXxx.class); }
}
```

**之后：**
```java
@JsonInclude(Include.NON_NULL)
public class InnerXxx extends InnerBase {
    public InnerXxx() {
        _v.put("field1", MAPPER.convertValue(new InnerType1()._v, Map.class));
        _v.put("field2", 0);
    }

    @JsonAnySetter
    public void setField(String key, Object value) {
        _v.put(key, value);
        _set.add(key);
    }

    @JsonIgnore public transient java.util.Set<String> _set = new java.util.HashSet<>();

    public byte[] encode() {
        java.util.Map<String, Object> out = new java.util.LinkedHashMap<>(_v);
        if (!_set.isEmpty()) out.keySet().retainAll(_set);
        return InnerNative.encode("Xxx", DEFAULT_ENCODING, MAPPER.writeValueAsString(out));
    }

    public static InnerXxx decode(byte[] d) {
        return MAPPER.readValue(InnerNative.decode(...), InnerXxx.class);
    }
}
```

关键变化：
- **无任何数据字段**
- 构造器填 `_v`：标量直接存值，子 SEQUENCE/CHOICE 存 `new InnerType()._v`
- **`@JsonAnySetter`** — 存值不进 Inner\* 对象，因为 JSON 反序列化后已经是 Map
- **encode 用 `MAPPER.writeValueAsString(_v)`**
- **`_set`** 追踪 OPTIONAL 字段

### 3. gen_choice.rs (CHOICE) — 重写

**之前：**
```java
public class InnerData extends InnerBase {
    @JsonIgnore public String _choice;
    @JsonIgnore public InnerServiceError error;
    @JsonIgnore public List<InnerData> array;
    public InnerData() {
        this._choice = "error"; this.error = new InnerServiceError();
        this.array = new ArrayList<>(); ...
    }
    @JsonAnyGetter public Map<...> serializeChoice() { ... }
    @JsonAnySetter public void deserializeChoice(key, value) { ... }
    public byte[] encode() { MAPPER.writeValueAsString(this); }
}
```

**之后：**
```java
@JsonInclude(Include.NON_NULL)
public class InnerData extends InnerBase {
    public InnerData() {
        _v.put("_choice", "error");
        _v.put("error", 0);  // ServiceError → scalar, 直接存 0
        _v.put("array", new java.util.ArrayList<>());
        _v.put("structure", new java.util.ArrayList<>());
        _v.put("bit_string", new byte[0]);
        // 其他 variant 不放（null），NON_NULL 排除
    }

    @JsonSetter("error") public void setError(int v) {
        _v.put("_choice", "error"); _v.put("error", v);
    }
    @JsonSetter("array") public void setArray(java.util.List<Map<String, Object>> v) {
        _v.put("_choice", "array"); _v.put("array", v);
    }
    // ... 每个 variant

    public byte[] encode() {
        return InnerNative.encode("Data", DEFAULT_ENCODING, MAPPER.writeValueAsString(_v));
    }
    public static InnerData decode(byte[] d) {
        return MAPPER.readValue(InnerNative.decode(...), InnerData.class);
    }
}
```

关键变化：
- **无 variant 字段**
- variant 值是纯数据（scalar 直接存值，sub-SEQUENCE 存 Map）
- `@JsonSetter` 设 `_choice` + variant 值
- encode 用 `MAPPER.writeValueAsString(_v)`

### 4. gen_newtype.rs (Scalars) — 不需要改

已使用 `_v` 模型。`_v = {"_": value}`。

### 5. test_struct.rs — 适配

测试代码从 `obj.fieldName = value` 改为 `obj._v.put("fieldName", value)`。
对于子 SEQUENCE/CHOICE：`obj._v.put("childField", new InnerChild()._v)`。

### 6. test_newtype.rs — 不需要改

## 影响范围

| 文件 | 改动量 |
|------|--------|
| native_gen.rs | ✅ 已改完 |
| gen_struct.rs | 🔄 完全重写 |
| gen_choice.rs | 🔄 大量重写 |
| gen_newtype.rs | ❌ 不需改 |
| test_struct.rs | 🔄 适配 |
| test_newtype.rs | ❌ 不需改 |
| mod.rs (DefaultInner) | ❌ 不需改 |

## 风险

1. **`_v` 的 Type Safety**: `_v` 是 `Map<String, Object>`，取值需要 cast。Cms\* 包装类应封装 cast。
2. **OPTIONAL 字段**: `_set` + encode 过滤需正确。
3. **DefaultInner\***: 它们在 mod.rs 生成，有自己的 `value` 字段。作为 SEQUENCE 字段时，被 `_v` 指向一个 DefaultInner\* 对象。序列化时 DefaultInner\* 的 `@JsonValue` 会展开。注意 DefaultInner\* 对象不是 Map，是 Java 对象。 → **建议：DefaultInner\* 可以保持原样，存到 `_v` 里的是 DefaultInner\* 对象，不是 Map。** 因为 DefaultInner\* 有 `@JsonValue`，Jackson 序列化时自动展开为 String/byte[]。

# `_v` 数据结构示例

> 所有 Inner\* 类没有 Java 数据字段。数据全在 `_v` (LinkedHashMap) 里。
> `_v` 里只有 Map / List / 基本类型 (String, Integer, Boolean, byte[])，没有 Inner\* 对象。

---

## 1. 标量 — Scalar

### InnerInt32 (INTEGER)

```json
// _v 内容
{"_": 42}

// 序列化结果
42
```

### InnerVisibleString (VisibleString SIZE(0..129))

```json
// _v 内容
{"_": "hello"}

// 序列化结果
"hello"
```

### InnerBoolean (INTEGER 0..1)

```json
// _v 内容
{"_": 1}

// 序列化结果
1
```

### InnerBitString (BIT STRING SIZE(7))

```json
// _v 内容
{"_": 0}

// 序列化结果（JSON 中是 hex 字符串）
"00"
```

### InnerOctetString (OCTET STRING SIZE(8))

```json
// _v 内容
{"_": [0, 0, 0, 0, 0, 0, 0, 0]}     // byte[8]

// 序列化结果（hex 字符串）
"0000000000000000"
```

---

## 2. 枚举 — ENUMERATED

### InnerServiceError (INTEGER 0..12)

```json
// _v 内容
{"_": 0}

// 序列化结果
0
```

枚举在 ASN.1 中是 INTEGER，所以 `_v` 结构和标量一样。

---

## 3. 选择体 — CHOICE

### InnerData (CHOICE，24 个 variant)

选 `error` variant：

```json
// _v 内容
{
  "_choice": "error",
  "error": 0
}

// 序列化结果
{"error": 0}
```

选 `Boolean` variant：

```json
// _v 内容
{
  "_choice": "Boolean",
  "Boolean": 1
}

// 序列化结果
{"Boolean": 1}
```

选 `array` variant（内嵌 SEQUENCE OF）：

```json
// _v 内容
{
  "_choice": "array",
  "array": [
    {"_choice": "int32", "int32": 10},
    {"_choice": "int32", "int32": 20}
  ]
}

// 序列化结果
{"array": [{"int32": 10}, {"int32": 20}]}
```

规则：
- `"_choice"` 记录当前选的是哪个 variant
- variant 值直接作为对应的 key 存入
- 其他 variant key 不在 `_v` 里（`@JsonInclude(NON_NULL)` 排除）

---

## 4. 序列 — SEQUENCE

### 无 OPTIONAL 字段

```asn.1
InnerSubReference ::= VisibleString (SIZE(0..129))
```

```json
// _v 内容
{"_": "xxxxxxxxx..."}
```

### 有 OPTIONAL 字段

```asn.1
InnerLogEntry ::= SEQUENCE {
    time     UtcTime,
    entry-data SEQUENCE OF LogEntryEntryData,
    reason   ReasonCode  OPTIONAL
}
```

设了 `time` + `entry-data`，没设 `reason`（OPTIONAL）：

```json
// _v 内容
{
  "time": {"_": "2026-07-30T12:00:00Z"},
  "entry-data": [{"_": [{子数据...}]}],
  "reason": {"_": 3},               // OPTIONAL 字段，构造器有默认值

  "_optional": []                    // 没有 OPTIONAL 字段被显式设过
}

// 序列化结果（encode 严格模式：_optional 外的 OPTIONAL 字段不编码）
{"time": "...", "entry-data": [...]}
// reason 不在 _optional 里 → 排除
```

设了 all，且显式设了 `reason`：

```json
// _v 内容
{
  "time": {"_": "2026-07-30T12:00:00Z"},
  "entry-data": [{"_": [{子数据...}]}],
  "reason": {"_": 3},

  "_optional": ["reason"]            // 只有 reason 被显式设过
}

// 序列化结果 — time/entry-data 是必选的始终编，reason 在 _optional 里也编
{"time": "...", "entry-data": [...], "reason": 3}
```

规则：
- **非 OPTIONAL 字段**：构造器就放进 `_v`，encode 时始终编码
- **OPTIONAL 字段**：构造器放默认值进 `_v`，但 encode 时**只编码在 `"_optional"` set 里的**
- 用户显式设了 OPTIONAL 字段 → `"_optional"` 加入该字段名 → encode 时包含
- `"_optional"` 只包含 OPTIONAL 字段名，不含必选字段

---

## 5. 序列的序列 — SEQUENCE OF

### InnerLogEntryEntryData (SEQUENCE OF)

```asn.1
LogEntry ::= SEQUENCE {
    time       UtcTime,
    entry-data SEQUENCE OF SEQUENCE {
        reference SubReference,
        value     Data
    }
}
```

```json
// _v 内容
{
  "time": {"_": "2026-07-30T12:00:00Z"},
  "entry-data": [
    {
      "reference": {"_": "C_B5041X/S1"},
      "value": {"_choice": "int32", "int32": 100}
    },
    {
      "reference": {"_": "C_B5041X/S2"},
      "value": {"_choice": "Boolean", "Boolean": 1}
    }
  ],

  "_optional": []     // 没有 OPTIONAL 字段，_optional 只含 OPTIONAL 字段名
}

// 序列化结果
{
  "time": "2026-07-30T12:00:00Z",
  "entry-data": [
    {
      "reference": "C_B5041X/S1",
      "value": {"int32": 100}
    },
    {
      "reference": "C_B5041X/S2",
      "value": {"Boolean": 1}
    }
  ]
}
```

---

## 6. 完整案例：赋值操作

```java
// 构造
InnerLogEntry entry = new InnerLogEntry();

// 设 time（标量）
((Map<String, Object>) entry._v.get("time")).put("_", "2026-07-30T12:00:00Z");

// 建 entry-data 列表
List<Map<String, Object>> entries = new ArrayList<>();

// 建第一个 entry
Map<String, Object> e1 = new LinkedHashMap<>();
e1.put("reference", new InnerSubReference()._v);
((Map<String, Object>) e1.get("reference")).put("_", "C_B5041X/S1");
e1.put("value", new InnerData()._v);
((Map<String, Object>) e1.get("value")).put("_choice", "int32");
((Map<String, Object>) e1.get("value")).put("int32", 100);
entries.add(e1);

entry._v.put("entry-data", entries);

// 标记 OPTIONAL 字段已设
Set<String> opt = (Set<String>) entry._v.get("_optional");
opt.add("time");
opt.add("entry-data");

// encode
byte[] data = entry.encode();
```

---

## 元字段命名规则

| key | 用途 |
|-----|------|
| `"_"` | 标量值 |
| `"_choice"` | CHOICE 当前选中的 variant 名 |
| `"_optional"` | 被显式设置的 OPTIONAL 字段名集合 |
| 其他 | ASN.1 字段名，直接作为 key |

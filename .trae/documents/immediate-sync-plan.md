# 即时同步方案

## 核心改动

**`CmsScalar.innerSet()` 加一行 `syncToInnerValue()`：**

```java
// 改前（延迟同步）
protected void innerSet(Object v) {
    innerCache.put("value", v);
}

// 改后（即时同步）
protected void innerSet(Object v) {
    innerCache.put("value", v);
    syncToInnerValue();  // ← 立即推到 inner
}
```

## 效果

```
改前：
  fileSize.value(1024L)
    → innerCache["value"] = 1024
    → inner.value 还是 0                       ← 不等
  encode() → syncToInner() → inner.value = 1024

改后：
  fileSize.value(1024L)
    → innerCache["value"] = 1024
    → inner.value = 1024                       ← 一致
  encode() → syncToInner() 什么都不用干
```

## 连锁清理

| 位置 | 当前 | 改后 | 理由 |
|---|---|---|---|
| `CmsScalar.syncToInner()` | `syncToInnerValue(); super.syncToInner()` | `super.syncToInner()` 即可 | innerCache → inner 不用再推了 |
| `CmsSequence.syncToInner()` wrapper 循环 | 每个子 wrapper 调 `syncToInner()` | 可保留，但 scalar wrapper 的 syncToInner 变成空操作 | 安全，不影响 |
| `CmsSequence.injectFields` 对 CmsScalar 的 syncFromInner | 绑定 inner 后调 `wrapper.syncFromInner()` | 保留不变 | decode 后需要拉 inner → cache |
| `CmsUtcTime.syncToInner()` | 打包子字段到字节数组 | 不变 | 本来就不依赖标量的延迟同步 |
| `CmsSequence.bindWrapper()` | 调 `wrapper.syncToInner()` | 可保留 } wrapper 已经即时同步了，没事 | |
| `encode()` 整体 | `syncToInner() → inner.encode()` | `syncToInner() → inner.encode()` 可保留 | syncToInner 退化为空操作 |

## 验证

改完后跑全量测试：
```powershell
$env:JAVA_HOME="D:\envs\.jdks\ms-21.0.10"; $env:PATH="$env:JAVA_HOME\bin;$env:PATH"; mvn test -pl jcms-core -am
```
预期全绿。

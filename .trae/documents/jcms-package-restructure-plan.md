# jcms-core 包结构彻底重组计划

## 核心理念

```
data/ = 零件（可复用的 ASN.1 类型组件）
pdu/  = 协议消息（可独立编码/解码的 PDU）
```

## 最终结构

```
com.ysh.jcms
├── data/                      ← 所有非 PDU 的零件类型
│   ├── core/                  ← 类型系统的抽象基类 + 注解
│   │   ├── CmsType.java
│   │   ├── CmsScalar.java
│   │   ├── CmsSequence.java
│   │   ├── CmsChoice.java
│   │   ├── CmsBits.java
│   │   ├── CmsEnum.java
│   │   └── CmsField.java
│   │
│   ├── scalar/                ← 标量值类型（extends CmsScalar）
│   │   ├── CmsBoolean.java
│   │   ├── CmsInt8.java / CmsInt8U.java / CmsInt16.java / CmsInt16U.java
│   │   ├── CmsInt24U.java / CmsInt32.java / CmsInt32U.java
│   │   ├── CmsInt64.java / CmsInt64U.java
│   │   ├── CmsFloat32.java / CmsFloat64.java
│   │   ├── CmsOctetString.java          ← 来自 data/string/
│   │   ├── CmsString.java               ← 来自 data/string/
│   │   ├── CmsFC.java                   ← 来自 data/fc/
│   │   ├── CmsSubReference.java         ← 来自 data/common/
│   │   ├── CmsObjectReference.java      ← 来自 data/common/
│   │   ├── CmsObjectName.java           ← 来自 data/common/
│   │   ├── CmsEntryId.java              ← 来自 data/common/
│   │   ├── CmsAssociationId.java        ← 来自 svc/other/
│   │   └── CmsReqId.java                ← 来自 svc/other/
│   │
│   ├── enumerate/              ← 枚举类型（extends CmsEnum，包括所有 ServiceError 子类）
│   │   ├── CmsServiceError.java         ← 来自 data/common/
│   │   ├── CmsDbpos.java                ← 来自 data/common/
│   │   ├── CmsTcmd.java                 ← 来自 data/common/
│   │   ├── CmsSmpMod.java               ← 来自 data/block/
│   │   ├── CmsOrCat.java                ← 来自 data/control/
│   │   ├── CmsAddCause.java             ← 来自 data/control/
│   │   ├── CmsAbortReason.java          ← 来自 svc/connection/
│   │   ├── CmsAssociateError.java       ← 来自 svc/connection/
│   │   ├── CmsReleaseError.java         ← 来自 svc/connection/
│   │   ├── CmsObjectClass.java          ← 来自 svc/directory/
│   │   ├── CmsAcsiClass.java            ← 来自 svc/directory/
│   │   ├── CmsGetServerDirectoryError.java          ← 来自 svc/directory/
│   │   ├── CmsGetLogicalDeviceDirectoryError.java   ← 来自 svc/directory/
│   │   ├── CmsGetLogicalNodeDirectoryError.java     ← 来自 svc/directory/
│   │   ├── CmsGetAllDataValuesError.java            ← 来自 svc/directory/
│   │   ├── CmsGetAllDataDefinitionError.java        ← 来自 svc/directory/
│   │   └── CmsGetAllCbValuesError.java              ← 来自 svc/directory/
│   │
│   ├── bitarray/               ← 位数组类型（extends CmsBits）
│   │   ├── CmsQuality.java              ← 来自 data/common/
│   │   ├── CmsTimeQuality.java          ← 来自 data/time/
│   │   ├── CmsCheck.java                ← 来自 data/control/
│   │   ├── CmsTriggerConditions.java    ← 来自 data/block/
│   │   ├── CmsReasonCode.java           ← 来自 data/block/
│   │   ├── CmsMsvcbOptFlds.java         ← 来自 data/block/
│   │   ├── CmsLcbOptFlds.java           ← 来自 data/block/
│   │   └── CmsRcbOptFlds.java           ← 来自 data/block/
│   │
│   ├── choice/                 ← 联合类型（extends CmsChoice）
│   │   ├── CmsData.java                 ← 来自 data/choice/
│   │   ├── CmsDataDefinition.java       ← 来自 data/choice/
│   │   ├── CmsCbValueChoice.java        ← 来自 svc/directory/
│   │   └── CmsReferenceChoice.java      ← 来自 svc/other/
│   │
│   ├── sequence/               ← 结构类型（extends CmsSequence，非 PDU）
│   │   ├── block/              ← 控制块
│   │   │   ├── CmsBrcb.java
│   │   │   ├── CmsUrcb.java
│   │   │   ├── CmsGoCb.java
│   │   │   ├── CmsLcb.java
│   │   │   ├── CmsMsvcb.java
│   │   │   └── CmsSgcb.java
│   │   │
│   │   ├── entry/              ← 目录条目
│   │   │   ├── CmsDataValueEntry.java        ← 来自 svc/directory/
│   │   │   ├── CmsDataDefinitionEntry.java   ← 来自 svc/directory/
│   │   │   └── CmsCbValueEntry.java          ← 来自 svc/directory/
│   │   │
│   │   └── common/             ← 通用共享结构
│   │       ├── CmsFileEntry.java             ← 来自 data/common/
│   │       ├── CmsPhyComAddr.java            ← 来自 data/common/
│   │       ├── CmsOriginator.java            ← 来自 data/control/
│   │       ├── CmsDataDefinitionArray.java   ← 来自 data/choice/
│   │       ├── CmsDataDefinitionStructElem.java ← 来自 data/choice/
│   │       └── CmsAuthenticationParameter.java ← 来自 svc/connection/
│   │
│   └── time/                   ← 特殊时间类型（直接 extends CmsType）
│       ├── CmsUtcTime.java                  ← 来自 data/time/
│       ├── CmsBinaryTime.java               ← 来自 data/time/
│       ├── CmsTimeStamp.java                ← 来自 data/common/
│       └── CmsEntryTime.java                ← 来自 data/common/
│
├── pdu/                       ← 协议数据单元（每个类对应一个独立的 Inner*PDU）
│   ├── connection/             ← 连接管理服务 (8.2.x)
│   │   ├── CmsAssociateRequest.java
│   │   ├── CmsAssociateResponse.java
│   │   ├── CmsReleaseRequest.java
│   │   ├── CmsReleaseResponse.java
│   │   └── CmsAbort.java
│   │
│   └── directory/              ← 目录服务 (8.4.x)
│       ├── CmsGetServerDirectoryRequest.java
│       ├── CmsGetServerDirectoryResponse.java
│       ├── CmsGetLogicalDeviceDirectoryRequest.java
│       ├── CmsGetLogicalDeviceDirectoryResponse.java
│       ├── CmsGetLogicalNodeDirectoryRequest.java
│       ├── CmsGetLogicalNodeDirectoryResponse.java
│       ├── CmsGetAllDataValuesRequest.java
│       ├── CmsGetAllDataValuesResponse.java
│       ├── CmsGetAllDataDefinitionRequest.java
│       ├── CmsGetAllDataDefinitionResponse.java
│       ├── CmsGetAllCbValuesRequest.java
│       └── CmsGetAllCbValuesResponse.java
│
├── info/                      不变（元信息枚举）
│   ├── CmsCdcInfo.java
│   ├── CmsDataTypeInfo.java
│   ├── CmsLnInfo.java
│   ├── CmsServiceInfo.java
│   └── FunctionalConstraint.java
│
└── util/                      不变（工具类）
    ├── CmsBytesUtil.java
    ├── CmsEqualUtil.java
    └── CmsFormatUtil.java
```

## 改动统计

| 目标包 | 文件数 | 来源 |
|--------|--------|------|
| `data/core/` | 7 | 不变 |
| `data/scalar/` | 19 | data/scalar(12) + data/string(2) + data/fc(1) + data/common(4) + svc/other(2) |
| `data/enumerate/` | 17 | data/common(3) + data/block(1) + data/control(2) + svc/connection(3) + svc/directory(8) |
| `data/bitarray/` | 8 | data/time(1) + data/common(1) + data/control(1) + data/block(5) |
| `data/choice/` | 4 | data/choice(2) + svc/directory(1) + svc/other(1) |
| `data/sequence/block/` | 6 | data/block 中的 CmsSequence 子类 |
| `data/sequence/entry/` | 3 | svc/directory 中的条目组件 |
| `data/sequence/common/` | 6 | data/common(2) + data/control(1) + data/choice(2) + svc/connection(1) |
| `data/time/` | 4 | data/time(2) + data/common(2) |
| `pdu/connection/` | 5 | svc/connection 中的 PDU 类型（减 AuthenticationParameter） |
| `pdu/directory/` | 12 | svc/directory 中的 PDU 类型（减 error/entry/choice 组件） |
| `info/` | 5 | 不变 |
| `util/` | 3 | 不变 |
| **合计** | **99** | |

**注意:** CmsOriginator, CmsCheck, CmsAddCause, CmsOrCat 等原来在 `data/control/` 的类型，分别归入了 `data/struct/common/`, `data/bitarray/`, `data/enumerate/` — 按继承链分类，不是按语义。

## 实施方式

由于你会在 IDEA 中用拖拽完成（IDEA 自动更新 package 声明和 import），只需要按上述结构把文件拖到对应目录即可。

验证：拖完后编译 + `mvn test -pl jcms-core -am` 全绿

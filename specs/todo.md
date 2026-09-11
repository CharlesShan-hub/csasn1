# 功能订正与语法整理

> 订正对象：`cms/csasn1/specs/dlt2811.asn`，对照 DL/T 2811 原文（含 ASN.1 附录）逐章核对。
>
> 背景备注：
>
> - GB/T 版本是 DL/T 2811 的重写，未收录 ASN.1，因此 DL/T 2811 的 ASN.1 附录是唯一规范文本。
> - 已核实（原文 7.1.1）："布尔型只有两个值：TRUE、FALSE。使用 GB/T 16263.2（即 ASN.1 PER，对应 ISO/IEC 8825-2）编码规则时，布尔型映射到 BOOLEAN 类型。"即标准全篇使用内置 `BOOLEAN`（PDU 中为 `BOOLEAN DEFAULT TRUE` / `DEFAULT FALSE`），**从未定义** `Boolean ::= INTEGER (0..1)`；本文件中的该定义及其"7.1.1"出处标注系此前生成时虚构，需删除。
> - 已确认互通性无影响：PER 下 `BOOLEAN` 与 `INTEGER (0..1)` 均为 1 bit，`DEFAULT TRUE` ≡ `DEFAULT 1`，SEQUENCE / SEQUENCE OF / CHOICE 内嵌套均等价，故历史实现无需返工。

## asn文件

### 0. 全局决策（影响所有 BOOLEAN 相关条目）
- [x] 方向已明确：原文 7.1.1 白纸黑字"布尔型映射到 BOOLEAN 类型" → 全文恢复内置 `BOOLEAN` / `DEFAULT TRUE` / `DEFAULT FALSE`（asn 已改完）
- [x] csasn1 生成管线适配：rasn 把内置 BOOLEAN 生成为 Rust `bool` → jcms-data 重新生成后 BOOLEAN 字段直接用裸 `Boolean` 对象（无 Inner/DefaultInner 包装类）；jcms-core 的 CmsBoolean 内核改用 InnerEmpty 占位 + `_v["_"]=true/false`，门面 API 不变，61 个使用类零改动；CmsBooleanTest 改为 CmsData Boolean 变体 roundtrip + 值语义；jcms-core 全量测试（100 类）通过
- [ ] 核对原文 7.7 Data CHOICE 中 `boolean [3]` 的写法并对齐（预期为内置 BOOLEAN）
- [ ] 文件头来源注释（如 "GB/T 45906.3" 等声明）一律当作未核实声明，随各章核对逐条验证

### 7.1 基本类型
- [x] 7.1.1 已核实：原文只说"布尔型…映射到 BOOLEAN 类型"，**没有**定义 `Boolean ::= INTEGER (0..1)` —— 本文件该行系虚构，确认删除
- [x] 删除 `Boolean ::= INTEGER (0..1)` 定义（改为注释：使用内置 BOOLEAN，取值 TRUE / FALSE）；全文 69 处类型引用 → `BOOLEAN`，26 处 `DEFAULT 1` → `DEFAULT TRUE`，2 处 `DEFAULT 0` → `DEFAULT FALSE`；Data CHOICE 变体名 `Boolean`（行 382/420 标识符）保留不动
- [ ] 7.1.2 核对 Int8 / Int8U / Int16 / Int16U / Int24U / Int32 / Int32U / Int64 / Int64U 的取值范围
- [ ] 7.1.4 核对 Float32 / Float64 的 SIZE(4) / SIZE(8)
- [ ] 核对章节编号：本文件缺 7.1.3，确认原文编号并补齐或修正注释

### 7.2 时间类型

- [ ] 7.2.1 UtcTime（OCTET STRING (8)）、TimeQuality 位定义与 SIZE(8) 与原文核对
- [ ] 7.2.2 BinaryTime：确认原文是 OCTET STRING (6) 还是 SEQUENCE（msOfDay/daysSince1984），处理被注释的定义

### 7.3 派生类型

- [ ] 7.3.1-7.3.8 ObjectName(64) / ObjectReference(129) / SubReference / TimeStamp / Dbpos(2) / Quality(13) / Tcmd(2) / EntryID(8) 与原文核对
- [ ] 7.3.10 FileEntry 字段与 tag 核对
- [ ] 7.3.11 ServiceError 0..12 枚举值逐项核对
- [ ] 7.3.12 PhyComAddr 核对
- [ ] 清理 7.3.9 被注释的重复 EntryTime（7.2.2 已有）

### 7.4 FC

- [ ] FunctionalConstraint VisibleString (SIZE(2)) 核对

### 7.5 控制块公共属性

- [ ] 7.5.2 Originator orCat 0..8 枚举核对
- [ ] 7.5.3 Check 位定义核对（原文是否带 named bits）
- [ ] 7.5.4 AddCause 0..27 枚举逐项核对

### 7.6 控制块属性位

- [ ] 7.6.2-7.6.6 TriggerConditions(6) / ReasonCode(7) / RcbOptFlds(10) / LcbOptFlds(1) / MsvcbOptFlds(5) 的 named bits 与 SIZE 核对
- [ ] 7.6.7 SmpMod 0..2 核对

### 7.7 Data CHOICE

- [ ] 24 个备选的 tag 与类型逐项核对
- [ ] 重点核对 bit-string \[14] 定义为 INTEGER 是否与原文一致

### 7.8 DataDefinition

- [ ] bit-string / octet-string / visible-string / unicode-string 备选用 INTEGER 作长度定义的写法与原文核对
- [ ] array / structure 的嵌套结构与 tag 核对

### 协议层 APCH / APDU

- [x] ControlCode 的 next / resp / err 三个 Boolean 类型引用已改内置 `BOOLEAN`
- [ ] next / resp / err 与原文 APCH 4 字节位定义核对
- [ ] Apdu.asdu SIZE(0..65531) 上限推导核对

### 8.2 关联服务

- [ ] 8.2.1 Associate 认证参数结构与 OPTIONAL 核对
- [ ] 8.2.2 Release / 8.2.3 Abort reason 枚举核对

### 8.3 目录服务 (0x51-0x56)

- [x] 6 处 `moreFollows Boolean DEFAULT 1` 按全局决策订正（已改 `BOOLEAN DEFAULT TRUE`）
- [ ] GetAllCBValues 的 CHOICE（brcb/urcb/lcb/sgcb/gocb/msvcb）核对

### 8.4 数据服务 (0x11-0x14)

- [x] 3 处 moreFollows 按全局决策订正
- [ ] SetDataValues-ErrorPDU 结构核对

### 8.5 数据集服务 (0x41-0x45)

- [x] 2 处 moreFollows 按全局决策订正
- [ ] CreateDataSet memberData 的 fc 必选性核对

### 8.6 定值组服务 (0x61-0x66)

- [ ] 8.6.3 SetEditSGValue：value \[2] 缺 \[1]，核对原文是否有 fc \[1]
- [x] 8.6.5 / 8.6.6 共 2 处 moreFollows 按全局决策订正
- [ ] SGCB 字段与 resvTms OPTIONAL 核对

### 8.7 报告与 RCB (0x31-0x35)

- [x] ReportPDU：moreSegmentsFollow / bufOvfl 按全局决策订正
- [ ] ReportPDU entry 结构核对
- [x] BRCB：rptEna / gi / purgeBuf 按全局决策订正
- [x] URCB：rptEna / gi / resv 按全局决策订正
- [ ] 核对 resv 的 tag（13 还是 14）
- [ ] SetBRCBValues / SetURCBValues 及各自 ErrorPDU 逐字段核对（rptID \[1] 实为 ServiceError 的命名是否原文如此；tag 跳号核对）
- [ ] 删除 992-993 行重复的 8.7.2 注释头

### 8.8 日志服务 (0x67-0x6B)

- [x] LCB logEna 按全局决策订正
- [ ] optFlds / bufTm OPTIONAL 核对
- [x] 8.8.4 / 8.8.5 / 8.8.6 共 3 处 moreFollows 按全局决策订正（连同 GetLCBValues 共 4 处）

### 8.9 GOOSE (0x81-0x85)

- [x] SendGOOSEMessage：simulation / ndsCom 按全局决策订正
- [x] GoCB：goEna / ndsCom 按全局决策订正（原文截图已确认为 BOOLEAN）
- [x] SetGoCBValues goEna、GetGoCbValues moreFollows 按全局决策订正
- [ ] GSEMngt 包装 PDU（0x88-B9 载体）与原文核对

### 8.10 采样值 (0x86-0x88)

- [x] SendMSVMessage simulation 按全局决策订正
- [x] MSVCB svEna 按全局决策订正
- [ ] smpMod / smpRate / optFlds / dstAddress OPTIONAL 核对
- [x] SetMSVCBValues svEna、GetMSVCBValues moreFollows 按全局决策订正

### 8.11 控制服务 (0x21-0x27)

- [x] SelectWithValue / Operate / Cancel / CommandTermination / TimeActivatedOperate / TimeActivatedOperateTermination 全部 test 字段按全局决策订正
- [ ] 各 PDU 的 origin / ctlNum / t / check 的 tag 核对（注意 Operate 无 operTm）
- [ ] addCause 仅在 Error / Termination PDU 出现，OPTIONAL 与否核对

### 8.12 文件服务 (0x71-0x75)

- [x] 8.12.1 / 8.12.2 共 2 处 `endOfFile Boolean DEFAULT 0` 按全局决策订正（已改 `BOOLEAN DEFAULT FALSE`）
- [x] 8.12.5 moreFollows 按全局决策订正

### 8.13 RPC (0x91-0x95)

- [x] 4 处 moreFollows 按全局决策订正
- [ ] GetRpcInterfaceDefinition / GetRpcMethodDefinition 结构核对

### 8.14 / 8.15

- [ ] Test (0xA1) FL=0 无字段确认
- [ ] AssociateNegotiate (0x04) 字段与 modelVersion 核对

### 全文语法整理

- [ ] 修复乱码：文件头及各处 "-- ? --"、"? APCH" 等（原为破折号）
- [ ] 统一章节注释行格式（部分标题行缺尾部 "--"）
- [ ] 清理整段被注释的死代码（7.1.5 / 7.1.6 / 7.1.7 / 7.1.8 / 7.2.2 SEQUENCE / 7.3.9 等），决定保留（作对照）或删除
- [ ] 收尾：BOOLEAN 路线已定，asn 已改完，jcms-data 已重新生成，jcms-core 已适配（CmsBoolean 内核 → InnerEmpty，100 测试类全绿）；剩 jcms-app 联调验证


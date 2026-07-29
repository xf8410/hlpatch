# APK 676 离线解包索引

输入为同批次 base APK 与 `config.arm64_v8a` APK。原始 APK、解包文件和 SO **未提交仓库**。

## 明确结果

- 完整解包文件：4498 个，共 658531869 字节。
- DEX 类：19479 条。
- Native 库：22 个；导出符号去重后 39853 条。
- 两个包中未发现明文 `global-metadata.dat`，全文件标准 metadata 魔数命中数：0。
- 现有运行时方法主表原始记录 27695 条，按 `namespace|class` 合并为 16553 个名称、25877 个语义版本。逐字母重复记录中已有语义 14598 条已跳过；仅保留 9834 条同名差异版本。比较不使用易变的运行时地址。
- 合并先前 metadata 类名报告后索引共 17204 条；来源逐条标注。

## 文件

- `file_manifest.tsv`：所有解包文件路径、大小、SHA-256。
- `dex_classes.tsv`：DEX class_defs 精确类名。
- `native_libraries.tsv`：所有 SO 的大小、Build ID、SHA-256。
- `native_exported_symbols.tsv`：可直接确认的动态导出符号。
- `il2cpp_class_index.tsv`：已有 IL2CPP 类名统一索引及来源。
- `il2cpp_changed_duplicates.tsv`：仅记录语义发生差异的重复类；未变化者不重复写入。
- `index_manifest.json`：统计、证据边界和限制。

## 证据边界

当前 APK 中没有匹配的明文 metadata，因此不能声称 `il2cpp_class_index.tsv` 是从 APK 676 新解密所得。它合并的是仓库既有运行时方法 dump 与先前 metadata 报告。要建立 APK 676 的精确类/方法/地址映射，仍需同构建的已解密 `global-metadata.dat`。

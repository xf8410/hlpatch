# Ramen planner evidence checkpoint

Updated: 2026-07-19

This file is the repository-side compact checkpoint for Ramen scenario 14 reverse engineering. Keep large runtime JSON, minified method indexes, and full disassembly out of this document.

## Safety and query policy

- Never call `/il2cpp/classes`; it has crashed the phone.
- Query one exact class/method/field at a time.
- Limit disassembly requests to 256–512 bytes initially.
- Never paste or print a complete minified JSON line. Extract a bounded 2–4 KB fragment into a temporary file first.
- After a useful query, summarize the evidence here and push it. Future analysis should read this file rather than reopening large JSON artifacts.
- Unknown semantics stay `raw` or `unknown`; do not infer from names alone.

## Published diagnostics

- `/debug/ramen_planner_state` schema v2:
  - inventory and acquisition-state raw arrays;
  - nested `FeelingTurnArray` decoder;
  - skips `_gaugeGainCountDict` and `GetGainCount`.
- `/debug/ramen_participants` schema v1:
  - equipped deck roster;
  - final `TrainingPartnerArray` for commands 101/102/103/105/106;
  - exact deck classification only by equipped support position;
  - non-deck entries remain `unknown_nondeck`.
- Current plugin release: v3.24.22, commit `82048fa`; Actions #446 run `29652139308` succeeded.

## Confirmed runtime mechanics

- Finished item inventory is a shared 10-slot FIFO/ring.
- New items append in production order; overflow removes the oldest item.
- Acquisition threshold reset is 7. In the strict sample, excess progress was discarded rather than carried over.
- Current-turn final progress vectors are exposed by `FeelingReduceTurnInfoArray[].FeelingTurnArray`.
- Strict sample: base vector `(3,3,4)` and speed vector `(5,3,4)`; two visible scenario partners and no deck supports were present in speed, strongly suggesting +2 to the command-linked feeling, but this is not yet a general formula.
- Races also advance acquisition gauges; exact race vector/result dependency is not yet measured.

## Participant snapshot evidence

Deck positions in the first snapshot:

- 1: support card 30305, chara 9001 (Tazuna), Link/friend card
- 2: 30275
- 3: 30242
- 4: 30226
- 5: 30173
- 6: 30227

Final non-deck partner IDs observed in one turn:

- 1022, 1060, 1077, 1080, 1120

They appeared alongside five equipped supports; deck position 3 was absent that turn. A single final-participant snapshot cannot prove a six-member scenario roster because one member may be absent.

## MDB key definitions

`single_mode_14_deck_info` has only five rows:

- deck_type 1: support cards 10021 and 30021 -> chara 9001
- deck_type 1: support cards 10083 and 30052 -> chara 9008
- deck_type 2: support card 30305 -> chara 9001

`single_mode_special_chara` for scenario 14 has seven chara IDs:

- 1022 Fine Motion
- 1058 Meisho Doto
- 1060 Nice Nature
- 1077 Narita Top Road
- 1120 Calstone Light O
- 9001 Tazuna
- 9008 Light Hello

Runtime IDs 1022/1060/1077/1120 directly match this definition. Runtime 1080 (Transcend) is not in the scenario-14 special list, which is strong evidence of dynamically supplied backfill. `single_mode_npc` is a general race-attribute table with multiple rows per character, not a demonstrated backfill-pool definition.

## Static reverse-engineering status

Exact method candidates found in local bounded method indexes:

- `Gallop.SingleMode.ScenarioRamen.PartsSingleModeScenarioRamenCheckPointCutDirector.GetShuffledScenarioLinkCharaIdArray`
  - current phone runtime address: `0x73384eed88` (the older method-index address differed, so raw addresses are not stable across builds/runtime layouts)
  - static, one parameter
  - complete boundary now observed: continuation from method offset `0x180` returns at continuation offset `0x4c`, so this method ends around total offset `0x1d0`; bytes after continuation offset `0x50` belong to the next function
  - no reported integer constants occur in the method, and it has no visible fixed six-member construction in the captured body
  - immediately before its final collection/result handling it calls runtime target `0x73384edc30` from total method offset `0x18c`; this is the next exact helper to identify
  - because this method belongs to the checkpoint cut director and receives one parameter, current evidence favors transforming/shuffling an already supplied scenario-link character collection for presentation, not defining the run-start backfill pool; keep this as a strong exclusion candidate until the helper is identified
- `Gallop.CharaInfoListContextFactory.CreateFromAllScenarioLinkCharaDownload`
  - address: `0x7338f68a68`
  - generic scenario-link download context; no Ramen initialization call-chain evidence yet.
- Generic methods `IsEnableScenarioLinkChara` and `IsEnableScenarioLinkSupportCardChara` exist, but are not yet linked to Ramen NPC selection.
- `WorkSingleModeScenarioRamen` exposes DataSet apply methods only; roster initialization is likely in a higher-level/common run-start path.
- Repository artifact provenance was rechecked offline. `/home/agora/hlpatch-reverse/artifacts/reverse/libil2cpp.so` exists and is a 218,715,344-byte ARM64 ELF. Repository reports label the game package version as v2.28.5; this does **not** prove the SO is obsolete, because package/app version and internal/native-library version labels are different concepts. The earlier comparison found only a generic 16-byte prologue at one location and divergence afterward; that is insufficient to identify the method or reject the SO. Runtime virtual addresses also require correct module slide/RVA mapping. Treat this SO as a potentially matching artifact until its build identity or a longer unique method signature is verified.

## Next exact query

### Unsafe query recorded

The exact-address request below caused the game to crash and must not be repeated:

```text
/il2cpp/disassemble_addr?addr=0x73384edc30&bytes=384
```

Possible causes include an unsafe/non-method boundary, unreadable span, or the address endpoint itself crossing a protected boundary. No semantic conclusion may be drawn from the crash. Mark runtime address `0x73384edc30` as unsafe for direct reads.

## Next exact query

Stop following raw call targets by address. The cut-director method is not required for the planner and is excluded from further phone probing as a presentation-layer candidate.

Offline artifact audit and one named-repository query:

- The protected historical workspace contains `/home/agora/hlpatch-reverse/artifacts/reverse/libil2cpp.so` (218,715,344 bytes). Its report's v2.28.5 label is the game package version and does not establish that the SO is outdated. A generic 16-byte prologue match followed by divergence only proves that the chosen offline offset was not yet mapped/identified correctly; it does not disqualify the binary.
- The reports describe a parsed 44 MB v31 metadata artifact, but no corresponding `global-metadata.dat` file is presently present in `/home/agora` or `/tmp`; only derived reports/method indexes remain.
- Existing named method index identifies `Gallop.SingleModeTrainingPartnerRepository` with overloads `Get()` / `Get(1)` / `Get(2)`, plus `ConvertToTrainingCommandIdList`, `get_ScenarioId`, `get_WorkSingleModeHomeInfoData`, `get_SingleModeCommandInfoDataArray`, `get_WorkSingleModeCharaData`, and `get_EvaluationList`. This repository is a confirmed named access layer for final training partners, not yet a run-start roster constructor.

One exact entity query adds a useful identity distinction: `SingleModeTrainingPartnerEntity` and `SingleModeTrainingPartnerUniqueCharaEntity` both expose separate `get_PartnerId` and `get_CharaId`, along with command-location getters. Therefore the current runtime equality between non-deck `partner_id` and MDB `chara_id` is an observed value relationship for these samples, not a type-level guarantee. Future diagnostics should read both named getters/fields before generalizing identity mapping.

Offline SO mapping result:

- Repository documentation gives its historical conversion as `load_base=0x7330ef37c4`, `vaddr=dump_addr-load_base`, then PT_LOAD file mapping. Applying it to the indexed cut-director address lands on bytes that are clearly mid-function, so the recorded method addresses and this on-disk SO still lack a proven common address epoch/slide.
- A full streaming search of the 218 MB SO found 1,279 copies of the generic 16-byte prologue. None had the phone method's next fixed instruction pair (`mov w19,w0; tbnz w8,#0,...`) at the expected offset. Thus the earlier prologue hit was non-unique. This still does not prove the SO is stale; it proves signature-based identity has not been established and runtime code may differ by relocation/patch/build.
- Existing named index adds `SingleModeTrainingPartnerEtcCharaEntity`, the mutable non-deck entity with setters for `PartnerId`, `CharaId`, `TrainingCommandId`, and `TrainingBaseCommandId`. The roster constructor must populate this entity or its backing data, making setter callers/write sites the correct static target rather than final `TrainingPartnerArray` getters.

Write-side/xref query result:

- Existing `dump_all_methods_*.json` files are declaration indexes only (class, method name, address, parameter count, return type). `SingleModeTrainingPartnerEtcCharaEntity` occurs only at its declaration; no caller/xref data exists in these indexes or compact reports.
- Exact name searches for `CreateTrainingPartner`/`SetupTrainingPartner` resolve UI/detail/setup methods, not the roster constructor. No indexed method named `CreateEtcChara` or `LotteryTrainingPartner` exists.
- `SingleModeRamenAPI.SendStart` exists with seven parameters, but the scenario-specific `ObscuredSingleModeRamenDataSetStart` has only `AutoSelectInfo`, `AutoSelectSetInfo`, and `IsCheckedUrafEvent`; it contains no NPC roster. This is evidence that roster/partner setup is handled by common single-mode start data or another common layer, not Ramen's three-field start payload.
- The current artifacts therefore cannot produce setter callers without either (a) correctly mapped/disassembled machine code plus xrefs or (b) a richer runtime metadata/code dump. Unsafe raw-address phone probing remains prohibited.

Common start-layer query result:

- `WorkSingleModeData.ApplySingleModeStartResponse` exists as a named instance method with one response parameter (indexed address `0x7339dc8ab8`). `ApplyCommonResponse` exists with seven parameters (`0x7339dc83ac`). These are the first credible common run-start entry points above Ramen's three-field start payload.
- `WorkSingleModeData` also exposes `CreateCharacter`, `ApplyCharacter`, `ApplyHomeInfo`, and `AddRaceConditionByStartInfo`; `WorkSingleModeHomeInfo.Apply` and scenario-specific `ApplyRamenCommandInfo` operate on home/command state.
- No common standalone `*DataSetStart` payload class is present in the declaration indexes beyond Ramen/Breeders scenario payloads. This supports the hypothesis that partner roster/evaluation data is embedded in the generic start response consumed by `ApplySingleModeStartResponse`/`ApplyCommonResponse`.

Additional exact named targets:

- `SingleModeUtils.IsEnableScenarioLinkChara(int?, int?)` is a static two-parameter boolean method at indexed address `0x733949b830`.
- `SingleModeUtils.IsEnableScenarioLinkSupportCardChara(int?, int?)` is a static two-parameter boolean method at `0x733949b968`.
- Compiler-generated predicates `<IsEnableScenarioLinkChara>b__0` and `<IsEnableScenarioLinkSupportCardChara>b__0` confirm both methods filter/search a collection rather than being simple constants. Parameter types remain unknown in the current index.
- These are stronger Link-conflict/filter candidates than the checkpoint cut-director shuffle method, but no caller link to `ApplySingleModeStartResponse` is yet present.

`single_mode_14_deck_info.deck_type` evidence:

- `MasterSingleMode14DeckInfo` exposes only generic unload/storage access in the current method index; no semantic getter or direct code caller is named. The table remains only five exact support-card rows.
- Exact cross-table query shows these five cards are also used by scenario-14 outing effects. All old Tazuna/Light Hello cards (10021/30021/10083/30052, deck_type 1) grant `special_feeling_num=1` on outing story steps 1..5. New Tazuna 30305 (deck_type 2) grants `special_feeling_num=2` on all five steps.
- Therefore deck_type is strongly associated with the card's Ramen-specific bonus tier/handling, not evidence for selecting an NPC roster. Do not use deck_type to infer a Tazuna-vs-Light-Hello NPC slot.

Offline tooling/address result:

- An isolated Alpine `binutils`/`objdump` was downloaded under `/tmp/binutils-offline` without installing system packages.
- Existing reports clarify that historical method-dump addresses may point inside/near method bodies and require a historical load base plus ELF PT_LOAD mapping. Directly treating each indexed address as an entry is invalid.
- The old SO can now be disassembled safely offline in bounded ranges, but exact method identity still needs boundary/call-chain validation before semantic claims.

Next exact static target: establish the generic start response parameter type/fields for `WorkSingleModeData.ApplySingleModeStartResponse`; then look for evaluation/partner arrays feeding `SingleModeTrainingPartnerEtcCharaEntity`. Separately identify callers of the two `SingleModeUtils` Link filters. Do not inspect broad start-dialog UI classes.

Additional runtime tests:

- race before/after: inventory and three Remain values;
- pull-support item before/after: `/debug/ramen_participants` before action execution;
- verify endpoint participant counts against the UI.

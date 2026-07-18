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
- Repository artifact provenance was rechecked offline. `/home/agora/hlpatch-reverse/artifacts/reverse/libil2cpp.so` exists and is a 218,715,344-byte ARM64 ELF, but repository reports identify it as game v2.28.5. The current phone method prefix diverges from this ELF after a generic 16-byte prologue, so this old SO must not be used to assign current addresses/control flow. The repository's metadata-derived reports and named method indexes remain useful for names/schema hypotheses, but current machine-code conclusions require a matching build artifact.

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

- The protected historical workspace does contain `/home/agora/hlpatch-reverse/artifacts/reverse/libil2cpp.so` (218,715,344 bytes), but repository provenance labels it game v2.28.5. A generic 16-byte prologue matches the current phone method, then diverges; therefore it is not a matching current binary and cannot support current control-flow claims.
- The reports describe a parsed 44 MB v31 metadata artifact, but no corresponding `global-metadata.dat` file is presently present in `/home/agora` or `/tmp`; only derived reports/method indexes remain.
- Existing named method index identifies `Gallop.SingleModeTrainingPartnerRepository` with overloads `Get()` / `Get(1)` / `Get(2)`, plus `ConvertToTrainingCommandIdList`, `get_ScenarioId`, `get_WorkSingleModeHomeInfoData`, `get_SingleModeCommandInfoDataArray`, `get_WorkSingleModeCharaData`, and `get_EvaluationList`. This repository is a confirmed named access layer for final training partners, not yet a run-start roster constructor.

One exact entity query adds a useful identity distinction: `SingleModeTrainingPartnerEntity` and `SingleModeTrainingPartnerUniqueCharaEntity` both expose separate `get_PartnerId` and `get_CharaId`, along with command-location getters. Therefore the current runtime equality between non-deck `partner_id` and MDB `chara_id` is an observed value relationship for these samples, not a type-level guarantee. Future diagnostics should read both named getters/fields before generalizing identity mapping.

Next offline-only query: inspect the exact named classes around training-partner *creation/setup* (not the final repository getter) in existing method indexes, one class at a time. Do not infer current code from the v2.28.5 SO. If no named constructor can be established, the real blocker is a matching current SO/runtime metadata dump—not entering a育成局.

Additional runtime tests:

- race before/after: inventory and three Remain values;
- pull-support item before/after: `/debug/ramen_participants` before action execution;
- verify endpoint participant counts against the UI.

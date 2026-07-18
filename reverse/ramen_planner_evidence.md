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
  - address: `0x7339ad6d88`
  - static, one parameter
  - likely checkpoint presentation shuffling; not yet connected to run-start NPC roster creation.
- `Gallop.CharaInfoListContextFactory.CreateFromAllScenarioLinkCharaDownload`
  - address: `0x7338f68a68`
  - generic scenario-link download context; no Ramen initialization call-chain evidence yet.
- Generic methods `IsEnableScenarioLinkChara` and `IsEnableScenarioLinkSupportCardChara` exist, but are not yet linked to Ramen NPC selection.
- `WorkSingleModeScenarioRamen` exposes DataSet apply methods only; roster initialization is likely in a higher-level/common run-start path.

## Next exact query

First exclude or confirm the checkpoint-presentation candidate with only 384 bytes:

```text
http://127.0.0.1:18765/il2cpp/disassemble?class=PartsSingleModeScenarioRamenCheckPointCutDirector&method=GetShuffledScenarioLinkCharaIdArray&bytes=384
```

After the result:

1. Save only a compact call/constant summary here and push it.
2. If presentation-only, mark it excluded and make one next exact query.
3. Do not perform global class or method scans on the phone.

Additional runtime tests:

- race before/after: inventory and three Remain values;
- pull-support item before/after: `/debug/ramen_participants` before action execution;
- verify endpoint participant counts against the UI.

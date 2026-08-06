# A-stage compiler diagnostics

```text
   Compiling flate2 v1.1.9
   Compiling fallible-streaming-iterator v0.1.9
   Compiling base64 v0.22.1
   Compiling bitflags v2.13.1
   Compiling fallible-iterator v0.3.0
   Compiling ureq v2.12.1
   Compiling rusqlite v0.31.0
   Compiling hachimi_ura v3.27.4 (/home/runner/work/hlpatch/hlpatch/hachimi_ura_plugin)
warning: unused doc comment
    --> src/lib.rs:5475:5
     |
5475 |       /// ★ v3.24.14: position → bond_threshold from MasterDB unique_effect
     |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
5476 | /     let mut bond_threshold_by_position: std::collections::HashMap<i32, i32> =
5477 | |         std::collections::HashMap::new();
     | |_________________________________________- rustdoc does not generate documentation for statements
     |
     = help: use `//` for a plain comment
     = note: `#[warn(unused_doc_comments)]` (part of `#[warn(unused)]`) on by default

warning: unused doc comment
    --> src/lib.rs:5478:5
     |
5478 |       /// ★ v3.24.15: position → support_card_type (1=普通, 2=友人, 3=团体)
     |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
5479 | /     let mut support_card_type_by_position: std::collections::HashMap<i32, i32> =
5480 | |         std::collections::HashMap::new();
     | |_________________________________________- rustdoc does not generate documentation for statements
     |
     = help: use `//` for a plain comment

error[E0308]: mismatched types
    --> src/lib.rs:7527:73
     |
7527 |     let _parsed_request_uri = parse_request_uri(req).unwrap_or_else(|_| full_uri.clone());
     |                                                                         ^^^^^^^^^^^^^^^^ expected `String`, found `&str`
     |
help: try using a conversion method
     |
7527 -     let _parsed_request_uri = parse_request_uri(req).unwrap_or_else(|_| full_uri.clone());
7527 +     let _parsed_request_uri = parse_request_uri(req).unwrap_or_else(|_| full_uri.to_string());
     |

error[E0282]: type annotations needed for `*mut _`
     --> src/lib.rs:22441:13
      |
22441 |         let mut found = ptr::null_mut();
      |             ^^^^^^^^^
...
22446 |                 if !found.is_null() { return ptr::null_mut(); }
      |                           ------- cannot call a method on a raw pointer with an unknown pointee type
      |
help: consider giving `found` an explicit type, where the placeholder `_` is specified
      |
22441 |         let mut found: *mut T = ptr::null_mut();
      |                      ++++++++

warning: unused variable: `chara_id`
    --> src/lib.rs:2713:9
     |
2713 |     let chara_id = call_getter_int(chara_data_class, chara_obj, "get_CardId");
     |         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_chara_id`
     |
     = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `ramen_gauge_gains`
    --> src/lib.rs:3880:5
     |
3880 |     ramen_gauge_gains: &std::collections::HashMap<i32, i32>, // 各训练素材进度增益
     |     ^^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ramen_gauge_gains`

warning: unused variable: `next_turn_race`
    --> src/lib.rs:3882:5
     |
3882 |     next_turn_race: bool, // 下回合是否比赛回合 [MDB single_mode_turn]
     |     ^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_next_turn_race`

warning: variable does not need to be mutable
    --> src/lib.rs:4841:9
     |
4841 |     let mut ramen_selectable_region_ids_derived_json = String::new();
     |         ----^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`
     |
     = note: `#[warn(unused_mut)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
    --> src/lib.rs:4843:9
     |
4843 |     let mut ramen_region_pool_phase_derived_json = String::new();
     |         ----^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
     |         |
     |         help: remove this `mut`

warning: unused variable: `limit_break`
    --> src/lib.rs:4747:9
     |
4747 |     let limit_break = read_obscured_int_at(chara_obj, 108); // LimitBreakCount
     |         ^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_limit_break`

warning: variable does not need to be mutable
    --> src/lib.rs:7160:37
     |
7160 |         let run_query = |sql: &str, mut on_row: &mut dyn FnMut(&[*const u8])| -> i32 {
     |                                     ----^^^^^^
     |                                     |
     |                                     help: remove this `mut`

warning: unused variable: `api`
     --> src/lib.rs:11550:9
      |
11550 |     let api = &*API;
      |         ^^^ help: if this is intentional, prefix it with an underscore: `_api`

warning: unused variable: `api`
     --> src/lib.rs:11574:9
      |
11574 |     let api = &*API;
      |         ^^^ help: if this is intentional, prefix it with an underscore: `_api`

warning: unnecessary `unsafe` block
     --> src/lib.rs:11846:25
      |
11812 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
11846 |                         unsafe { get_config() }.http_port
      |                         ^^^^^^ unnecessary `unsafe` block
      |
      = note: `#[warn(unused_unsafe)]` (part of `#[warn(unused)]`) on by default

warning: unnecessary `unsafe` block
     --> src/lib.rs:11992:23
      |
11812 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
11992 |             let cfg = unsafe { get_config() };
      |                       ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:11995:13
      |
11812 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
11995 |             unsafe {
      |             ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12018:37
      |
11812 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12018 |                 let changed = f(ui, unsafe { GUI_HOST_BUF.as_mut_ptr() as *mut c_char }, 64);
      |                                     ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12020:21
      |
11812 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12020 |                     unsafe {
      |                     ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12045:37
      |
11812 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12045 |                 let changed = f(ui, unsafe { GUI_PORT_BUF.as_mut_ptr() as *mut c_char }, 8);
      |                                     ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12047:21
      |
11812 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12047 |                     unsafe {
      |                     ^^^^^^ unnecessary `unsafe` block

warning: variable does not need to be mutable
     --> src/lib.rs:12708:9
      |
12708 |     let mut dream_overflow = false;
      |         ----^^^^^^^^^^^^^^
      |         |
      |         help: remove this `mut`

warning: value assigned to `min_level` is never read
     --> src/lib.rs:12884:9
      |
12884 |         min_level = 0;
      |         ^^^^^^^^^^^^^
      |
      = help: maybe it is overwritten before being read?
      = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
     --> src/lib.rs:15408:20
      |
15408 |                 Ok(mut stmt) => stmt.column_names().iter().map(|s| s.to_string()).collect(),
      |                    ----^^^^
      |                    |
      |                    help: remove this `mut`

warning: value assigned to `all_events_count` is never read
     --> src/lib.rs:20362:37
      |
20362 |       let mut all_events_count: i32 = 0;
      |                                       ^ this value is reassigned later and never used
...
20401 | /             all_events_count = conn
20402 | |                 .query_row("SELECT COUNT(*) FROM single_mode_story_data", [], |r| {
20403 | |                     r.get(0)
20404 | |                 })
20405 | |                 .unwrap_or(0);
      | |_____________________________- `all_events_count` is overwritten here before the previous value is read

warning: unused variable: `ds_class`
     --> src/lib.rs:20891:9
      |
20891 |     let ds_class = get_class_from_object(ds_obj);
      |         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ds_class`

warning: unused variable: `is_literal`
     --> src/lib.rs:21911:13
      |
21911 |         let is_literal = match field_get_flags {
      |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_is_literal`

Some errors have detailed explanations: E0282, E0308.
For more information about an error, try `rustc --explain E0282`.
warning: `hachimi_ura` (lib) generated 24 warnings
error: could not compile `hachimi_ura` (lib) due to 2 previous errors; 24 warnings emitted
```

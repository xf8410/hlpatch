# Unified storage compiler diagnostics

```text
    Updating crates.io index
 Downloading crates ...
  Downloaded adler2 v2.0.1
  Downloaded find-msvc-tools v0.1.9
  Downloaded bitflags v2.13.1
  Downloaded cc v1.4.0
  Downloaded base64 v0.22.1
  Downloaded ahash v0.8.12
  Downloaded form_urlencoded v1.2.2
  Downloaded crc32fast v1.5.0
  Downloaded icu_properties v2.2.0
  Downloaded fallible-streaming-iterator v0.1.9
  Downloaded getrandom v0.2.17
  Downloaded fallible-iterator v0.3.0
  Downloaded hashbrown v0.14.5
  Downloaded icu_provider v2.2.0
  Downloaded miniz_oxide v0.8.9
  Downloaded displaydoc v0.2.7
  Downloaded ureq v2.12.1
  Downloaded idna_adapter v1.2.2
  Downloaded cfg-if v1.0.4
  Downloaded utf8_iter v1.0.4
  Downloaded percent-encoding v2.3.2
  Downloaded subtle v2.6.1
  Downloaded hashlink v0.9.1
  Downloaded litemap v0.8.2
  Downloaded potential_utf v0.1.5
  Downloaded yoke v0.8.3
  Downloaded rustls-pki-types v1.15.1
  Downloaded zerovec-derive v0.11.3
  Downloaded zerofrom-derive v0.1.7
  Downloaded version_check v0.9.5
  Downloaded webpki-roots v0.26.11
  Downloaded untrusted v0.9.0
  Downloaded stable_deref_trait v1.2.1
  Downloaded synstructure v0.13.2
  Downloaded writeable v0.6.3
  Downloaded yoke-derive v0.8.2
  Downloaded simd-adler32 v0.3.10
  Downloaded once_cell v1.21.4
  Downloaded icu_properties_data v2.2.0
  Downloaded zerofrom v0.1.8
  Downloaded tinystr v0.8.3
  Downloaded unicode-ident v1.0.24
  Downloaded pkg-config v0.3.33
  Downloaded smallvec v1.15.2
  Downloaded log v0.4.33
  Downloaded quote v1.0.47
  Downloaded shlex v2.0.1
  Downloaded zeroize v1.9.0
  Downloaded flate2 v1.1.9
  Downloaded serde_core v1.0.229
  Downloaded serde v1.0.229
  Downloaded icu_normalizer v2.2.0
  Downloaded icu_collections v2.2.0
  Downloaded rustls-webpki v0.103.13
  Downloaded icu_locale_core v2.2.0
  Downloaded icu_normalizer_data v2.2.0
  Downloaded zerotrie v0.2.4
  Downloaded proc-macro2 v1.0.107
  Downloaded url v2.5.8
  Downloaded idna v1.1.0
  Downloaded zerovec v0.11.6
  Downloaded rusqlite v0.31.0
  Downloaded vcpkg v0.2.15
  Downloaded webpki-roots v1.0.9
  Downloaded zerocopy v0.8.55
  Downloaded syn v3.0.3
  Downloaded syn v2.0.119
  Downloaded rustls v0.23.43
  Downloaded libc v0.2.189
  Downloaded ring v0.17.14
  Downloaded libsqlite3-sys v0.28.0
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling stable_deref_trait v1.2.1
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.9
   Compiling cfg-if v1.0.4
   Compiling writeable v0.6.3
   Compiling cc v1.4.0
   Compiling litemap v0.8.2
   Compiling icu_normalizer_data v2.2.0
   Compiling libc v0.2.189
   Compiling icu_properties_data v2.2.0
   Compiling utf8_iter v1.0.4
   Compiling ring v0.17.14
   Compiling zerocopy v0.8.55
   Compiling version_check v0.9.5
   Compiling smallvec v1.15.2
   Compiling zeroize v1.9.0
   Compiling once_cell v1.21.4
   Compiling rustls-pki-types v1.15.1
   Compiling ahash v0.8.12
   Compiling syn v2.0.119
   Compiling syn v3.0.3
   Compiling getrandom v0.2.17
   Compiling synstructure v0.13.2
   Compiling zerovec-derive v0.11.3
   Compiling zerofrom-derive v0.1.7
   Compiling yoke-derive v0.8.2
   Compiling zerofrom v0.1.8
   Compiling displaydoc v0.2.7
   Compiling crc32fast v1.5.0
   Compiling vcpkg v0.2.15
   Compiling yoke v0.8.3
   Compiling zerovec v0.11.6
   Compiling zerotrie v0.2.4
   Compiling untrusted v0.9.0
   Compiling pkg-config v0.3.33
   Compiling tinystr v0.8.3
   Compiling potential_utf v0.1.5
   Compiling icu_locale_core v2.2.0
   Compiling icu_collections v2.2.0
   Compiling libsqlite3-sys v0.28.0
   Compiling simd-adler32 v0.3.10
   Compiling percent-encoding v2.3.2
   Compiling rustls v0.23.43
   Compiling adler2 v2.0.1
   Compiling miniz_oxide v0.8.9
   Compiling icu_provider v2.2.0
   Compiling icu_properties v2.2.0
   Compiling icu_normalizer v2.2.0
   Compiling rustls-webpki v0.103.13
   Compiling idna_adapter v1.2.2
   Compiling form_urlencoded v1.2.2
   Compiling idna v1.1.0
   Compiling hashbrown v0.14.5
   Compiling webpki-roots v1.0.9
   Compiling log v0.4.33
   Compiling subtle v2.6.1
   Compiling webpki-roots v0.26.11
   Compiling hashlink v0.9.1
   Compiling url v2.5.8
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

warning: unused variable: `resp`
    --> src/lib.rs:9269:13
     |
9269 |         let resp = format!(
     |             ^^^^ help: if this is intentional, prefix it with an underscore: `_resp`

warning: unused variable: `api`
     --> src/lib.rs:11560:9
      |
11560 |     let api = &*API;
      |         ^^^ help: if this is intentional, prefix it with an underscore: `_api`

warning: unused variable: `api`
     --> src/lib.rs:11584:9
      |
11584 |     let api = &*API;
      |         ^^^ help: if this is intentional, prefix it with an underscore: `_api`

warning: unnecessary `unsafe` block
     --> src/lib.rs:11856:25
      |
11822 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
11856 |                         unsafe { get_config() }.http_port
      |                         ^^^^^^ unnecessary `unsafe` block
      |
      = note: `#[warn(unused_unsafe)]` (part of `#[warn(unused)]`) on by default

warning: unnecessary `unsafe` block
     --> src/lib.rs:12002:23
      |
11822 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12002 |             let cfg = unsafe { get_config() };
      |                       ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12005:13
      |
11822 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12005 |             unsafe {
      |             ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12028:37
      |
11822 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12028 |                 let changed = f(ui, unsafe { GUI_HOST_BUF.as_mut_ptr() as *mut c_char }, 64);
      |                                     ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12030:21
      |
11822 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12030 |                     unsafe {
      |                     ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12055:37
      |
11822 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12055 |                 let changed = f(ui, unsafe { GUI_PORT_BUF.as_mut_ptr() as *mut c_char }, 8);
      |                                     ^^^^^^ unnecessary `unsafe` block

warning: unnecessary `unsafe` block
     --> src/lib.rs:12057:21
      |
11822 |     unsafe {
      |     ------ because it's nested under this `unsafe` block
...
12057 |                     unsafe {
      |                     ^^^^^^ unnecessary `unsafe` block

warning: variable does not need to be mutable
     --> src/lib.rs:12718:9
      |
12718 |     let mut dream_overflow = false;
      |         ----^^^^^^^^^^^^^^
      |         |
      |         help: remove this `mut`

warning: value assigned to `min_level` is never read
     --> src/lib.rs:12894:9
      |
12894 |         min_level = 0;
      |         ^^^^^^^^^^^^^
      |
      = help: maybe it is overwritten before being read?
      = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

warning: variable does not need to be mutable
     --> src/lib.rs:15418:20
      |
15418 |                 Ok(mut stmt) => stmt.column_names().iter().map(|s| s.to_string()).collect(),
      |                    ----^^^^
      |                    |
      |                    help: remove this `mut`

warning: value assigned to `all_events_count` is never read
     --> src/lib.rs:20372:37
      |
20372 |       let mut all_events_count: i32 = 0;
      |                                       ^ this value is reassigned later and never used
...
20411 | /             all_events_count = conn
20412 | |                 .query_row("SELECT COUNT(*) FROM single_mode_story_data", [], |r| {
20413 | |                     r.get(0)
20414 | |                 })
20415 | |                 .unwrap_or(0);
      | |_____________________________- `all_events_count` is overwritten here before the previous value is read

warning: unused variable: `ds_class`
     --> src/lib.rs:20901:9
      |
20901 |     let ds_class = get_class_from_object(ds_obj);
      |         ^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_ds_class`

warning: unused variable: `is_literal`
     --> src/lib.rs:21921:13
      |
21921 |         let is_literal = match field_get_flags {
      |             ^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_is_literal`

error[E0382]: borrow of moved value: `current_session`
     --> src/lib.rs:22805:21
      |
22796 |     let current_session = session.ok();
      |         --------------- move occurs because `current_session` has type `Option<String>`, which does not implement the `Copy` trait
...
22801 |     let session_json = current_session.map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());
      |                                        --------------------------------------------------- `current_session` moved due to this method call
...
22805 |         writable && current_session.is_some(), json_escape(&root.to_string_lossy()),
      |                     ^^^^^^^^^^^^^^^ value borrowed here after move
      |
note: `Option::<T>::map` takes ownership of the receiver `self`, which moves `current_session`
     --> /rustc/8bab26f4f68e0e26f0bb7960be334d5b520ea452/library/core/src/option.rs:1157:27
help: consider calling `.as_ref()` to borrow the value's contents
      |
22801 |     let session_json = current_session.as_ref().map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());
      |                                       +++++++++
help: consider calling `.as_mut()` to mutably borrow the value's contents
      |
22801 |     let session_json = current_session.as_mut().map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());
      |                                       +++++++++
help: you can `clone` the value and consume it, but this might not be your desired behavior
      |
22801 |     let session_json = current_session.clone().map(|value| format!("\"{}\"", json_escape(&value))).unwrap_or_else(|| "null".to_string());
      |                                       ++++++++

For more information about this error, try `rustc --explain E0382`.
warning: `hachimi_ura` (lib) generated 25 warnings
error: could not compile `hachimi_ura` (lib) due to 1 previous error; 25 warnings emitted
```

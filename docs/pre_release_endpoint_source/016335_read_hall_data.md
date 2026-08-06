# `read_hall_data`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `16335`

```rust
unsafe fn read_hall_data() -> String {
    if API.is_null() {
        return r#"{"error":"api_null"}"#.to_string();
    }
    let image = match get_image() {
        img if !img.is_null() => img,
        _ => return r#"{"error":"image_null"}"#.to_string(),
    };

    // 1. Get WDM singleton
    let wdm_class = find_class(
        image,
        to_cstr("Gallop").as_ptr(),
        to_cstr("WorkDataManager").as_ptr(),
    );
    if wdm_class.is_null() {
        return r#"{"error":"no_wdm"}"#.to_string();
    }
    let wdm_inst = get_singleton(wdm_class);
    if wdm_inst.is_null() {
        return r#"{"error":"no_wdm_inst"}"#.to_string();
    }

    // 2. Get WorkTrainedCharaData from WDM
    let wtcd_inst = call_getter_ref(wdm_class, wdm_inst, "get_TrainedCharaData");
    if wtcd_inst.is_null() {
        ura_log(1, "/hall: get_TrainedCharaData returned null");
        return r#"{"error":"no_trained_chara_data"}"#.to_string();
    }
    ura_log(2, "/hall: got WorkTrainedCharaData instance");

    // 3. Find WorkTrainedCharaData class for calling get_List
    let wtcd_class = find_class_by_short_name(image, "WorkTrainedCharaData");

    // 4. Get List<TrainedCharaData> from WorkTrainedCharaData
    let list_obj = call_getter_ref(wtcd_class, wtcd_inst, "get_List");
    if list_obj.is_null() {
        ura_log(1, "/hall: get_List returned null");
        return r#"{"error":"no_list"}"#.to_string();
    }

    // 5. Read List<TrainedCharaData> internals
    // List<T> IL2CPP layout (64-bit):
    //   +0x00: Il2CppObject header (16 bytes)
    //   +0x10: _items (Il2CppArray* pointer, 8 bytes)
    //   +0x18: _size (int32, 4 bytes)
    let list_base = list_obj as *const u8;
    let items_arr = std::ptr::read_unaligned::<*mut c_void>(
        list_base.add(IL2CPP_LIST_ARRAY_OFF) as *const *mut c_void
    );
    let list_size =
        std::ptr::read_unaligned::<usize>(list_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize)
            as i32;

    if items_arr.is_null() || list_size <= 0 {
        ura_log(1, &format!("/hall: List null or empty, size={}", list_size));
        return format!(r#"{{"error":"empty_list","list_size":{}}}"#, list_size);
    }
    ura_log(2, &format!("/hall: List has {} entries", list_size));

    // 6. Find TrainedCharaData class
    let tcd_class = find_class_by_short_name(image, "TrainedCharaData");
    if tcd_class.is_null() {
        ura_log(1, "/hall: TrainedCharaData class not found");
        return r#"{"error":"no_tcd_class"}"#.to_string();
    }

    // 7. Read array elements from List._items
    // Il2CppArray layout: +0x18: max_length (usize), +0x20: data[0]
    let arr_base = items_arr as *const u8;
    let arr_len =
        std::ptr::read_unaligned::<usize>(arr_base.add(IL2CPP_LIST_COUNT_OFF) as *const usize);

    let mut entries = Vec::new();
    let count = std::cmp::min(list_size as usize, std::cmp::min(arr_len, 200));

    for i in 0..count {
        let elem_ptr = std::ptr::read_unaligned::<*mut c_void>(
            arr_base.add(IL2CPP_LIST_ITEMS_OFF + i * IL2CPP_LIST_ITEM_SIZE) as *const *mut c_void,
        );
        if elem_ptr.is_null() {
            continue;
        }

        // Read fields via getter methods
        let card_id = call_getter_int(tcd_class, elem_ptr, "get_CardId");
        let speed = call_getter_int(tcd_class, elem_ptr, "get_Speed");
        let stamina = call_getter_int(tcd_class, elem_ptr, "get_Stamina");
        let power = call_getter_int(tcd_class, elem_ptr, "get_Power");
        let guts = call_getter_int(tcd_class, elem_ptr, "get_Guts");
        let wiz = call_getter_int(tcd_class, elem_ptr, "get_Wiz");
        let rank_score = call_getter_int(tcd_class, elem_ptr, "get_RankScore");
        let rank = call_getter_int(tcd_class, elem_ptr, "get_Rank");
        let scenario_id = call_getter_obscured_int(tcd_class, elem_ptr, "get_ScenarioId");
        let fans = call_getter_int(tcd_class, elem_ptr, "get_Fans");
        let rarity = call_getter_obscured_int(tcd_class, elem_ptr, "get_Rarity");

        // Skip entries with no meaningful data
        if speed <= 0 && stamina <= 0 && rank_score <= 0 {
            continue;
        }

        entries.push(format!(
            r#"{{"idx":{},"card_id":{},"speed":{},"stamina":{},"power":{},"guts":{},"wiz":{},"rank_score":{},"rank":{},"scenario_id":{},"fans":{},"rarity":{}}}"#,
            i, card_id, speed, stamina, power, guts, wiz, rank_score, rank, scenario_id, fans, rarity
        ));
    }

    if entries.is_empty() {
        return r#"{"error":"no_valid_entries"}"#.to_string();
    }

    ura_log(2, &format!("/hall: {} valid entries", entries.len()));
    format!(
        r#"{{"count":{},"entries":[{}]}}"#,
        entries.len(),
        entries.join(",")
    )
}
```

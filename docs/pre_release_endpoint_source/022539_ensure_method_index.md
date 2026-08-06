# `ensure_method_index`

source_commit: `ffc3748df2d3c8c57b34aa3fdd64f75d09ed0866`
source_line: `22539`

```rust
unsafe fn ensure_method_index() -> Result<(), String> {
    {
        let mut state = METHOD_INDEX.lock().map_err(|_| "method_index_lock_poisoned".to_string())?;
        match state.status {
            "ready" => return Ok(()),
            "building" => return Err("method_index_building".to_string()),
            "failed" => return Err(state.error.clone()),
            _ => state.status = "building",
        }
    }
    let result = build_method_index();
    let mut state = METHOD_INDEX.lock().map_err(|_| "method_index_lock_poisoned".to_string())?;
    match result {
        Ok(entries) => {
            let class_count = {
                let image = get_image();
                let pointer = resolve_il2cpp_symbol("il2cpp_image_get_class_count");
                if image.is_null() || pointer.is_null() { 0 } else {
                    let function: FnImageGetClassCount = std::mem::transmute(pointer);
                    function(image)
                }
            };
            let null_count = entries.iter().filter(|entry| entry.method_pointer == 0).count();
            let mut duplicate_count = 0usize;
            let mut previous = 0usize;
            for entry in entries.iter().filter(|entry| entry.method_pointer != 0) {
                if entry.method_pointer == previous { duplicate_count += 1; }
                previous = entry.method_pointer;
            }
            state.image_class_count = class_count;
            state.indexed_class_count = class_count;
            state.indexed_method_count = entries.len();
            state.null_method_pointer_count = null_count;
            state.duplicate_method_pointer_count = duplicate_count;
            state.entries = entries;
            state.error.clear();
            state.status = "ready";
            Ok(())
        }
        Err(error) => {
            state.status = "failed";
            state.error = error.clone();
            Err(error)
        }
    }
}
```

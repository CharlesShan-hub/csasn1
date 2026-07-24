use std::path::PathBuf;
use std::fs;
use std::io::Write;
use std::collections::HashMap;

/// Rust primitive integer types that a newtype may wrap.
const INT_TYPES: &[&str] = &["u8", "i8", "u16", "i16", "u32", "i32", "u64", "i64"];

fn main() {
    println!("cargo:rerun-if-changed=specs/dlt2811.asn");

    // 第 1 步：运行 rasn-compiler，生成 Rust 类型
    rasn_compiler::Compiler::<
        rasn_compiler::prelude::RasnBackend,
        rasn_compiler::CompilerMissingParams,
    >::new()
        .set_output_mode(rasn_compiler::OutputMode::SingleFile(
            PathBuf::from("src/generated.rs"),
        ))
        .add_asn_by_path(PathBuf::from("specs/dlt2811.asn"))
        .compile()
        .expect("ASN.1 编译失败");

    // 第 2 步：扫描生成的代码，提取所有结构体类型名以及 newtype 的内层类型
    let generated = fs::read_to_string("src/generated.rs")
        .expect("无法读取 generated.rs");

    let mut types: Vec<String> = Vec::new();
    // type_name → inner_type for newtype structs (e.g. "Int32U" → "u32")
    let mut newtype_inner: HashMap<String, String> = HashMap::new();
    let mut pos = 0;
    let bytes = generated.as_bytes();

    while pos < bytes.len() {
        let struct_keyword = b"pub struct ";
        let enum_keyword = b"pub enum ";
        let mut found_pos: Option<usize> = None;

        if let Some(p) = find_subsequence(&bytes[pos..], struct_keyword) {
            found_pos = Some(pos + p);
        }
        if let Some(p) = find_subsequence(&bytes[pos..], enum_keyword) {
            let abs_p = pos + p;
            found_pos = match found_pos {
                Some(existing) => Some(existing.min(abs_p)),
                None => Some(abs_p),
            };
        }

        match found_pos {
            Some(p) => {
                pos = p;
                if bytes[pos..].starts_with(struct_keyword) {
                    pos += struct_keyword.len();
                } else {
                    pos += enum_keyword.len();
                }
                // 读取类型名
                let start = pos;
                while pos < bytes.len() && bytes[pos] != b' ' && bytes[pos] != b'{' && bytes[pos] != b'(' {
                    pos += 1;
                }
                let name = String::from_utf8_lossy(&bytes[start..pos]).to_string();

                if name.starts_with('_') {
                    pos = skip_paren_block(&bytes, pos);
                    continue;
                }

                // 检测 newtype: "pub struct Name (pub InnerType)"
                // 生成的代码中可能有空格: "pub struct Name(pub u32)" 或 "pub struct Name (pub u32)"
                let is_newtype = {
                    // 跳过名字后的空白字符
                    let mut scan = pos;
                    while scan < bytes.len() && bytes[scan] == b' ' { scan += 1; }
                    if scan < bytes.len() && bytes[scan] == b'(' {
                        // 提取内层类型 "(pub InnerType)"
                        let mut inner_pos = scan + 1;
                        while inner_pos < bytes.len() && bytes[inner_pos] != b'p' && bytes[inner_pos] != b')' { inner_pos += 1; }
                        if inner_pos + 3 < bytes.len() && &bytes[inner_pos..inner_pos+3] == b"pub" {
                            inner_pos += 3;
                            // skip spaces
                            while inner_pos < bytes.len() && bytes[inner_pos] == b' ' { inner_pos += 1; }
                            // 读取内层类型名
                            let inner_type_start = inner_pos;
                            while inner_pos < bytes.len() && bytes[inner_pos] != b')' && bytes[inner_pos] != b' ' {
                                inner_pos += 1;
                            }
                            let inner_type = String::from_utf8_lossy(&bytes[inner_type_start..inner_pos]).to_string();
                            if INT_TYPES.contains(&inner_type.as_str()) {
                                newtype_inner.insert(name.clone(), inner_type);
                            }
                        }
                        true
                    } else {
                        false
                    }
                };

                if !is_newtype {
                    // regular struct/enum — only add if not already present
                    if !types.contains(&name) {
                        types.push(name);
                    }
                } else {
                    // newtype — only add if not already present
                    if !types.contains(&name) {
                        types.push(name);
                    }
                }
                pos = skip_paren_block(&bytes, pos);
            }
            None => break,
        }
    }

    // 去重
    types.sort();
    types.dedup();

    // 第 3 步：生成 FFI dispatch 代码
    generate_ffi_dispatch(&types, &newtype_inner, "src/ffi_auto.rs");
}

fn generate_ffi_dispatch(types: &[String], newtype_inner: &HashMap<String, String>, output_path: &str) {
    let mut code = String::new();
    code.push_str(&format!(
        "// 自动生成 - 勿手动编辑\n\
         // 由 build.rs 扫描 generated.rs 自动生成\n\
         // 共 {} 个类型\n\n\
         #![allow(non_camel_case_types, non_snake_case, unused)]\n\n\
         use std::ffi::{{c_char, CStr, CString}};\n\
         use std::borrow::Cow;\n\n\
         #[path = \"generated.rs\"]\n\
         mod generated;\n\
         use generated::dlt2811_data_types::*;\n\n\
         /* ---- Jackson JSON ↔ JER adapter ---- */\n\
         /// Strips Jackson's `_choice` helper field, then unwraps `{{\"value\": X}}` if scalar.\n\
         fn unwrap_jackson_value<'a>(json: &'a str) -> Cow<'a, str> {{\n\
             if let Ok(serde_json::Value::Object(mut map)) = serde_json::from_str(json.trim()) {{\n\
                 map.remove(\"_choice\");\n\
                 if map.len() == 1 {{\n\
                     // Only unwrap `{{\"value\": X}}` — CHOICE variants keep their object wrapper\n\
                     if let Some(value) = map.get(\"value\") {{\n\
                         if !value.is_object() {{\n\
                             return Cow::Owned(serde_json::to_string(value).unwrap_or_default());\n\
                         }}\n\
                     }}\n\
                     return Cow::Owned(serde_json::to_string(&map).unwrap_or_default());\n\
                 }}\n\
             }}\n\
             Cow::Borrowed(json)\n\
         }}\n\n\
         /// If json is a bare value (not an object), wrap in {{\"value\": ...}}.\n\
         fn wrap_in_jackson(json: &str) -> String {{\n\
             let t = json.trim();\n\
             if t.starts_with('{{') {{\n\
                 t.to_string()\n\
             }} else {{\n\
                 format!(\"{{{{\\\"value\\\": {{}}}}}}\", t)\n\
             }}\n\
         }}\n\n",
        types.len()
    ));

    // Helper: generate the encode-encoding dispatch (BER/DER/APER/UPER)
    fn encode_dispatch(t: &str) -> String {
        format!(
            "match enc.as_str() {{\n\
             \"ber\" | \"\" | \"per\" => rasn::ber::encode(&v)\n\
             .map_err(|e| format!(\"BER encode {{type_name}}: {{e:?}}\")),\n\
             \"der\" => rasn::der::encode(&v)\n\
             .map_err(|e| format!(\"DER encode {{type_name}}: {{e:?}}\")),\n\
             \"aper\" => rasn::aper::encode(&v)\n\
             .map_err(|e| format!(\"APER encode {{type_name}}: {{e:?}}\")),\n\
             \"uper\" => rasn::uper::encode(&v)\n\
             .map_err(|e| format!(\"UPER encode {{type_name}}: {{e:?}}\")),\n\
             _ => Err(format!(\"Unsupported encoding: {{enc}}\")),\n\
             }}"
        )
    }

    // encode_json dispatch
    code.push_str(
        "fn encode_json(type_name: &str, encoding: &str, json: &str) -> Result<Vec<u8>, String> {\n\
         let enc = encoding.to_lowercase();\n\
         match type_name {\n"
    );
    for t in types {
        // For integer newtypes, use serde_json directly (bypass rasn JER decode bug for u32 values > i32::MAX)
        if let Some(inner) = newtype_inner.get(t) {
            code.push_str(&format!(
                "        \"{t}\" => {{\n\
                 let unwrapped = unwrap_jackson_value(json);\n\
                 let v: {t} = {{\n\
                 let n: {inner} = serde_json::from_str(&unwrapped)\n\
                 .map_err(|e| format!(\"Failed to parse {{type_name}} JSON: {{e}}\"))?;\n\
                 {t}(n)\n\
                 }};\n\
                 {encode}\n\
                 }}\n",
                encode = encode_dispatch(t)
            ));
        } else {
            code.push_str(&format!(
                "        \"{t}\" => {{\n\
                 let v: {t} = rasn::jer::decode(&unwrap_jackson_value(json))\n\
                 .map_err(|e| format!(\"JER decode {{type_name}}: {{e:?}}\"))?;\n\
                 {encode}\n\
                 }}\n",
                encode = encode_dispatch(t)
            ));
        }
    }
    code.push_str(
        "        _ => Err(format!(\"Unknown type: {}\", type_name))\n\
         }\n}\n\n"
    );

    // decode_to_json dispatch
    code.push_str(
        "fn decode_to_json(type_name: &str, encoding: &str, data: &[u8]) -> Result<String, String> {\n\
         let enc = encoding.to_lowercase();\n\
         match type_name {\n"
    );
    for t in types {
        code.push_str(&format!(
            "        \"{t}\" => {{\n\
             let v: {t} = match enc.as_str() {{\n\
             \"ber\" | \"\" | \"per\" => rasn::ber::decode(data)\n\
             .map_err(|e| format!(\"BER decode {{type_name}}: {{e:?}}\"))?,\n\
             \"der\" => rasn::der::decode(data)\n\
             .map_err(|e| format!(\"DER decode {{type_name}}: {{e:?}}\"))?,\n\
             \"aper\" => rasn::aper::decode(data)\n\
             .map_err(|e| format!(\"APER decode {{type_name}}: {{e:?}}\"))?,\n\
             \"uper\" => rasn::uper::decode(data)\n\
             .map_err(|e| format!(\"UPER decode {{type_name}}: {{e:?}}\"))?,\n\
             _ => return Err(format!(\"Unsupported encoding: {{enc}}\")),\n\
             }};\n\
             let jer_bytes = rasn::jer::encode(&v)\n\
             .map_err(|e| format!(\"JER encode {{type_name}}: {{e:?}}\"))?;\n\
             Ok(wrap_in_jackson(&jer_bytes))\n\
             }}\n"
        ));
    }
    code.push_str(
        "        _ => Err(format!(\"Unknown type: {}\", type_name))\n\
         }\n}\n\n"
    );

    // -- C API: all functions return *mut c_char (JSON response string) --
    code.push_str(
        "#[unsafe(no_mangle)]\npub extern \"C\" fn csasn1_encode(\n\
         type_name: *const c_char,\n\
         encoding: *const c_char,\n\
         json: *const c_char,\n\
         ) -> *mut c_char {\n\
         let result = (|| -> Result<String, String> {\n\
         let type_name = unsafe { CStr::from_ptr(type_name) }.to_str().map_err(|e| format!(\"{e}\"))?;\n\
         let encoding = unsafe { CStr::from_ptr(encoding) }.to_str().map_err(|e| format!(\"{e}\"))?;\n\
         let json = unsafe { CStr::from_ptr(json) }.to_str().map_err(|e| format!(\"{e}\"))?;\n\
         let bytes = encode_json(type_name, encoding, json)?;\n\
         Ok(serde_json::json!({\"ok\": true, \"bytes\": bytes}).to_string())\n\
         })().unwrap_or_else(|e| serde_json::json!({\"ok\": false, \"error\": e}).to_string());\n\
         CString::new(result).unwrap().into_raw()\n\
         }\n\n"
    );

    code.push_str(
        "#[unsafe(no_mangle)]\npub extern \"C\" fn csasn1_decode(\n\
         type_name: *const c_char,\n\
         encoding: *const c_char,\n\
         data: *const u8,\n\
         len: usize,\n\
         ) -> *mut c_char {\n\
         let result = (|| -> Result<String, String> {\n\
         let type_name = unsafe { CStr::from_ptr(type_name) }.to_str().map_err(|e| format!(\"{e}\"))?;\n\
         let encoding = unsafe { CStr::from_ptr(encoding) }.to_str().map_err(|e| format!(\"{e}\"))?;\n\
         let slice = unsafe { std::slice::from_raw_parts(data, len) };\n\
         let value_json: serde_json::Value = serde_json::from_str(&decode_to_json(type_name, encoding, slice)?).unwrap_or(serde_json::Value::Null);\n\
         Ok(serde_json::json!({\"ok\": true, \"value\": value_json}).to_string())\n\
         })().unwrap_or_else(|e| serde_json::json!({\"ok\": false, \"error\": e}).to_string());\n\
         CString::new(result).unwrap().into_raw()\n\
         }\n\n"
    );

    // csasn1_free_string
    code.push_str(
        "#[unsafe(no_mangle)]\npub extern \"C\" fn csasn1_free_string(s: *mut c_char) {\n\
         if !s.is_null() {\n\
         unsafe { drop(CString::from_raw(s)); }\n\
         }\n\
         }\n\n\
         #[unsafe(no_mangle)]\npub extern \"C\" fn csasn1_ping() -> *mut c_char {\n\
         CString::new(\"pong\").unwrap().into_raw()\n\
         }\n\n"
    );

    let mut file = fs::File::create(output_path).expect("无法创建 ffi_auto.rs");
    file.write_all(code.as_bytes()).expect("写入失败");
    println!("cargo:info=生成了 {} 个类型的 FFI dispatch", types.len());

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-cdylib-link-arg=/DEF:asn1.def");
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len())
        .position(|window| window == needle)
}

fn skip_paren_block(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && bytes[pos] == b' ' { pos += 1; }
    if pos >= bytes.len() { return pos; }

    let (open, close) = match bytes[pos] {
        b'{' => (b'{', b'}'),
        b'(' => (b'(', b')'),
        _ => return pos + 1,
    };
    pos += 1;
    let mut depth = 1;
    while pos < bytes.len() && depth > 0 {
        match bytes[pos] {
            c if c == open => depth += 1,
            c if c == close => depth -= 1,
            _ => {}
        }
        pos += 1;
    }
    pos
}

use std::collections::HashMap;

// ── ASN.1 spec text parsers ────────────────────────────────────

/// Extract ASN.1 type definitions from a spec file.
/// Returns a map of type_name → definition text (including the `::=` line).
/// For anonymous inline types (e.g. `AnonymousGetAllCBValuesResponsePDUCbValueValue`),
/// falls back to the parent ASN.1 type definition via substring matching.
pub fn extract_asn1_definitions(spec_path: &str, type_names: &[&str]) -> HashMap<String, String> {
    let src = match std::fs::read_to_string(spec_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    let lines: Vec<&str> = src.lines().collect();

    let mut all_defs: HashMap<String, (usize, String)> = HashMap::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(eq_pos) = trimmed.find("::=") {
            let before_eq = trimmed[..eq_pos].trim();
            if before_eq.is_empty() || before_eq.contains(' ') {
                continue;
            }
            let after_eq = trimmed[eq_pos + 3..].trim();
            let mut def_text = String::new();
            def_text.push_str(trimmed);
            def_text.push('\n');

            let name_dashless = before_eq.replace('-', "");

            if after_eq.contains('{') {
                let mut depth: i32 = 1;
                for j in i + 1..lines.len() {
                    let l = lines[j];
                    let code = l.split("--").next().unwrap_or("");
                    let mut stop = false;
                    for ch in code.chars() {
                        if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                            if depth == 0 {
                                stop = true;
                                break;
                            }
                        }
                    }
                    def_text.push_str(l);
                    def_text.push('\n');
                    if stop {
                        break;
                    }
                }
            }

            all_defs.insert(name_dashless, (i, def_text.trim().to_string()));
        }
    }

    let mut sorted_names: Vec<&str> = all_defs.keys().map(|s| s.as_str()).collect();
    sorted_names.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut defs = HashMap::new();
    for tn in type_names {
        if let Some(def) = all_defs.get(*tn) {
            defs.insert(tn.to_string(), def.1.clone());
            continue;
        }
        let tn_dashless = tn.replace('-', "");
        if let Some(def) = all_defs.get(&tn_dashless) {
            defs.insert(tn.to_string(), def.1.clone());
            continue;
        }
        if tn.starts_with("Anonymous") {
            for parent in &sorted_names {
                if tn.contains(parent) {
                    defs.insert(
                        tn.to_string(),
                        format!(
                            "(inline type within {})",
                            all_defs.get(*parent).unwrap().1.lines().next().unwrap_or("")
                        ),
                    );
                    break;
                }
            }
        }
    }
    defs
}

/// Convert dash-separated name to UPPER_SNAKE_CASE (e.g. "data-change" → "DATA_CHANGE")
pub fn constant_name(s: &str) -> String {
    s.to_uppercase().replace('-', "_")
}

/// Extract named constants from BIT STRING / ENUMERATED definitions.
/// Returns type_name -> [(Java constant name, value)].
pub fn extract_asn1_named_constants(spec_path: &str) -> HashMap<String, Vec<(String, i32)>> {
    let src = match std::fs::read_to_string(spec_path) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };
    let lines: Vec<&str> = src.lines().collect();
    let mut result = HashMap::new();

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        if let Some(before_brace) = trimmed.split('{').next() {
            let before_eq = before_brace.split("::=").next().unwrap_or("").trim();
            let after_eq = before_brace.split("::=").nth(1).unwrap_or("").trim();
            if before_eq.is_empty()
                || !after_eq.contains("BIT STRING") && !after_eq.contains("ENUMERATED")
            {
                i += 1;
                continue;
            }
            let type_name = before_eq;
            let mut constants = Vec::new();
            i += 1;
            while i < lines.len() {
                let line = lines[i].split("--").next().unwrap_or("").trim();
                if line.contains('}') {
                    break;
                }
                if let Some(paren_start) = line.find('(') {
                    let name = line[..paren_start].trim();
                    let after_paren = &line[paren_start + 1..];
                    if let Some(paren_end) = after_paren.find(')') {
                        if let Ok(val) = after_paren[..paren_end].trim().parse::<i32>() {
                            if !name.is_empty() {
                                constants.push((constant_name(name), val));
                            }
                        }
                    }
                }
                i += 1;
            }
            if !constants.is_empty() {
                result.insert(type_name.to_string(), constants);
            }
        }
        i += 1;
    }
    result
}
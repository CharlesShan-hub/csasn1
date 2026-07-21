use super::super::*;
use super::type_map::resolve_java_type;
use super::helpers;
use super::helpers::safe_field_name;

pub fn generate(_ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, variants: &[VariantInfo]) -> String {
    let mut c = String::new();
    let test_variants: Vec<_> = variants.iter().take(2).collect();

    for v in &test_variants {
        let fname = safe_field_name(&v.name);
        c.push_str(&helpers::ln(1, "@Test"));
        c.push_str(&helpers::ln(1, &format!("public void testChoice{}() throws Exception {{", v.name)));
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, &format!("obj._choice = \"{}\";", v.name)));
        let jt = resolve_java_type(&v.inner_type, all, prefix);
        match jt.as_str() {
            "int" => c.push_str(&helpers::ln(2, &format!("obj.{} = 42;", fname))),
            "long" => c.push_str(&helpers::ln(2, &format!("obj.{} = 42L;", fname))),
            "boolean" => c.push_str(&helpers::ln(2, &format!("obj.{} = true;", fname))),
            "float" => c.push_str(&helpers::ln(2, &format!("obj.{} = 1.5f;", fname))),
            "double" => c.push_str(&helpers::ln(2, &format!("obj.{} = 2.5;", fname))),
            "String" => c.push_str(&helpers::ln(2, &format!("obj.{} = \"test\";", fname))),
            "byte[]" => c.push_str(&helpers::ln(2, &format!("obj.{} = new byte[0];", fname))),
            s if s.starts_with("java.util.List<") => {}
            _ => c.push_str(&helpers::ln(2, &format!("obj.{} = new {}();", fname, jt))),
        }
        c.push_str(&helpers::ln(2, "String json = MAPPER.writeValueAsString(obj);"));
        c.push_str(&helpers::ln(2, &format!("{} d = MAPPER.readValue(json, {}.class);", cn, cn)));
        c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
        c.push_str(&helpers::ln(1, "}"));
        c.push('\n');
    }

    // testEncodeDecode (first variant)
    if let Some(v) = variants.first() {
        let fname = safe_field_name(&v.name);
        c.push_str(&helpers::ln(1, "@Test"));
        c.push_str(&helpers::ln(1, "public void testEncodeDecode() throws Exception {"));
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, &format!("obj._choice = \"{}\";", v.name)));
        let jt = resolve_java_type(&v.inner_type, all, prefix);
        match jt.as_str() {
            "int" => c.push_str(&helpers::ln(2, &format!("obj.{} = 42;", fname))),
            "long" => c.push_str(&helpers::ln(2, &format!("obj.{} = 42L;", fname))),
            "boolean" => c.push_str(&helpers::ln(2, &format!("obj.{} = true;", fname))),
            "float" => c.push_str(&helpers::ln(2, &format!("obj.{} = 1.5f;", fname))),
            "double" => c.push_str(&helpers::ln(2, &format!("obj.{} = 2.5;", fname))),
            "String" => c.push_str(&helpers::ln(2, &format!("obj.{} = \"test\";", fname))),
            _ => c.push_str(&helpers::ln(2, &format!("obj.{} = new {}();", fname, jt))),
        }
        c.push_str(&helpers::ln(2, "byte[] data = obj.encode(\"uper\");"));
        c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(\"uper\", data);", cn, cn)));
        c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
        c.push_str(&helpers::ln(1, "}"));
    }
    c
}

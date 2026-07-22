use super::super::*;
use super::type_map::resolve_wrapper_type;
use super::helpers;
use super::helpers::safe_field_name;

pub fn generate(_ti: &TypeInfo, all: &[TypeInfo], prefix: &str, cn: &str, variants: &[VariantInfo]) -> String {
    let mut c = String::new();
    let test_variants: Vec<_> = variants.iter().take(2).collect();

    for v in &test_variants {
        let fname = safe_field_name(&v.name);
        let json_key = v.identifier.as_deref().unwrap_or(&v.name);
        c.push_str(&helpers::ln(1, "@Test"));
        c.push_str(&helpers::ln(1, &format!("public void testEncodeDecodeAper{}() throws Exception {{", v.name)));
        c.push_str(&helpers::ln(2, &format!("{} obj = new {}();", cn, cn)));
        c.push_str(&helpers::ln(2, &format!("obj._choice = \"{}\";", json_key)));
        let jt = resolve_wrapper_type(&v.inner_type, all, prefix);
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
        c.push_str(&helpers::ln(2, "byte[] data = obj.encode(\"aper\");"));
        c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(\"aper\", data);", cn, cn)));
        c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
        c.push_str(&helpers::ln(1, "}"));
        c.push('\n');
    }
    c
}

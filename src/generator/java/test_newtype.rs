use super::helpers;

/// Newtype roundtrip test: sample() → encodeTest → decode → assert equality.
pub fn generate(cn: &str) -> String {
    let mut c = String::new();

    c.push_str(&helpers::ln(1, "@Test"));
    c.push_str(&helpers::ln(
        1,
        "public void testEncodeDecodeAper() throws Exception {",
    ));
    c.push_str(&helpers::ln(2, &format!("{} obj = {}.sample();", cn, cn)));
    c.push_str(&helpers::ln(2, "byte[] data = obj.encodeTest();"));
    c.push_str(&helpers::ln(2, &format!("{} d = {}.decode(data);", cn, cn)));
    c.push_str(&helpers::ln(2, "assertEquals(obj, d);"));
    c.push_str(&helpers::ln(1, "}"));
    c
}

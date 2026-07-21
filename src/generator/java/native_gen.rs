/// Generate CmsNative.java — JNA bridge to the Rust asn1.dll.
pub fn gen_native(prefix: &str, package: &str) -> String {
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    format!(
        r#"// Auto-generated. JNA bridge to native ASN.1 codec (csasn1 Rust lib).
// Requires: com.sun.jna (net.java.dev.jna:jna:5.14.0)
// All native functions return JSON strings; binary data is base64-encoded.

{pkg}import com.sun.jna.*;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.util.Map;

public class {pfx}Native {{
    private static final NativeImpl LIB = Native.load("asn1", NativeImpl.class);
    private static final ObjectMapper MAPPER = new ObjectMapper();

    private interface NativeImpl extends Library {{
        Pointer csasn1_ping();
        Pointer csasn1_encode(String typeName, String encoding, String json);
        Pointer csasn1_decode(String typeName, String encoding, byte[] data, int len);
        void csasn1_free_string(Pointer s);
    }}

    /** Quick check that native library loads and responds. */
    public static String ping() {{
        Pointer p = LIB.csasn1_ping();
        try {{
            return p.getString(0, "UTF-8");
        }} finally {{
            LIB.csasn1_free_string(p);
        }}
    }}

    /** Read a null-terminated C string from a Pointer and free it. */
    private static String readAndFree(Pointer p) {{
        try {{
            return p.getString(0, "UTF-8");
        }} finally {{
            LIB.csasn1_free_string(p);
        }}
    }}

    /**
     * Encode JSON to ASN.1 binary.
     * @param typeName  ASN.1 type name
     * @param encoding  encoding rule ("ber", "der", "aper", "uper")
     * @param json      JSON representation of the data
     * @return encoded binary data
     */
    public static byte[] encode(String typeName, String encoding, String json) {{
        try {{
            String resp = readAndFree(LIB.csasn1_encode(typeName, encoding, json));
            @SuppressWarnings("unchecked")
            Map<String, Object> m = MAPPER.readValue(resp, Map.class);
            if (Boolean.TRUE.equals(m.get("ok"))) {{
                return MAPPER.convertValue(m.get("bytes"), byte[].class);
            }}
            throw new RuntimeException("encode failed: " + m.get("error"));
        }} catch (Exception e) {{
            throw new RuntimeException(e);
        }}
    }}

    /**
     * Decode ASN.1 binary to JSON.
     * @param typeName  ASN.1 type name
     * @param encoding  encoding rule
     * @param data      binary encoded data
     * @return JSON representation
     */
    public static String decode(String typeName, String encoding, byte[] data) {{
        try {{
            String resp = readAndFree(LIB.csasn1_decode(typeName, encoding, data, data.length));
            @SuppressWarnings("unchecked")
            Map<String, Object> m = MAPPER.readValue(resp, Map.class);
            if (Boolean.TRUE.equals(m.get("ok"))) {{
                return (String) m.get("value");
            }}
            throw new RuntimeException("decode failed: " + m.get("error"));
        }} catch (Exception e) {{
            throw new RuntimeException(e);
        }}
    }}
}}"#,
        pkg = pkg,
        pfx = prefix,
    )
}

/// Generate CmsBase.java — abstract base class for all data types.
pub fn gen_base(prefix: &str, package: &str, default_enc: &str) -> String {
    let pkg = if package.is_empty() {
        String::new()
    } else {
        format!("package {};\n\n", package)
    };
    format!(
        r#"// Auto-generated. Base class for all {pfx} data types.

{pkg}import com.fasterxml.jackson.databind.ObjectMapper;

public abstract class {pfx}Base {{
    public static final String DEFAULT_ENCODING = "{enc}";

    @Override
    public String toString() {{
        try {{
            return new ObjectMapper().writeValueAsString(this);
        }} catch (Exception e) {{
            return getClass().getSimpleName() + "{{...}}";
        }}
    }}
}}
"#,
        pkg = pkg,
        pfx = prefix,
        enc = default_enc,
    )
}

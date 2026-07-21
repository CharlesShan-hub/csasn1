// Auto-generated. JNI bridge to native ASN.1 codec.

public class AsnNative {
    static {
        System.loadLibrary("asn1");
    }

    /**
     * Encode JSON to ASN.1 binary.
     * @param typeName  ASN.1 type name (e.g. "Apdu", "Boolean")
     * @param encoding  encoding rule ("ber", "der", "aper", "uper")
     * @param json      JSON representation of the data
     * @return encoded binary data
     */
    public static native byte[] encode(String typeName, String encoding, String json);

    /**
     * Decode ASN.1 binary to JSON.
     * @param typeName  ASN.1 type name
     * @param encoding  encoding rule
     * @param data      binary encoded data
     * @return JSON representation
     */
    public static native String decode(String typeName, String encoding, byte[] data);
}

# Build release binary + generate Java classes + tests (Linux/macOS)
# The csasn1 binary automatically copies libasn1.so to assets/java/src/main/resources/
build-assets:
    rm -rf assets/java
    cargo build --release
    cargo run --release -- --src specs/dlt2811.asn --dest assets/java --prefix Cms --enc aper --package com.ysh.jcms.data

# Build + generate + run Java tests
test-java: build-assets
    cd assets/java && mvn test

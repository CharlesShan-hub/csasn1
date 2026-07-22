# Build release binary + DLL, generate Java classes + tests, copy DLL/SO to assets/
build-assets:
    powershell -NoProfile -Command "Remove-Item -Recurse -Force 'assets/java' -ErrorAction SilentlyContinue; exit 0"
    cargo build --release
    cargo run --release -- --src specs/dlt2811.asn --dest assets/java --prefix Cms --enc aper --package com.example.csasn1
    powershell -NoProfile -Command "Copy-Item -Path target/release/asn1.dll -Destination assets/java/ -Force"
    powershell -NoProfile -Command "if (Test-Path target/release/libasn1.so) { Copy-Item -Path target/release/libasn1.so -Destination assets/java/ -Force }"

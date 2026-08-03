# ─── Platform paths ──────────────────────────────────────
# csasn1 与 jcms 同级（均在 cms 目录下），从 csasn1 出发 .. 即 cms 层
# 可用环境变量 JCMS_DIR 覆盖，直接指向 cms 层（jcms 的上级）
jcms_dir := env_var_or_default("JCMS_DIR", "..")
jcms_root := jcms_dir

# ─── Build ───────────────────────────────────────────────
# Compile Rust binary + library
build:
    cargo build --release

# ─── Java ────────────────────────────────────────────────
# Generate Java classes from ASN.1 spec (standalone test project)
gen-java:
    rm -rf assets/java
    cargo run --release -- --src specs/dlt2811.asn --dest assets/java --prefix Inner --enc aper --package com.ysh.jcms.data

# Build + generate + run Java standalone tests
test-java: gen-java
    cd assets/java && mvn test

# Generate Java classes directly into the jcms-data Maven module
gen-jcms-data:
    rm -rf {{jcms_root}}/jcms/jcms-data
    cargo run --release -- --src specs/dlt2811.asn --dest {{jcms_root}}/jcms/jcms-data --prefix Inner --enc aper --package com.ysh.jcms.data

# Run a single Java test by class name (e.g. just test-java-one CmsObjectNameTest)
test-java-one cls:
    cd assets/java && mvn test -Dtest={{cls}}

# ─── Python ──────────────────────────────────────────────
# Generate Python package from ASN.1 spec
gen-python:
    rm -rf assets/python
    cargo run --release -- --lang python --src specs/dlt2811.asn --dest assets/python --prefix Inner --enc aper --package com.ysh.jcms.data

# Generate + run Python tests (requires pixi installed)
test-python: gen-python
    cd assets/python && pixi run test

# ─── Debug / Explore ─────────────────────────────────────
# List all ASN.1 type names from the generated Rust code
rust-all:
    @powershell -NoProfile -File scripts/list_types.ps1

# Show JER JSON ("上帝格式") via example binary (any type, needs JSON input)
# 用法: just jer <typename> <json>  如 just jer RcbOptFlds 6800
jer type json:
    cargo run --example jer_god -- {{type}} '{{json}}' 2>&1

# Show JER JSON via pre-defined test (only types with a test_*_jer_god_format test)
# 用法: just json <typename>  如 just json urcb
json type:
    cargo test test_{{type}}_jer_god_format -- --nocapture 2>&1

# Run ALL JER god-format tests
json-all:
    cargo test jer_god_format -- --nocapture 2>&1

# ─── Build All ───────────────────────────────────────────
# Build Rust + generate both Java and Python
gen-all: build gen-java gen-python

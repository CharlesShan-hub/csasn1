# ─── Platform paths ──────────────────────────────────────
# dlt2811bean project root — override via env var JCMS_DIR or edit below
#   Windows default: d:\project\work\standard\dlt2811bean
#   Linux default: /home/user/projects/dlt2811bean
jcms_dir := if os() == "windows" { "d:/project/work/standard/dlt2811bean" } else { "/home/user/projects/dlt2811bean" }

# ─── Build ───────────────────────────────────────────────
# Compile Rust binary + library
build:
    cargo build --release

# ─── Java ────────────────────────────────────────────────
# Generate Java classes from ASN.1 spec (standalone test project)
gen-java:
    cargo run --release -- --src specs/dlt2811.asn --dest assets/java --prefix Cms --enc aper --package com.ysh.jcms.data

# Build + generate + run Java standalone tests
test-java: gen-java
    cd assets/java && mvn test

# Generate Java classes directly into the jcms-data Maven module
gen-jcms-data:
    rm -rf {{jcms_dir}}/cms/jcms/jcms-data
    cargo run --release -- --src specs/dlt2811.asn --dest {{jcms_dir}}/cms/jcms/jcms-data --prefix Inner --enc aper --package com.ysh.jcms.data

# Run a single Java test by class name (e.g. just test-java-one CmsObjectNameTest)
test-java-one cls:
    cd assets/java && mvn test -Dtest={{cls}}

# ─── Python ──────────────────────────────────────────────
# Generate Python package from ASN.1 spec
gen-python:
    cargo run --release -- --lang python --src specs/dlt2811.asn --dest assets/python --prefix Cms --enc aper --package com.ysh.jcms.data

# Generate + run Python tests (requires pixi installed)
test-python: gen-python
    cd assets/python && pixi run test

# ─── Build All ───────────────────────────────────────────
# Build Rust + generate both Java and Python
gen-all: build gen-java gen-python

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use super::*;

pub mod type_map;
pub mod native_gen;
pub mod helpers;
mod class_gen;
mod test_gen;
pub(crate) mod gen_newtype;
pub(crate) mod gen_struct;
pub(crate) mod gen_choice;
pub(crate) mod test_newtype;
pub(crate) mod test_struct;
pub(crate) mod test_choice;

/// Default Java class name prefix
const DEFAULT_PREFIX: &str = "Asn";

pub struct JavaConfig {
    pub prefix: String,
    pub default_enc: String,
    pub package: String,
    pub out_dir: PathBuf,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            prefix: DEFAULT_PREFIX.to_string(),
            default_enc: "ber".to_string(),
            package: String::new(),
            out_dir: PathBuf::from("java/src"),
        }
    }
}

/// Entry point: generate Java classes from parsed types
pub fn generate(
    types: &[TypeInfo],
    cfg: &JavaConfig,
    asn_defs: &HashMap<String, String>,
    named_consts: &HashMap<String, Vec<(String, i32)>>,
) {
    // —— Java source files ——————————————————————————
    fs::create_dir_all(&cfg.out_dir).expect("failed to create output directory");
    for t in types {
        let code = class_gen::gen_class(t, types, &cfg.prefix, &cfg.default_enc, &cfg.package, asn_defs, named_consts);
        fs::write(
            cfg.out_dir.join(format!("{}{}.java", cfg.prefix, t.name)),
            &code,
        )
        .unwrap();

        let test_code = test_gen::gen_test_class(t, types, &cfg.prefix, &cfg.package, asn_defs);
        let test_dir = helpers::derive_test_dir(&cfg.out_dir);
        fs::create_dir_all(&test_dir).expect("failed to create test output directory");
        fs::write(
            test_dir.join(format!("{}{}Test.java", cfg.prefix, t.name)),
            &test_code,
        )
        .unwrap();
    }
    fs::write(
        cfg.out_dir.join(format!("{}Native.java", cfg.prefix)),
        &native_gen::gen_native(&cfg.prefix, &cfg.package),
    )
    .unwrap();
    fs::write(
        cfg.out_dir.join(format!("{}Base.java", cfg.prefix)),
        &native_gen::gen_base(&cfg.prefix, &cfg.package, &cfg.default_enc),
    )
    .unwrap();

    // —— Maven project structure ———————————————————
    let project_root = helpers::derive_project_root(&cfg.out_dir);
    let res_dir = project_root.join("src").join("main").join("resources");
    fs::create_dir_all(&res_dir).expect("failed to create resources dir");

    let pom_path = project_root.join("pom.xml");
    if !pom_path.exists() {
        fs::write(&pom_path, &gen_pom(&cfg.prefix, &cfg.package))
            .expect("failed to write pom.xml");
        println!("  wrote pom.xml");
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let dll_name = if cfg!(target_os = "windows") { "asn1.dll" } else { "libasn1.so" };
            let dll_src = exe_dir.join(dll_name);
            if dll_src.exists() {
                fs::copy(&dll_src, res_dir.join(dll_name))
                    .expect("failed to copy asn1.dll to resources");
                println!("  copied {} to resources/", dll_name);
            }
        }
    }

    println!(
        "✓ generated {} Java classes (incl. {}Native.java, {}Base.java) in {:?}",
        types.len(), cfg.prefix, cfg.prefix, cfg.out_dir
    );
    println!(
        "✓ generated {} Java test classes in {:?}",
        types.len(),
        helpers::derive_test_dir(&cfg.out_dir)
    );
}

/// Generate a standalone Maven pom.xml for the auto-generated data types.
fn gen_pom(prefix: &str, package: &str) -> String {
    let group_id = if let Some(dot) = package.rfind('.') {
        package[..dot].to_string()
    } else {
        "com.example".to_string()
    };
    let artifact_id = format!("{}-data", prefix.to_lowercase());
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>

    <groupId>{group_id}</groupId>
    <artifactId>{artifact_id}</artifactId>
    <version>1.0.0-SNAPSHOT</version>
    <packaging>jar</packaging>

    <name>{artifact_id} — Auto-generated ASN.1 POJO data types</name>

    <properties>
        <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>
        <maven.compiler.source>8</maven.compiler.source>
        <maven.compiler.target>8</maven.compiler.target>
        <junit.version>4.13.2</junit.version>
        <lombok.version>1.18.36</lombok.version>
        <jna.version>5.14.0</jna.version>
    </properties>

    <dependencies>
        <dependency>
            <groupId>com.fasterxml.jackson.core</groupId>
            <artifactId>jackson-databind</artifactId>
            <version>2.17.0</version>
        </dependency>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>${{junit.version}}</version>
            <scope>test</scope>
        </dependency>
        <dependency>
            <groupId>org.projectlombok</groupId>
            <artifactId>lombok</artifactId>
            <version>${{lombok.version}}</version>
            <scope>provided</scope>
        </dependency>
        <dependency>
            <groupId>net.java.dev.jna</groupId>
            <artifactId>jna</artifactId>
            <version>${{jna.version}}</version>
        </dependency>
    </dependencies>

    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-compiler-plugin</artifactId>
                <configuration>
                    <source>8</source>
                    <target>8</target>
                </configuration>
            </plugin>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-surefire-plugin</artifactId>
                <configuration>
                    <argLine>-Djava.library.path=${{project.basedir}}/src/main/resources</argLine>
                </configuration>
            </plugin>
        </plugins>
    </build>
</project>
"#,
        group_id = group_id,
        artifact_id = artifact_id,
    )
}

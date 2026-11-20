use std::process::Command;

use anyhow::{Context, ensure};

/// Uses JBang and a JDK to deserialize JSON into a Java object and return its
/// Java Object Serialization bytes.
///
/// `target_class` is the fully qualified class name. `classes` lists the `.java`
/// or `.class` files to load. Relative paths are resolved from the crate root.
/// `json` is the inline JSON payload.
///
/// # Errors
///
/// Returns an error if JBang cannot be started, `Ser.java` exits unsuccessfully,
/// or its standard output cannot be decoded as hexadecimal bytes.
pub(crate) fn serialize(
    target_class: &str,
    classes: &[&str],
    json: &str,
) -> anyhow::Result<Vec<u8>> {
    let mut command = Command::new("jbang");
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("Ser.java");
    for class in classes {
        command.args(["-c", class]);
    }

    let output = command
        .args(["-n", target_class, "-j", json])
        .output()
        .context("failed to run jbang Ser.java")?;
    ensure!(
        output.status.success(),
        "jbang Ser.java failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    );

    let hex = std::str::from_utf8(&output.stdout).context("jbang output is not UTF-8")?;
    hex::decode(hex.trim()).context("invalid serialization hex from jbang Ser.java")
}

#[cfg(test)]
mod tests {
    use super::serialize;

    #[test]
    #[ignore = "requires JBang and a JDK"]
    fn test_serialize_demo() -> anyhow::Result<()> {
        let actual = serialize(
            "com.example.Demo",
            &["testdata/Demo.java"],
            r#"{"i":123,"message":"hello"}"#,
        )?;
        let expected = hex::decode(concat!(
            "aced000573720010636f6d2e6578616d706c652e44656d6f",
            "477d87c81fbf509e020002490001694c00076d657373616765",
            "7400124c6a6176612f6c616e672f537472696e673b78700000007b74000568656c6c6f",
        ))?;
        assert_eq!(actual, expected);
        Ok(())
    }
}

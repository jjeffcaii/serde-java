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
    fn test_serialize_self_reference() -> anyhow::Result<()> {
        let actual = serialize("java.util.HashMap", &[], r#"{"self":{"$ref":"$"}}"#)?;
        // Generated with ObjectOutputStream after inserting the map into itself.
        let expected = hex::decode(concat!(
            "aced0005737200116a6176612e7574696c2e486173684d6170",
            "0507dac1c31660d103000246000a6c6f6164466163746f724900097468726573686f6c64",
            "78703f4000000000000c7708000000100000000174000473656c6671007e000178",
        ))?;
        assert_eq!(actual, expected);
        Ok(())
    }

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

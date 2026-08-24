#!/usr/bin/env bash
# Verifies that #[derive(JavaSerialize)] rejects unsupported inputs with a
# readable message. Compile-failure cases cannot be plain #[test]s, so each case
# is compiled in a throwaway test file and the error output is grepped.
#
# Run from the repo root:  bash derive/verify-compile-errors.sh
set -uo pipefail

PROBE="tests/__compile_fail_probe.rs"
FAILED=0

cleanup() { rm -f "$PROBE"; }
trap cleanup EXIT

check() {
    local label="$1" expect="$2" body="$3"
    printf 'use serde_java::*;\n%s\n' "$body" > "$PROBE"

    local out
    out=$(cargo build --test __compile_fail_probe 2>&1)

    if [ $? -eq 0 ]; then
        echo "FAIL  $label -- compiled successfully but should have been rejected"
        FAILED=1
    elif ! grep -qF "$expect" <<< "$out"; then
        echo "FAIL  $label -- expected message not found: $expect"
        echo "$out" | grep -E '^error' | head -5 | sed 's/^/        /'
        FAILED=1
    else
        echo "ok    $label"
    fi
    rm -f "$PROBE"
}

check "missing class" \
    'missing `#[java(class = "...")]`' '
#[derive(JavaSerialize)]
#[java(serial_version_uid = 1)]
struct A { x: i32 }
'

check "missing serial_version_uid" \
    'missing `#[java(serial_version_uid = ...)]`' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A")]
struct A { x: i32 }
'

check "unknown container attribute" \
    'unknown `java` container attribute' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1, bogus = 2)]
struct A { x: i32 }
'

check "unknown field attribute" \
    'unknown `java` field attribute' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { #[java(bogus)] x: i32 }
'

check "rust char" \
    "Rust \`char\` is 4 bytes" '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: char }
'

check "u32 has no java equivalent" \
    'has no Java equivalent' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: u32 }
'

check "Vec<String>" \
    'are not supported' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: Vec<String> }
'

check "Vec<i8>" \
    'use `u8` instead' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: Vec<i8> }
'

check "Vec<Vec<u8>>" \
    'are not supported' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: Vec<Vec<u8>> }
'

check "Option<i32>" \
    'Java primitives cannot be null' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: Option<i32> }
'

check "Option<Option<String>>" \
    'nested `Option`' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: Option<Option<String>> }
'

check "tuple struct" \
    'requires a struct with named fields' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A(i32);
'

check "enum" \
    'cannot be derived for an enum' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
enum A { X }
'

check "generic struct" \
    'cannot depend on type parameters' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A<T> { x: T }
'

check "primitive signature override" \
    'must start with `L` or `[`' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { #[java(signature = "I")] x: i32 }
'

check "with without signature" \
    'requires an explicit `#[java(signature = "...")]`' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { #[java(with = "MyLayout")] x: Vec<String> }
'

check "with a malformed path" \
    'expected identifier' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { #[java(signature = "Ljava/util/List;", with = "1not::a::path")] x: Vec<String> }
'

check "duplicate java field name" \
    'duplicate Java field name' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { a: i32, #[java(rename = "a")] b: i32 }
'

check "duplicate java field name straddling primitive/object boundary" \
    'duplicate Java field name' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 1)]
struct A { x: i32, a: String, #[java(rename = "x")] y: String }
'

check "suid out of range" \
    'does not fit in a Java `long`' '
#[derive(JavaSerialize)]
#[java(class = "com.example.A", serial_version_uid = 99999999999999999999)]
struct A { x: i32 }
'

if [ "$FAILED" -ne 0 ]; then
    echo
    echo "some compile-error cases did not behave as expected"
    exit 1
fi
echo
echo "all compile-error cases behave as expected"

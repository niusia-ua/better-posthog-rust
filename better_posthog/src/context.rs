use std::sync::LazyLock;

/// Cached OS information to avoid repeated system calls.
struct OsInfo {
  name: String,
  version: String,
  arch: String,
}

static OS_INFO: LazyLock<OsInfo> = LazyLock::new(|| {
  let info = os_info::get();
  OsInfo {
    name: info.os_type().to_string(),
    version: info.version().to_string(),
    arch: std::env::consts::ARCH.to_string(),
  }
});

/// Library version parsed from Cargo.toml.
static LIB_VERSION: LazyLock<semver::Version> = LazyLock::new(|| {
  semver::Version::parse(env!("CARGO_PKG_VERSION")).expect("CARGO_PKG_VERSION should be valid semver")
});

/// Saturates the event with library and OS context metadata.
///
/// This adds the following properties to the event:
/// - `$lib`: The crate name
/// - `$lib_version`: Full version string
/// - `$lib_version_major`: Major version number
/// - `$lib_version_minor`: Minor version number
/// - `$lib_version_patch`: Patch version number
/// - `$os`: Operating system name
/// - `$os_version`: Operating system version
/// - `$os_arch`: System architecture
pub fn saturate_event(event: &mut crate::Event) {
  let props = &mut event.properties;

  // Library metadata.
  let version = &*LIB_VERSION;
  props
    .entry("$lib".to_string())
    .or_insert_with(|| serde_json::Value::String(env!("CARGO_PKG_NAME").to_string()));
  props
    .entry("$lib_version".to_string())
    .or_insert_with(|| serde_json::Value::String(version.to_string()));
  props
    .entry("$lib_version_major".to_string())
    .or_insert_with(|| serde_json::Value::Number(version.major.into()));
  props
    .entry("$lib_version_minor".to_string())
    .or_insert_with(|| serde_json::Value::Number(version.minor.into()));
  props
    .entry("$lib_version_patch".to_string())
    .or_insert_with(|| serde_json::Value::Number(version.patch.into()));

  // OS metadata.
  let os_info = &*OS_INFO;
  props
    .entry("$os".to_string())
    .or_insert_with(|| serde_json::Value::String(os_info.name.clone()));
  props
    .entry("$os_version".to_string())
    .or_insert_with(|| serde_json::Value::String(os_info.version.clone()));
  props
    .entry("$os_arch".to_string())
    .or_insert_with(|| serde_json::Value::String(os_info.arch.clone()));
}

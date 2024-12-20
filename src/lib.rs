use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub enum PackageKind {
    Module,
    Provider,
}

fn return_package_type(kind: PackageKind) -> String {
    match kind {
        PackageKind::Module => String::from("modules"),
        PackageKind::Provider => String::from("providers"),
    }
}

pub trait StorageBackend {
    fn check_package_available(
        &self,
        package_kind: PackageKind,
        namespace: &str,
        package_name: &str,
    ) -> Result<bool, std::io::Error>;
    // This must likely live here, any implementation may require a different URL, but maybe not. TBD
    fn return_package_link(
        &self,
        package_kind: PackageKind,
        namespace: &str,
        package_name: &str,
    ) -> Result<String, std::io::Error>;
    fn fetch_package(
        &self,
        package_kind: PackageKind,
        namespace: &str,
        package_name: &str,
    ) -> Result<String, std::io::Error>;
}

pub struct LocalStorageBackend {
    storage: String,
}

impl LocalStorageBackend {
    fn new() -> Self {
        Self {
            storage: String::new(),
        }
    }
}

impl StorageBackend for LocalStorageBackend {
    fn check_package_available(
        &self,
        _package_kind: PackageKind,
        _namespace: &str,
        _package_name: &str,
    ) -> Result<bool, std::io::Error> {
        todo!()
    }
    fn return_package_link(
        &self,
        _package_kind: PackageKind,
        _namespace: &str,
        _package_name: &str,
    ) -> Result<String, std::io::Error> {
        todo!()
    }
    fn fetch_package(
        &self,
        _package_kind: PackageKind,
        _namespace: &str,
        _package_name: &str,
    ) -> Result<String, std::io::Error> {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
pub struct RegistryVersion {
    version: String,
    platforms: Vec<PlatformArchPair>,
}

#[derive(Deserialize, Debug)]
pub struct PlatformArchPair {
    os: String,
    arch: String,
}

#[derive(Deserialize, Debug)]
pub struct RegistryVersionsList {
    versions: Vec<RegistryVersion>,
}

#[derive(Serialize)]
pub struct MirrorVersionsList {
    versions: HashMap<String, MirrorVersion>,
}

#[derive(Serialize)]
pub struct MirrorVersion {}

fn transform_version_list(registry_versions: RegistryVersionsList) -> MirrorVersionsList {
    MirrorVersionsList {
        versions: registry_versions
            .versions
            .into_iter()
            .map(|v| (v.version.clone(), MirrorVersion {}))
            .collect::<HashMap<String, MirrorVersion>>(),
    }
}

pub trait ProviderMirror {
    // FIXME: return type should not be limited only to reqwest::Error, but can be any error
    async fn list_versions(
        &mut self,
        hostname: &str,
        namespace: &str,
        kind: PackageKind,
        package_name: &str,
    ) -> Result<MirrorVersionsList, reqwest::Error>;

    async fn list_installation_packages(
        &mut self,
        hostname: &str,
        namespace: &str,
        kind: PackageKind,
        package_name: &str,
        version: &str,
    ) -> Result<AvailablePackages, reqwest::Error>;
}

pub struct RealProviderRegistry {}

impl ProviderMirror for RealProviderRegistry {
    // FIXME (Andriy): (how?) this is basically a DDoS generator
    // FIXME (Mattia): cache maybe
    async fn list_versions(
        &mut self,
        hostname: &str,
        namespace: &str,
        kind: PackageKind,
        package_name: &str,
    ) -> Result<MirrorVersionsList, reqwest::Error> {
        Ok(reqwest::get(format!(
            "{}",
            format_args!(
                "https://{}/v1/{}/{}/{}/versions",
                hostname,
                return_package_type(kind),
                namespace,
                package_name
            )
        ))
        .await?
        .json::<RegistryVersionsList>()
        .await
        .map(transform_version_list)?)
    }

    async fn list_installation_packages(
        &mut self,
        hostname: &str,
        namespace: &str,
        kind: PackageKind,
        package_name: &str,
        version: &str,
    ) -> Result<AvailablePackages, reqwest::Error> {
        Ok(reqwest::get(format!(
            "{}",
            format_args!(
                "https://{}/v1/{}/{}/{}/versions",
                hostname,
                return_package_type(kind),
                namespace,
                package_name
            )
        ))
        .await?
        .json::<RegistryVersionsList>()
        .await
        .map(|rvl| generate_installation_packages(rvl, version))?)
    }
}

#[derive(Serialize)]
pub struct AvailablePackages {
    archives: HashMap<String, Archive>,
}

#[derive(Serialize)]
pub struct Archive {
    url: String,
}

fn generate_installation_packages(rvl: RegistryVersionsList, version: &str) -> AvailablePackages {
    let matching_version = rvl
        .versions
        .into_iter()
        .find(|v| v.version == version)
        .unwrap();

    let mut archives = HashMap::new();

    for pair in matching_version.platforms {
        let key = format!("{}_{}", pair.os, pair.arch);
        let value = Archive {
            url: format!("{}_{}_{}.zip", version, pair.os, pair.arch),
        };
        archives.insert(key, value);
    }

    return AvailablePackages { archives };
}

#[derive(Serialize)]
pub enum ListOrDownloadResponse {
    AvailablePackages(AvailablePackages),
    DownloadLinkRedirect,
}

impl From<AvailablePackages> for ListOrDownloadResponse {
    fn from(value: AvailablePackages) -> Self {
        Self::AvailablePackages(value)
    }
}

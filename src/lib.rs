use std::collections::HashMap;

use axum::response::Redirect;
use serde::{Deserialize, Serialize};

mod storage_backend;
pub use storage_backend::{LocalStorageBackend, StorageBackend};

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

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ProviderPackageVersion {
    num: String,
    os: String,
    arch: String,
}

impl ProviderPackageVersion {
    pub fn new(num: String, os: String, arch: String) -> Self {
        Self { num, os, arch }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ProviderPackage {
    pub hostname: String,
    pub namespace: String,
    pub name: String,
    version: Option<ProviderPackageVersion>,
}

impl ProviderPackage {
    pub fn new(hostname: &str, namespace: &str, name: &str) -> Self {
        Self {
            hostname: hostname.to_string(),
            namespace: namespace.to_string(),
            name: name.to_string(),
            version: None,
        }
    }

    pub fn with_version(
        hostname: &str,
        namespace: &str,
        name: &str,
        version: ProviderPackageVersion,
    ) -> Self {
        Self {
            hostname: hostname.to_string(),
            namespace: namespace.to_string(),
            name: name.to_string(),
            version: Some(version),
        }
    }

    pub fn arch(&self) -> String {
        self.version.clone().unwrap().arch.to_string()
    }

    pub fn os(&self) -> String {
        self.version.clone().unwrap().os.to_string()
    }

    pub fn version(&self) -> String {
        self.version.clone().unwrap().num.to_string()
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
        &self,
        package: &ProviderPackage,
    ) -> Result<MirrorVersionsList, reqwest::Error>;
    async fn list_installation_packages(
        &self,
        package: &ProviderPackage,
        version: &str,
    ) -> Result<AvailablePackages, reqwest::Error>;
}

#[derive(Clone)]
pub struct RealProviderRegistry {}

impl ProviderMirror for RealProviderRegistry {
    // FIXME (Andriy): (how?) this is basically a DDoS generator
    // FIXME (Mattia): cache maybe
    async fn list_versions(
        &self,
        package: &ProviderPackage,
    ) -> Result<MirrorVersionsList, reqwest::Error> {
        Ok(reqwest::get(format!(
            "{}",
            format_args!(
                "https://{}/v1/providers/{}/{}/versions",
                package.hostname, package.namespace, package.name
            )
        ))
        .await?
        .json::<RegistryVersionsList>()
        .await
        .map(transform_version_list)?)
    }

    async fn list_installation_packages(
        &self,
        package: &ProviderPackage,
        version: &str,
    ) -> Result<AvailablePackages, reqwest::Error> {
        Ok(reqwest::get(format!(
            "{}",
            format_args!(
                "https://{}/v1/providers/{}/{}/versions",
                package.hostname, package.namespace, package.name,
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

// TODO: this can be functionalized more.
// TODO: this assumes we always find a matching version. ideally, this should return Option<> instead.
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
            // NOTE: this url is what later be used/passed into the download handler again so we MUST have (at leasst) the version part here!
            // and everything else can be reconstructed back from the full url path: registry, namespace, package_name.
            url: format!("{}/download/{}/{}", version, pair.os, pair.arch),
        };
        archives.insert(key, value);
    }

    return AvailablePackages { archives };
}

#[derive(Deserialize)]
pub struct DownloadMetadata {
    download_url: String,
}

pub async fn redirect_to_real_download(
    package: &ProviderPackage,
) -> Result<Redirect, reqwest::Error> {
    Ok(reqwest::get(format!(
        "{}",
        format_args!(
            "https://{}/v1/providers/{}/{}/{}/download/{}/{}",
            package.hostname,
            package.namespace,
            package.name,
            package.version(),
            package.os(),
            package.arch()
        )
    ))
    .await?
    .json::<DownloadMetadata>()
    .await
    .map(construct_redirect)?)
}

fn construct_redirect(metadata: DownloadMetadata) -> Redirect {
    Redirect::temporary(&metadata.download_url)
}

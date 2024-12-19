use std::collections::{HashMap, HashSet};

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
    fn new() -> Self;
}

pub struct LocalStorageBackend;

impl StorageBackend for LocalStorageBackend {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Deserialize)]
pub struct RegistryVersion {
    version: String,
}

#[derive(Deserialize)]
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
            .collect::<HashMap<_, _>>(),
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
    ) -> ();
}

pub struct RealProviderRegistry {}

impl ProviderMirror for RealProviderRegistry {
    // FIXME: (how?) this is basically a DDoS generator
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
    ) -> () {
        todo!()
    }
}

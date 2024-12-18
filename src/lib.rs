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
    async fn check_package_versions(
        &self,
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
        )).await?.json::<RegistryVersionsList>().await.map(|res| transform_version_list(res))?)
    }
}

pub struct LocalStorageBackend;

impl StorageBackend for LocalStorageBackend {
    fn new() -> Self {
        Self {}
    }
}

#[derive(Deserialize)]
pub struct RegistryVersion {
    version: String
}

#[derive(Deserialize)]
pub struct RegistryVersionsList {
    versions: Vec<RegistryVersion>
}

#[derive(Serialize)]
// TODO: implement custome serialize function to turn null into empty objects
pub struct MirrorVersionsList {
    versions: HashSet<String>
}

fn transform_version_list(registry_versions: RegistryVersionsList) -> MirrorVersionsList {
    let mut result = MirrorVersionsList{
        versions: HashSet::new(),
    };

    for version in &registry_versions.versions {
        result.versions.insert(version.version.clone());
    }

    return result;
}

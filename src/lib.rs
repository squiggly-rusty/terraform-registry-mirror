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
        namespace: &str,
        kind: PackageKind,
        package_name: &str,
    ) -> Result<String, reqwest::Error> {
        Ok(reqwest::get(format!(
            "{}",
            format_args!(
                "https://registry.terraform.io/v1/{}/{}/{}/versions",
                return_package_type(kind),
                namespace,
                package_name
            )
        ))
        .await?
        .text()
        .await?)
    }
}

pub struct LocalStorageBackend;

impl StorageBackend for LocalStorageBackend {
    fn new() -> Self {
        Self {}
    }
}
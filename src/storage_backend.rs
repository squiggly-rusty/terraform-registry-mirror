use dashmap::DashMap;

use crate::ProviderPackage;

pub trait StorageBackend {
    fn check_package_available(&self, package: &ProviderPackage) -> bool;
    // This must likely live here, any implementation may require a different URL, but maybe not. TBD
    fn return_package_link(&self, package: &ProviderPackage) -> Option<String>;
    fn fetch_package(&self, package: &ProviderPackage) -> Result<String, std::io::Error>;
}

enum PackageStatus {
    Downloading,
    Ready(String),
}

pub struct LocalStorageBackend {
    packages_status: DashMap<ProviderPackage, PackageStatus>,
}

impl LocalStorageBackend {
    pub fn new() -> Self {
        Self {
            packages_status: DashMap::new(),
        }
    }
}

impl StorageBackend for LocalStorageBackend {
    fn check_package_available(&self, package: &ProviderPackage) -> bool {
        self.packages_status
            .get(&package)
            .filter(|status| matches!(**status, PackageStatus::Ready { .. }))
            .is_some()
    }
    fn return_package_link(&self, package: &ProviderPackage) -> Option<String> {
        if self.check_package_available(package) {
            Some("".to_string())
        } else {
            self.fetch_package(package);
            None
        }
    }
    fn fetch_package(&self, package: &ProviderPackage) {
        // self.packages_status
        //     .insert(package.clone(), PackageStatus::Downloading);
        // TODO: do scary async stuff here
        tokio::spawn(async {});
    }
}

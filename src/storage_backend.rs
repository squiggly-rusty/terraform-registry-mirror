use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use tracing::info;

use crate::ProviderPackage;

pub trait StorageBackend {
    fn check_package_available(&self, package: &ProviderPackage) -> bool;
    // This must likely live here, any implementation may require a different URL, but maybe not. TBD
    fn return_package_link(&self, package: &ProviderPackage) -> Option<String>;
    fn fetch_package(&self, package: &ProviderPackage);
}

enum PackageStatus {
    Downloading,
    Ready(String),
}

#[derive(Clone)]
pub struct LocalStorageBackend {
    packages_status: Arc<DashMap<ProviderPackage, PackageStatus>>,
}

impl LocalStorageBackend {
    pub fn new() -> Self {
        Self {
            packages_status: DashMap::new().into(),
        }
    }
}

impl StorageBackend for LocalStorageBackend {
    fn check_package_available(&self, package: &ProviderPackage) -> bool {
        self.packages_status
            .get(package)
            .filter(|status| matches!(**status, PackageStatus::Ready(_)))
            .is_some()
    }
    fn return_package_link(&self, package: &ProviderPackage) -> Option<String> {
        if self.check_package_available(package) {
            if let PackageStatus::Ready(uri) =  &(*self.packages_status.get(package).unwrap()) {
                Some(uri.clone())
            } else {
                None
            }
        } else {
            self.fetch_package(package);
            None
        }
    }
    fn fetch_package(&self, package: &ProviderPackage) {
        // NOTE: this is ugly, is there any way to avoid this?
        let r = self.packages_status.clone();
        let pc = package.clone();
        tokio::spawn(async move{
            r.insert(pc.clone(), PackageStatus::Downloading);
            info!("fetching package...");
            tokio::time::sleep(Duration::from_secs(5)).await;
            r.alter(&pc, |_,_| PackageStatus::Ready("".to_string()));
            info!("fetched package!");
        });
    }
}

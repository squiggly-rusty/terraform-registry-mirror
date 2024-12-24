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
            .try_get(package)
            .try_unwrap()
            .filter(|status| matches!(**status, PackageStatus::Ready(_)))
            .is_some()
    }
    fn return_package_link(&self, package: &ProviderPackage) -> Option<String> {
        if self.check_package_available(package) {
            // NOTE: someone can (potentially) modify the package between the two if statements
            if let PackageStatus::Ready(uri) = &(*self.packages_status.try_get(package).try_unwrap().unwrap()) {
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
        let pc = self.packages_status.clone();
        let p = package.clone();

        // NOTE: spawnine a new thread just to check the status seems, wasteful, but is there any way to do it otherwise? Rust borrow checker complains that data must have 'static lifetime. :shrug:
        tokio::spawn(async move {
            // try to get an entry, if it's locked by someone, give up
            match pc.try_entry(p) {
                // we got exclusive access to the entry, do some stuff with it
                Some(entry) => match entry {
                    // there's no value yet, do some useful work here
                    dashmap::Entry::Vacant(vacant) => {
                        let mut r = vacant.insert(PackageStatus::Downloading);
                        info!("fetching package...");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        *r = PackageStatus::Ready("".to_string());
                        info!("fetched package!");
                    }
                    // somebody already wrote to it, do nothing
                    dashmap::Entry::Occupied(_) => (),
                },
                // entry is currently locked, nothing to do here
                None => (),
            };
        });
    }
}

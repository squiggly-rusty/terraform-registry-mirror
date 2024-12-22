use crate::ProviderPackage;

pub trait StorageBackend {
    fn check_package_available(&self, package: ProviderPackage) -> bool;
    // This must likely live here, any implementation may require a different URL, but maybe not. TBD
    fn return_package_link(&self, package: ProviderPackage) -> Option<String>;
    fn fetch_package(&self, package: ProviderPackage) -> Result<String, std::io::Error>;
}

#[derive(Clone)]
pub struct LocalStorageBackend {
    storage: String,
}

impl LocalStorageBackend {
    pub fn new() -> Self {
        Self {
            storage: String::new(),
        }
    }
}

impl StorageBackend for LocalStorageBackend {
    fn check_package_available(&self, _package: ProviderPackage) -> bool {
        false
    }
    fn return_package_link(&self, package: ProviderPackage) -> Option<String> {
        if self.check_package_available(package) {
            Some("".to_string())
        } else {
            // TODO: fire off the download here
            None
        }
    }
    fn fetch_package(&self, _package: ProviderPackage) -> Result<String, std::io::Error> {
        todo!()
    }
}

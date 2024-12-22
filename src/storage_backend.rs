pub trait StorageBackend {
    fn check_package_available(
        &self,
        namespace: &str,
        package_name: &str,
    ) -> Result<bool, std::io::Error>;
    // This must likely live here, any implementation may require a different URL, but maybe not. TBD
    fn return_package_link(
        &self,
        namespace: &str,
        package_name: &str,
    ) -> Result<String, std::io::Error>;
    fn fetch_package(&self, namespace: &str, package_name: &str) -> Result<String, std::io::Error>;
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
        _namespace: &str,
        _package_name: &str,
    ) -> Result<bool, std::io::Error> {
        todo!()
    }
    fn return_package_link(
        &self,
        _namespace: &str,
        _package_name: &str,
    ) -> Result<String, std::io::Error> {
        todo!()
    }
    fn fetch_package(
        &self,
        _namespace: &str,
        _package_name: &str,
    ) -> Result<String, std::io::Error> {
        todo!()
    }
}

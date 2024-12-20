
# Table of Contents

1.  [v0.0.1](#org51e8163)
2.  [v0.0.2](#org47e4b2a)



<a id="org51e8163"></a>

# v0.0.1

1.  Client makes a request to `list_available_versions`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-versions))
    1.  Make a request upstream and fetch missing information.
2.  Client makes a request to `list_available_installation_packages`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-installation-packages))
    1.  Make a request upstream and construct json with listing of available platform/os download links.
3.  Client makes a request to `download_package`. (this allows our mirror to track specifically what package client wants to download).
    1.  Server returns a 301 to the original download location upstream. TODO: this needs to be verified if it works at all!


<a id="org47e4b2a"></a>

# v0.0.2

1.  Client makes a request to `list_available_versions`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-versions))
    1.  Make a request upstream and fetch missing information.
2.  Client makes a request to `list_available_installation_packages`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-installation-packages))
    1.  Make a request upstream and construct json with listing of available platform/os download links.
3.  Client makes a request to `download_package`. (this allows our mirror to track specifically what package client wants to download).
    1.  Server returns a 301 to the original download location upstream. TODO: this needs to be verified!
    2.  Server queues up a download to a storage backend.
    3.  On next request, if the package is available in a storage backend, return a link to it, instead of upstream link.
        This has different meanings, depending on the storage backend - for the local one, it would mean to return the bytes (or start streaming them), for the s3 one, it would mean to return a pre-authorized download url.


# Table of Contents

1.  [v0.0.1 (done)](#org3322b45)
2.  [v0.0.2](#org3a60c0b)



<a id="org3322b45"></a>

# v0.0.1 (done)

1.  Client makes a request to `list_available_versions`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-versions))
    1.  Make a request upstream and fetch missing information.
2.  Client makes a request to `list_available_installation_packages`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-installation-packages))
    1.  Make a request upstream and construct json with listing of available platform/os download links.
3.  Client makes a request to `download_package`. (this allows our mirror to track specifically what package client wants to download).
    1.  Server returns a 307 (temporary redirect) to the original download location upstream.


<a id="org3a60c0b"></a>

# v0.0.2

1.  Client makes a request to `list_available_versions`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-versions))
    1.  Make a request upstream and fetch missing information.
2.  Client makes a request to `list_available_installation_packages`. ([upstream docs](https://developer.hashicorp.com/terraform/internals/provider-network-mirror-protocol#list-available-installation-packages))
    1.  Make a request upstream and construct json with listing of available platform/os download links.
3.  Client makes a request to `download_package`. (this allows our mirror to track specifically what package client wants to download).
    1.  Server returns a 307 (temporary redirect) to the original download location upstream.
    2.  Server queues up a download to a storage backend.
    3.  On next request, if the package is available in a storage backend, return a link to it, instead of upstream link.
        This has different meanings, depending on the storage backend - for the local one, it would mean to return the bytes (or start streaming them), for the s3 one, it would mean to return a pre-authorized download url.


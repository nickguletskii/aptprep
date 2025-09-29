"""Bazel rules for fetching SPDX license-list-data repository."""

def _spdx_license_repo_impl(repository_ctx):
    """Implementation of the spdx_license_repo repository rule."""
    version = "3.27.0"

    repository_ctx.download_and_extract(
        url = "https://github.com/spdx/license-list-data/archive/refs/tags/v{}.tar.gz".format(version),
        stripPrefix = "license-list-data-{}".format(version),
        sha256 = "7a1eade71449d2ff3ae42957452f6e3a660a3704b477d0e72afc2b43be94c907",
    )

    # Create a BUILD file to expose the license text files
    repository_ctx.file("BUILD.bazel", """
filegroup(
    name = "text_files",
    srcs = glob(["text/*.txt"]),
    visibility = ["//visibility:public"],
)

filegroup(
    name = "json_files",
    srcs = glob(["json/details/*.json"]),
    visibility = ["//visibility:public"],
)

filegroup(
    name = "all_files",
    srcs = glob(["**/*"]),
    visibility = ["//visibility:public"],
)
""")

spdx_license_repo = repository_rule(
    implementation = _spdx_license_repo_impl,
    doc = "Fetches the SPDX license-list-data repository",
)

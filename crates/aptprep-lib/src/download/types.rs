use debian_packaging::checksum::AnyContentDigest;

#[derive(Clone, Debug)]
pub struct DownloadItem {
    pub base_url: String,
    pub rel_path: String,
    pub size: Option<u64>,
    pub digest: AnyContentDigest,
    pub output_path: Option<String>, // Optional custom output path, relative to output_dir
}

#[derive(Clone, Copy, Debug)]
pub struct DownloadAndCheckOptions {
    pub max_concurrency_per_host: usize,
    pub max_retries: usize,
    pub download_parallelism: usize,
    pub checking_parallelism: usize,
}

impl Default for DownloadAndCheckOptions {
    fn default() -> Self {
        Self {
            max_concurrency_per_host: 8,
            max_retries: 5,
            download_parallelism: 16,
            checking_parallelism: 128,
        }
    }
}

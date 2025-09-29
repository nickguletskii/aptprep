use debian_packaging::io::ContentDigest;

#[derive(Clone, Debug)]
pub struct DownloadItem {
    pub base_url: String,
    pub rel_path: String,
    pub size: Option<u64>,
    pub digest: ContentDigest,
    pub output_path: Option<String>, // Optional custom output path, relative to output_dir
}

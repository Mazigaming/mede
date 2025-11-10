use std::path::PathBuf;
use yt_dlp::{Youtube, fetcher::deps::Libraries};

#[derive(Clone)]
pub struct Downloader {
    libraries: Libraries,
    output_dir: PathBuf,
}

impl Downloader {
    pub fn new(libraries_dir: PathBuf, output_dir: PathBuf) -> Self {
        let youtube = libraries_dir.join("yt-dlp");
        let ffmpeg = libraries_dir.join("ffmpeg");
        let libraries = Libraries::new(youtube, ffmpeg);
        Self { libraries, output_dir }
    }
    
    pub async fn download(
        &self, 
        url: String, 
        output_file: String
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let fetcher = Youtube::new(self.libraries.clone(), self.output_dir.clone())?;
        let video_path = fetcher.download_video_from_url(url, &output_file).await?;
        Ok(video_path)
    }
}

use std::path::PathBuf;
use std::fs;
use std::io::Write;

const LIBS_DIR: &str = "libs";
const OUTPUT_DIR: &str = "output";
const YT_DLP_SUBDIR: &str = "yt-dlp";
const FFMPEG_SUBDIR: &str = "ffmpeg";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    let libs_dir = PathBuf::from(LIBS_DIR);
    let yt_dlp_dir = libs_dir.join(YT_DLP_SUBDIR);
    let ffmpeg_dir = libs_dir.join(FFMPEG_SUBDIR);
    
    if let Err(e) = fs::create_dir_all(&yt_dlp_dir) {
        println!("cargo:warning=Failed to create yt-dlp directory: {}", e);
    }
    if let Err(e) = fs::create_dir_all(&ffmpeg_dir) {
        println!("cargo:warning=Failed to create ffmpeg directory: {}", e);
    }
    if let Err(e) = fs::create_dir_all(OUTPUT_DIR) {
        println!("cargo:warning=Failed to create output directory: {}", e);
    }
    
    let yt_dlp_path = yt_dlp_dir.join(if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" });
    if !yt_dlp_path.exists() {
        println!("cargo:warning=Downloading yt-dlp...");
        if let Err(e) = download_yt_dlp(&yt_dlp_path) {
            println!("cargo:warning=Failed to download yt-dlp: {}", e);
        }
    } else {
        println!("cargo:warning=yt-dlp already exists at {:?}", yt_dlp_path);
    }
    
    let ffmpeg_path = ffmpeg_dir.join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
    if !ffmpeg_path.exists() {
        println!("cargo:warning=Downloading ffmpeg...");
        if let Err(e) = download_ffmpeg(&ffmpeg_path) {
            println!("cargo:warning=Failed to download ffmpeg: {}", e);
            println!("cargo:warning=You may need to manually install ffmpeg");
        }
    } else {
        println!("cargo:warning=ffmpeg already exists at {:?}", ffmpeg_path);
    }
}

fn download_yt_dlp(dest_path: &PathBuf) -> Result<(), String> {
    let url = if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    };
    
    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("Network error: {}", e))?;
    
    let mut file = fs::File::create(dest_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    std::io::copy(&mut response.into_reader(), &mut file)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(dest_path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dest_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    
    println!("cargo:warning=Successfully downloaded yt-dlp");
    Ok(())
}

fn download_ffmpeg(dest_path: &PathBuf) -> Result<(), String> {
    if cfg!(target_os = "linux") {
        println!("cargo:warning=Downloading ffmpeg static build for Linux...");
        
        let url = "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz";
        let response = ureq::get(url)
            .call()
            .map_err(|e| format!("Network error: {}", e))?;
        
        let temp_dir = std::env::temp_dir();
        let tar_path = temp_dir.join("ffmpeg.tar.xz");
        
        let mut file = fs::File::create(&tar_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        std::io::copy(&mut response.into_reader(), &mut file)
            .map_err(|e| format!("Failed to write tarball: {}", e))?;
        
        let extract_dir = temp_dir.join("ffmpeg_extract");
        fs::create_dir_all(&extract_dir)
            .map_err(|e| format!("Failed to create extract dir: {}", e))?;
        
        let output = std::process::Command::new("tar")
            .args(&["-xJf", tar_path.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
            .output()
            .map_err(|e| format!("Failed to execute tar: {}", e))?;
        
        if !output.status.success() {
            return Err(format!("tar extraction failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        let entries = fs::read_dir(&extract_dir)
            .map_err(|e| format!("Failed to read extract dir: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            if path.is_dir() {
                let ffmpeg_bin = path.join("ffmpeg");
                if ffmpeg_bin.exists() {
                    fs::copy(&ffmpeg_bin, dest_path)
                        .map_err(|e| format!("Failed to copy ffmpeg: {}", e))?;
                    
                    use std::os::unix::fs::PermissionsExt;
                    let metadata = fs::metadata(dest_path)
                        .map_err(|e| format!("Failed to get metadata: {}", e))?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(dest_path, perms)
                        .map_err(|e| format!("Failed to set permissions: {}", e))?;
                    
                    println!("cargo:warning=Successfully installed ffmpeg");
                    return Ok(());
                }
            }
        }
        Err("Could not find ffmpeg binary in extracted archive".to_string())
        
    } else if cfg!(target_os = "macos") {
        println!("cargo:warning=For macOS, install ffmpeg via: brew install ffmpeg");
        println!("cargo:warning=Then create a symlink: ln -s $(which ffmpeg) libs/ffmpeg/ffmpeg");
        Err("ffmpeg not found on macOS - manual installation required".to_string())
        
    } else if cfg!(target_os = "windows") {
        println!("cargo:warning=Downloading ffmpeg for Windows...");
        
        let url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
        let response = ureq::get(url)
            .call()
            .map_err(|e| format!("Network error: {}", e))?;
        
        let temp_dir = std::env::temp_dir();
        let zip_path = temp_dir.join("ffmpeg.zip");
        
        let mut file = fs::File::create(&zip_path)
            .map_err(|e| format!("Failed to create temp file: {}", e))?;
        std::io::copy(&mut response.into_reader(), &mut file)
            .map_err(|e| format!("Failed to write zip: {}", e))?;
        
        let file = fs::File::open(&zip_path)
            .map_err(|e| format!("Failed to open zip: {}", e))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip: {}", e))?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;
            if file.name().ends_with("bin/ffmpeg.exe") {
                let mut outfile = fs::File::create(dest_path)
                    .map_err(|e| format!("Failed to create ffmpeg.exe: {}", e))?;
                std::io::copy(&mut file, &mut outfile)
                    .map_err(|e| format!("Failed to extract ffmpeg.exe: {}", e))?;
                println!("cargo:warning=Successfully installed ffmpeg");
                return Ok(());
            }
        }
        Err("Could not find ffmpeg.exe in zip archive".to_string())
    } else {
        Err("Unsupported platform".to_string())
    }
}

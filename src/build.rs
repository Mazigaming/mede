use std::path::PathBuf;
use std::fs;
use std::io::Write;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    let libs_dir = PathBuf::from("libs");
    let yt_dlp_dir = libs_dir.join("yt-dlp");
    let ffmpeg_dir = libs_dir.join("ffmpeg");
    
    fs::create_dir_all(&yt_dlp_dir).expect("Failed to create yt-dlp directory");
    fs::create_dir_all(&ffmpeg_dir).expect("Failed to create ffmpeg directory");
    fs::create_dir_all("output").expect("Failed to create output directory");
    
    let yt_dlp_path = yt_dlp_dir.join(if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" });
    if !yt_dlp_path.exists() {
        println!("cargo:warning=Downloading yt-dlp...");
        download_yt_dlp(&yt_dlp_path);
    } else {
        println!("cargo:warning=yt-dlp already exists at {:?}", yt_dlp_path);
    }
    
    let ffmpeg_path = ffmpeg_dir.join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });
    if !ffmpeg_path.exists() {
        println!("cargo:warning=Downloading ffmpeg...");
        download_ffmpeg(&ffmpeg_path);
    } else {
        println!("cargo:warning=ffmpeg already exists at {:?}", ffmpeg_path);
    }
}

fn download_yt_dlp(dest_path: &PathBuf) {
    let url = if cfg!(target_os = "windows") {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    } else {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp"
    };
    
    let response = ureq::get(url)
        .call()
        .expect("Failed to download yt-dlp");
    
    let mut file = fs::File::create(dest_path)
        .expect("Failed to create yt-dlp file");
    
    std::io::copy(&mut response.into_reader(), &mut file)
        .expect("Failed to write yt-dlp file");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(dest_path)
            .expect("Failed to get file metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(dest_path, perms)
            .expect("Failed to set executable permissions");
    }
    
    println!("cargo:warning=Successfully downloaded yt-dlp");
}

fn download_ffmpeg(dest_path: &PathBuf) {
    if cfg!(target_os = "linux") {
        println!("cargo:warning=Downloading ffmpeg static build for Linux...");
        
        let url = "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz";
        let response = ureq::get(url)
            .call()
            .expect("Failed to download ffmpeg");
        
        let temp_dir = std::env::temp_dir();
        let tar_path = temp_dir.join("ffmpeg.tar.xz");
        
        let mut file = fs::File::create(&tar_path)
            .expect("Failed to create temp file");
        std::io::copy(&mut response.into_reader(), &mut file)
            .expect("Failed to write ffmpeg tarball");
        
        let extract_dir = temp_dir.join("ffmpeg_extract");
        fs::create_dir_all(&extract_dir).expect("Failed to create extract dir");
        
        let output = std::process::Command::new("tar")
            .args(&["-xJf", tar_path.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
            .output()
            .expect("Failed to extract ffmpeg");
        
        if !output.status.success() {
            panic!("Failed to extract ffmpeg: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        let entries = fs::read_dir(&extract_dir).expect("Failed to read extract dir");
        for entry in entries {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.is_dir() {
                let ffmpeg_bin = path.join("ffmpeg");
                if ffmpeg_bin.exists() {
                    fs::copy(&ffmpeg_bin, dest_path)
                        .expect("Failed to copy ffmpeg binary");
                    
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(dest_path)
                        .expect("Failed to get file metadata")
                        .permissions();
                    perms.set_mode(0o755);
                    fs::set_permissions(dest_path, perms)
                        .expect("Failed to set executable permissions");
                    
                    println!("cargo:warning=Successfully installed ffmpeg");
                    return;
                }
            }
        }
        panic!("Could not find ffmpeg binary in extracted archive");
        
    } else if cfg!(target_os = "macos") {
        println!("cargo:warning=For macOS, please install ffmpeg via: brew install ffmpeg");
        println!("cargo:warning=Then create a symlink: ln -s $(which ffmpeg) libs/ffmpeg/ffmpeg");
        panic!("Please install ffmpeg manually on macOS");
        
    } else if cfg!(target_os = "windows") {
        println!("cargo:warning=Downloading ffmpeg for Windows...");
        
        let url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
        let response = ureq::get(url)
            .call()
            .expect("Failed to download ffmpeg");
        
        let temp_dir = std::env::temp_dir();
        let zip_path = temp_dir.join("ffmpeg.zip");
        
        let mut file = fs::File::create(&zip_path)
            .expect("Failed to create temp file");
        std::io::copy(&mut response.into_reader(), &mut file)
            .expect("Failed to write ffmpeg zip");
        
        let file = fs::File::open(&zip_path).expect("Failed to open zip");
        let mut archive = zip::ZipArchive::new(file).expect("Failed to read zip");
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).expect("Failed to read zip entry");
            if file.name().ends_with("bin/ffmpeg.exe") {
                let mut outfile = fs::File::create(dest_path)
                    .expect("Failed to create ffmpeg.exe");
                std::io::copy(&mut file, &mut outfile)
                    .expect("Failed to extract ffmpeg.exe");
                println!("cargo:warning=Successfully installed ffmpeg");
                return;
            }
        }
        panic!("Could not find ffmpeg.exe in zip archive");
    }
}

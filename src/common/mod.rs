use std::path::Path;
const VIDEO_EXTENSIONS: [&str; 4] = [".mkv", ".mp4", ".avi", ".m4v"];
const SUBTITLE_EXTENSIONS: [&str; 2] = [".srt", ".ass"];
pub async fn check_file_extensions(file_path: &Path) -> bool {
    let file_extension = file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    // todo 支持用户自定义配置
    VIDEO_EXTENSIONS.contains(&file_extension) || SUBTITLE_EXTENSIONS.contains(&file_extension)
}

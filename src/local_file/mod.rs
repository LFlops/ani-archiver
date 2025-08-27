use crate::local_file::regex_parser::extract_episode_info;
use std::env;
use std::fs::hard_link;
use std::path::{Path, PathBuf};
mod regex_parser;

use log::{info, warn};
#[cfg(test)]
use mockall::automock;
// 引入 nix crate 以获取文件系统元数据
#[cfg(target_family = "unix")]
use nix::sys::stat::stat;
use std::io;

#[cfg_attr(test, automock)]
pub(crate) trait StatPath {
    fn get_device_id(&self) -> io::Result<i32>;
}

impl StatPath for PathBuf {
    #[cfg(target_family = "unix")]
    fn get_device_id(&self) -> io::Result<i32> {
        let metadata = stat(self)?;
        Ok(metadata.st_dev)
    }

    // 辅助函数：在 Windows 上获取设备信息（这里仅作为占位符，需自行实现）
    // #[cfg(target_family = "windows")]
    // fn get_device_id(&self) -> io::Result<i32> {
    //     // 在 Windows 上，需要调用 GetVolumeInformationW 等 API 来获取卷序列号
    //     // 这个实现会比较复杂，这里简化为 None
    //     Ok(-1)
    // }
}

pub(crate) struct LocalFile<T: StatPath> {
    source_dir: T,
    dest_dir: T,
}
impl<T: StatPath> LocalFile<T> {
    fn new(source_dir: T, dest_dir: T) -> Self {
        LocalFile {
            source_dir,
            dest_dir,
        }
    }
    pub fn from_env() -> LocalFile<PathBuf> {
        let source_dir: PathBuf = PathBuf::from(env::var("SOURCE").expect("SOURCE not set"));
        let dest_dir: PathBuf = PathBuf::from(env::var("DEST").expect("DEST not set"));
        LocalFile {
            source_dir,
            dest_dir,
        }
    }
    // is_same_partition 现在直接使用 LocalFile 内部的泛型类型
    fn is_same_partition(&self) -> Result<bool, io::Error> {
        let source_id = self.source_dir.get_device_id()?;
        let dest_id = self.dest_dir.get_device_id()?;
        Ok(source_id == dest_id && source_id != -1 && dest_id != -1)
    }
}
impl<T: StatPath + AsRef<Path>> LocalFile<T> {
    pub async fn organize_files(&self, show_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let is_same_partition = self.is_same_partition()?;
        println!("  Organizing files...");
        for video_file in self.source_dir.as_ref().read_dir()? {
            let video_file = video_file?;
            match video_file.file_name().to_str() {
                Some(file_name) => {
                    if let Some((s_num, e_num)) = extract_episode_info(file_name) {
                        let new_file_name =
                            format!("{} S{}E{}.{:?}", show_name, s_num, e_num, file_name);
                        let dest_path = self.dest_dir.as_ref().join(&new_file_name);

                        if is_same_partition {
                            hard_link(video_file.path(), &dest_path)?;
                            info!("Created hard link: {}", dest_path.display())
                        } else {
                            println!("两个路径不在同一个分区。将创建软链接。");
                            // 创建软链接
                            #[cfg(target_family = "unix")]
                            {
                                use std::os::unix::fs::symlink;
                                symlink(&self.source_dir.as_ref(), &self.dest_dir.as_ref())?;
                                println!("软链接创建成功。");
                            }
                            // #[cfg(target_family = "windows")]
                            // {
                            //     // 在 Windows 上，需要判断是文件还是目录来创建不同的软链接
                            //     if path1.is_file() {
                            //         use std::os::windows::fs::symlink_file;
                            //         symlink_file(&path1, &path2)?;
                            //     } else {
                            //         use std::os::windows::fs::symlink_dir;
                            //         symlink_dir(&path1, &path2)?;
                            //     }
                            //     println!("软链接创建成功。");
                            // }
                        }
                    }
                }
                None => {
                    warn!("Skipping '{video_file:?}': Could not extract episode info.");
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_device_id_on_unix_like_system() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = PathBuf::from(temp_dir.path());
        let device_id = path.get_device_id().unwrap();
        assert!(device_id > 0)
    }
    #[test]
    fn test_local_file_new_from_env() {
        temp_env::with_vars(
            [("SOURCE", Some("test_source")), ("DEST", Some("test_dest"))],
            || {
                let local_file = LocalFile::<PathBuf>::from_env();

                assert_eq!(local_file.source_dir, PathBuf::from("test_source"));
                assert_eq!(local_file.dest_dir, PathBuf::from("test_dest"));
            },
        );
    }

    #[test]
    fn test_is_same_partition_return_true() {
        let mut mock_stat_path = MockStatPath::new();
        let mut mock_stat_path2 = MockStatPath::new();
        mock_stat_path.expect_get_device_id().returning(|| Ok(1));
        mock_stat_path2.expect_get_device_id().returning(|| Ok(1));
        let local_file = LocalFile::new(mock_stat_path, mock_stat_path2);
        match local_file.is_same_partition() {
            Ok(result) => {
                assert!(result);
            }
            Err(err) => {
                panic!("Error: {}", err);
            }
        }
    }
}

use std::ffi::OsString;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use tokio::fs;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;

#[derive(Debug, Default, Clone)]
pub enum FileSystem {
    #[default]
    Physical,
    Virtual,
}

impl FileSystem {
    pub async fn open(
        &self,
        path: &Path,
    ) -> io::Result<Box<dyn AsyncReadWrite>> {
        match self {
            FileSystem::Physical => Ok(Box::new(fs::File::open(&path).await?)),
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn canonicalize(
        &self,
        path: &Path,
    ) -> io::Result<PathBuf> {
        match self {
            FileSystem::Physical => fs::canonicalize(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn read_link(
        &self,
        path: &Path,
    ) -> io::Result<PathBuf> {
        match self {
            FileSystem::Physical => fs::read_link(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn copy(
        &self,
        from: &Path,
        to: &Path,
    ) -> io::Result<u64> {
        match self {
            FileSystem::Physical => fs::copy(from, to).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn create_dir(
        &self,
        path: &Path,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::create_dir(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn create_dir_all(
        &self,
        path: &Path,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::create_dir_all(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn hard_link(
        &self,
        original: &Path,
        link: &Path,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::hard_link(original, link).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn metadata(
        &self,
        path: &Path,
    ) -> io::Result<Metadata> {
        match self {
            FileSystem::Physical => Ok(Metadata::Physical(fs::metadata(path).await?)),
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn read(
        &self,
        path: &Path,
    ) -> io::Result<Vec<u8>> {
        match self {
            FileSystem::Physical => fs::read(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn try_exists(
        &self,
        path: &Path,
    ) -> io::Result<bool> {
        match self {
            FileSystem::Physical => fs::try_exists(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn read_dir(
        &self,
        path: &Path,
    ) -> io::Result<ReadDir> {
        match self {
            FileSystem::Physical => Ok(ReadDir::Physical(fs::read_dir(path).await?)),
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn read_to_string(
        &self,
        path: &Path,
    ) -> io::Result<String> {
        match self {
            FileSystem::Physical => fs::read_to_string(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub fn read_to_string_sync(
        &self,
        path: &Path,
    ) -> io::Result<String> {
        match self {
            FileSystem::Physical => std::fs::read_to_string(path),
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn remove_dir(
        &self,
        path: &Path,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::remove_dir(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn remove_dir_all(
        &self,
        path: &Path,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::remove_dir_all(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn remove_file(
        &self,
        path: &Path,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::remove_file(path).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn rename(
        &self,
        from: &Path,
        to: &Path,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::rename(from, to).await,
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn set_permissions(
        &self,
        path: &Path,
        perm: Permissions,
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;

                    let Permissions::Physical(perm) = perm else {
                        unreachable!()
                    };
                    fs::set_permissions(path, std::fs::Permissions::from_mode(perm.mode())).await
                }
                #[cfg(windows)]
                {
                    let mut x = fs::metadata(path).await?.permissions();
                    x.set_readonly(perm.readonly());
                    Ok(())
                }
            }
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn symlink_metadata(
        &self,
        path: &Path,
    ) -> io::Result<Metadata> {
        match self {
            FileSystem::Physical => Ok(Metadata::Physical(fs::symlink_metadata(path).await?)),
            FileSystem::Virtual => todo!(),
        }
    }

    pub async fn write(
        &self,
        path: &Path,
        contents: &[u8],
    ) -> io::Result<()> {
        match self {
            FileSystem::Physical => fs::write(path, contents).await,
            FileSystem::Virtual => todo!(),
        }
    }
}

pub trait AsyncReadWrite: AsyncRead + AsyncWrite {}

impl AsyncReadWrite for fs::File {}

pub enum File {
    Physical(Box<dyn AsyncReadWrite>),
    Virtual,
}

pub enum ReadDir {
    Physical(tokio::fs::ReadDir),
    Virtual,
}

impl ReadDir {
    pub async fn next_entry(&mut self) -> io::Result<Option<DirEntry>> {
        match self {
            ReadDir::Physical(read_dir) => {
                let Some(dir_entry) = read_dir.next_entry().await? else {
                    return Ok(None);
                };
                Ok(Some(DirEntry::Physical(dir_entry)))
            }
            ReadDir::Virtual => todo!(),
        }
    }
}

pub enum DirEntry {
    Physical(tokio::fs::DirEntry),
    Virtual,
}

impl DirEntry {
    pub fn path(&self) -> PathBuf {
        match self {
            DirEntry::Physical(dir_entry) => dir_entry.path(),
            DirEntry::Virtual => todo!(),
        }
    }

    pub fn file_name(&self) -> OsString {
        match self {
            DirEntry::Physical(dir_entry) => dir_entry.file_name(),
            DirEntry::Virtual => todo!(),
        }
    }

    pub async fn metadata(&self) -> io::Result<Metadata> {
        match self {
            DirEntry::Physical(_dir_entry) => todo!(),
            DirEntry::Virtual => todo!(),
        }
    }

    // pub async fn file_type(&self) -> io::Result<FileType> {
    //     match self {
    //         DirEntry::Physical(dir_entry) => todo!(),
    //     }
    // }
}

pub enum Metadata {
    Physical(std::fs::Metadata),
    Virtual,
}

impl Metadata {
    pub fn accessed(&self) -> std::io::Result<SystemTime> {
        match self {
            Metadata::Physical(metadata) => metadata.accessed(),
            Metadata::Virtual => todo!(),
        }
    }

    pub fn created(&self) -> std::io::Result<SystemTime> {
        match self {
            Metadata::Physical(metadata) => metadata.created(),
            Metadata::Virtual => todo!(),
        }
    }

    pub fn file_type(&self) -> std::fs::FileType {
        match self {
            Metadata::Physical(metadata) => metadata.file_type(),
            Metadata::Virtual => todo!(),
        }
    }

    pub fn is_dir(&self) -> bool {
        match self {
            Metadata::Physical(metadata) => metadata.is_dir(),
            Metadata::Virtual => todo!(),
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Metadata::Physical(metadata) => metadata.is_file(),
            Metadata::Virtual => todo!(),
        }
    }

    pub fn is_symlink(&self) -> bool {
        match self {
            Metadata::Physical(metadata) => metadata.is_symlink(),
            Metadata::Virtual => todo!(),
        }
    }

    pub fn modified(&self) -> std::io::Result<SystemTime> {
        match self {
            Metadata::Physical(metadata) => metadata.modified(),
            Metadata::Virtual => todo!(),
        }
    }

    pub fn permissions(&self) -> std::fs::Permissions {
        match self {
            Metadata::Physical(metadata) => metadata.permissions(),
            Metadata::Virtual => todo!(),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        match self {
            Metadata::Physical(metadata) => metadata.len(),
            Metadata::Virtual => todo!(),
        }
    }
}

pub enum Permissions {
    Physical(std::fs::Permissions),
    Virtual,
}

impl Permissions {
    #[cfg(windows)]
    pub fn set_readonly(
        &mut self,
        readonly: bool,
    ) {
        match self {
            Permissions::Physical(permissions) => permissions.set_readonly(readonly),
            Permissions::Virtual => todo!(),
        }
    }

    #[cfg(windows)]
    pub fn readonly(&self) -> bool {
        match self {
            Permissions::Physical(permissions) => permissions.readonly(),
            Permissions::Virtual => todo!(),
        }
    }
}

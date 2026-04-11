use tokio::{
    fs::{self, File},
    io::{self, AsyncWriteExt},
};

pub struct Cache {}

impl Cache {
    pub async fn init() -> io::Result<()> {
        fs::create_dir_all(".cache").await?;
        Ok(())
    }

    #[inline]
    pub fn format_path(avatar: &str) -> String {
        format!(".cache/avatar_{avatar}.webp")
    }   

    #[inline]
    async fn path_exists(path: &str) -> bool {
        fs::try_exists(path).await.unwrap_or(false)
    }

    pub async fn save(image: &[u8], avatar: &str) -> io::Result<()> {
        let path = Self::format_path(avatar);
        if Self::path_exists(&path).await {
            return Ok(());
        }

        let mut file = File::create(path).await?;
        file.write_all(image).await?;
        Ok(())
    }

    pub async fn get(avatar: &str) -> Option<Vec<u8>> {
        let path = Self::format_path(avatar);
        fs::read(path).await.ok()
    }
}

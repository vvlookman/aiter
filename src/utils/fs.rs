use std::path::Path;

pub fn extract_filename_from_path(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .trim()
        .to_string()
}

pub fn extract_filestem_from_path(path: &Path) -> String {
    path.file_stem()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {

    use std::{path::PathBuf, str::FromStr};

    use super::*;

    #[test]
    fn test_extract_filestem_from_path() {
        assert_eq!(
            extract_filestem_from_path(&PathBuf::from_str(r"~/书名 (作者) .epub").unwrap()),
            r"书名 (作者)"
        );
    }
}

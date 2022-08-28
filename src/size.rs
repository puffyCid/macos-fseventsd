use std::path::Path;

use log::warn;

// Check if provided file path is larger the max file size
pub(crate) fn get_file_size(path: &str) -> bool {
    let size_results = Path::new(&path).metadata();
    let file_size = match size_results {
        Ok(results) => results.len(),
        Err(err) => {
            warn!(
                "[macos-fsevents] Can not determine file size for fsevents file {}, error: {:?}",
                path, err
            );
            return false;
        }
    };

    let max_size = 2147483648; // 2GB
    if file_size < max_size {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::get_file_size;

    #[test]
    fn test_get_file_size() {
        let path = "/bin/ls";
        let result = get_file_size(path);
        assert_eq!(result, true)
    }
}

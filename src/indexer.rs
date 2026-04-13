use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub path: PathBuf,
}

pub fn build_index() -> Vec<AppEntry> {
    let mut index = Vec::new();

    if let Some(mut path) = dirs::data_dir() {
        path.push("Microsoft\\Windows\\Start Menu\\Programs");
        scan_directory(&path, &mut index, &["lnk"], 5); 
    }

    let sys_start_menu = Path::new("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs");
    scan_directory(sys_start_menu, &mut index, &["lnk"], 5);

    let sys32 = Path::new("C:\\Windows\\System32");
    scan_directory(sys32, &mut index, &["exe"], 1); 

    if let Some(desktop) = dirs::desktop_dir() {
        scan_directory(&desktop, &mut index, &["lnk", "exe"], 2);
    }
    if let Some(documents) = dirs::document_dir() {
        scan_directory(&documents, &mut index, &["exe", "lnk", "pdf"], 3);
    }
    if let Some(downloads) = dirs::download_dir() {
        scan_directory(&downloads, &mut index, &["exe", "msi"], 2);
    }
    index.sort_by(|a, b| a.name.cmp(&b.name));
    index.dedup_by(|a, b| a.name == b.name);

    index
}

fn scan_directory(dir: &Path, index: &mut Vec<AppEntry>, allowed_extensions: &[&str], max_depth: usize) {
    if !dir.exists() { return; }

    for entry in WalkDir::new(dir).max_depth(max_depth).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if allowed_extensions.contains(&ext.to_lowercase().as_str()) {
                    let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                    
                    if !name.to_lowercase().contains("uninstall") && !name.to_lowercase().contains("desinstalar") {
                        index.push(AppEntry { 
                            name, 
                            path: path.to_path_buf() 
                        });
                    }
                }
            }
        }
    }
}
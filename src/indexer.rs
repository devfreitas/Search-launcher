use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub path: PathBuf,
}

pub fn build_index() -> Vec<AppEntry> {
    let mut index = Vec::new();

    // 1. Menu Iniciar (Utilizador e Sistema)
    if let Some(mut path) = dirs::data_dir() {
        path.push("Microsoft\\Windows\\Start Menu\\Programs");
        scan_directory(&path, &mut index, &["lnk"], 5, false); 
    }
    let sys_start_menu = Path::new("C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs");
    scan_directory(sys_start_menu, &mut index, &["lnk"], 5, false);

    // 2. Apps Nativas Clássicas (System32)
    let sys32 = Path::new("C:\\Windows\\System32");
    scan_directory(sys32, &mut index, &["exe"], 1, false); 

    // 3. Pastas Pessoais (Com permissão para indexar as pastas em si)
    if let Some(desktop) = dirs::desktop_dir() {
        scan_directory(&desktop, &mut index, &["lnk", "exe"], 2, true);
    }
    if let Some(documents) = dirs::document_dir() {
        scan_directory(&documents, &mut index, &["exe", "lnk", "pdf"], 3, true);
    }
    if let Some(downloads) = dirs::download_dir() {
        scan_directory(&downloads, &mut index, &["exe", "msi"], 2, true);
    }

    // 4. Instalações de Programas (Program Files e Local AppData)
    let prog_files = Path::new("C:\\Program Files");
    scan_directory(prog_files, &mut index, &["exe"], 3, false); 
    
    let prog_files_x86 = Path::new("C:\\Program Files (x86)");
    scan_directory(prog_files_x86, &mut index, &["exe"], 3, false);

    if let Some(local_appdata) = dirs::data_local_dir() {
        scan_directory(&local_appdata, &mut index, &["exe", "lnk"], 4, false);
    }

    // 5. RADAR UNIVERSAL: Microsoft Store Apps (UWP) - ESTRATÉGIA TEXTO BRUTO
    // O comando 'chcp 65001' força o Windows a usar UTF-8 para não quebrar acentos (ex: Calculadora)
    if let Ok(output) = Command::new("powershell")
        .args(&[
            "-NoProfile", 
            "-Command", 
            "chcp 65001 >$null; Get-StartApps | ForEach-Object { \"$($_.Name)|$($_.AppID)\" }"
        ])
        .output() 
    {
        // from_utf8_lossy garante que não há falhas mesmo com caracteres estranhos
        let result_string = String::from_utf8_lossy(&output.stdout);
        
        for line in result_string.lines() {
            let parts: Vec<&str> = line.splitn(2, '|').collect();
            if parts.len() == 2 {
                let name = parts[0].trim().to_string();
                let app_id = parts[1].trim().to_string();
                
                if !name.is_empty() {
                    index.push(AppEntry {
                        name,
                        path: PathBuf::from(format!("UWP:{}", app_id)),
                    });
                }
            }
        }
    }

    // Limpeza de duplicados e ordenação
    index.sort_by(|a, b| a.name.cmp(&b.name));
    index.dedup_by(|a, b| a.name == b.name);

    index
}

fn scan_directory(dir: &Path, index: &mut Vec<AppEntry>, allowed_extensions: &[&str], max_depth: usize, include_dirs: bool) {
    if !dir.exists() { return; }

    let blacklist = [
        "msinfo", "unins", "desinstalar", "helper", "setup", "update",
        "diagnostics", "crash", "coretools", "vcredist", "systemsettings", 
        "toastnotification", "host", "broker", "service", "microsoft.windows.",
        "softwarelogo", "adminflows", "sysinfo"
    ];

    for entry in WalkDir::new(dir).max_depth(max_depth).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let name_lower = name.to_lowercase();

        if blacklist.iter().any(|term| name_lower.contains(term)) {
            continue;
        }

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if allowed_extensions.contains(&ext.to_lowercase().as_str()) {
                    index.push(AppEntry { name, path: path.to_path_buf() });
                }
            }
        } 
        else if include_dirs && path.is_dir() {
            if !name.starts_with('.') {
                index.push(AppEntry { 
                    name: format!("📁 {}", name),
                    path: path.to_path_buf() 
                });
            }
        }
    }
}
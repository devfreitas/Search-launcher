use std::path::Path;
use eframe::egui;
use windows::Win32::UI::Shell::ExtractIconExW;
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};
use windows::Win32::Graphics::Gdi::{
    GetDC, ReleaseDC, GetObjectW, GetDIBits, BITMAP, BITMAPINFOHEADER, BITMAPINFO,
    DIB_RGB_COLORS, BI_RGB, DeleteObject
};
use std::collections::HashMap;
use std::os::windows::ffi::OsStrExt;

pub struct IconManager {
    cache: HashMap<String, egui::TextureHandle>,
    usage_order: Vec<String>,
    max_size: usize,
}

impl IconManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            usage_order: Vec::new(),
            max_size,
        }
    }

    pub fn get_icon(&mut self, ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
        let path_str = path.to_str()?.to_string();

        // HIT
        if let Some(handle) = self.cache.get(&path_str) {
            return Some(handle.clone());
        }

        // MISS
        if let Some(image) = extract_win_icon(path) {
            let name = format!("icon_{}", path_str);
            let texture = ctx.load_texture(name, image, Default::default());
            
            // LRU
            if self.cache.len() >= self.max_size {
                if let Some(oldest) = self.usage_order.get(0).cloned() {
                    self.cache.remove(&oldest);
                    self.usage_order.remove(0);
                }
            }

            self.cache.insert(path_str.clone(), texture.clone());
            self.usage_order.push(path_str);
            return Some(texture);
        }

        None
    }
}

fn extract_win_icon(path: &Path) -> Option<egui::ColorImage> {
    unsafe {
        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut large_icon = [HICON::default(); 1];
        if ExtractIconExW(windows::core::PCWSTR(wide_path.as_ptr()), 0, Some(large_icon.as_mut_ptr()), None, 1) > 0 {
            let hicon = large_icon[0];
            if hicon.is_invalid() { return None; }

            let mut icon_info = ICONINFO::default();
            if GetIconInfo(hicon, &mut icon_info).is_ok() {
                let mut bmp = BITMAP::default();
                let bytes = std::mem::size_of::<BITMAP>() as i32;
                
                if GetObjectW(icon_info.hbmColor, bytes, Some(&mut bmp as *mut _ as *mut _)) > 0 {
                    let width = bmp.bmWidth;
                    let height = bmp.bmHeight;
                    
                    let dc = GetDC(None);
                    let mut bmi = BITMAPINFO {
                        bmiHeader: BITMAPINFOHEADER {
                            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                            biWidth: width,
                            biHeight: -height,
                            biPlanes: 1,
                            biBitCount: 32,
                            biCompression: BI_RGB.0 as u32,
                            ..Default::default()
                        },
                        ..Default::default()
                    };

                    let mut pixels: Vec<u8> = vec![0; (width * height * 4) as usize];
                    
                    if GetDIBits(dc, icon_info.hbmColor, 0, height as u32, Some(pixels.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS) > 0 {
                        ReleaseDC(None, dc);
                        
                        // BGRA -> RGBA
                        for chunk in pixels.chunks_exact_mut(4) {
                            chunk.swap(0, 2); 
                        }

                        let image = egui::ColorImage::from_rgba_unmultiplied(
                            [width as usize, height as usize],
                            &pixels,
                        );

                        DeleteObject(icon_info.hbmColor);
                        DeleteObject(icon_info.hbmMask);
                        DestroyIcon(hicon).ok();
                        
                        return Some(image);
                    }
                    ReleaseDC(None, dc);
                }
                DeleteObject(icon_info.hbmColor);
                DeleteObject(icon_info.hbmMask);
            }
            DestroyIcon(hicon).ok();
        }
    }
    None
}
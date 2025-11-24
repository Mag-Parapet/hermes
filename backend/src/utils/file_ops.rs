use std::fs;
use std::path::Path;
use image::{ImageReader, ImageFormat, GenericImageView, ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder}; 
use std::io::{Cursor, BufWriter, Write};
use uuid::Uuid;
use blurhash::encode; 

pub struct FileOps;

impl FileOps {
    pub fn ensure_storage_exists(root_path: &str) {
        if !Path::new(root_path).exists() {
            fs::create_dir_all(root_path).expect("Failed to create root storage directory");
        }
    }

    pub fn is_allowed_type(filename: &str, allowed_types: &str) -> bool {
        if allowed_types == "*" || allowed_types.trim().is_empty() {
            return true;
        }
        let ext = Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        let types: Vec<&str> = allowed_types.split(',').map(|s| s.trim()).collect();
        types.contains(&ext.as_str())
    }

    /// Helper to save an image variant with compression quality applied.
    fn save_variant(
        img: &image::DynamicImage,
        path: &str,
        format: ImageFormat,
        compression_enabled: bool,
    ) -> Result<(), String> {
        let file = fs::File::create(path).map_err(|e| e.to_string())?;
        let writer = BufWriter::new(file);

        match format {
            ImageFormat::Jpeg => {
                let quality = if compression_enabled { 85 } else { 100 };
                let mut encoder = JpegEncoder::new_with_quality(writer, quality);
                let rgb_img = img.to_rgb8();
                encoder.write_image(
                    rgb_img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgb8
                ).map_err(|e| format!("JPEG encoding failed: {}", e))
            },
            ImageFormat::WebP => {
                // WebP encoding with proper lossy compression using webp crate
                let rgba_img = img.to_rgba8();
                let (width, height) = (img.width(), img.height());
                
                let webp_data = if compression_enabled {
                    // Lossy compression with quality 80 for smaller files
                    webp::Encoder::from_rgba(&rgba_img, width, height)
                        .encode(80.0)
                } else {
                    // Lossy compression with quality 95 for better quality
                    webp::Encoder::from_rgba(&rgba_img, width, height)
                        .encode(95.0)
                };
                
                let mut writer = writer;
                writer.write_all(&webp_data).map_err(|e| format!("WebP write failed: {}", e))
            },
            ImageFormat::Png => {
                let encoder = PngEncoder::new(writer);
                let rgba_img = img.to_rgba8();
                encoder.write_image(
                    rgba_img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgba8
                ).map_err(|e| format!("PNG encoding failed: {}", e))
            },
            _ => {
                img.save_with_format(path, format)
                    .map_err(|e| format!("Image encoding failed: {}", e))
            }
        }
    }

    /// Returns: (File Size, Content Type, Option<BlurHashString>)
    pub fn save_file(
        storage_root: &str,
        storage_id: Uuid,
        file_id: Uuid,
        original_filename: &str,
        data: &[u8],
        compression_enabled: bool,
        resize_max: i32,
        format_ext: &str, 
    ) -> Result<(i64, String, Option<String>), String> {
        
        let folder_path = format!("{}/{}", storage_root, storage_id);
        fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;

        let is_image = original_filename.to_lowercase().ends_with(".jpg") 
                    || original_filename.to_lowercase().ends_with(".jpeg") 
                    || original_filename.to_lowercase().ends_with(".png") 
                    || original_filename.to_lowercase().ends_with(".webp");
        
        if is_image {
            let img = ImageReader::new(Cursor::new(data))
                .with_guessed_format()
                .map_err(|e| format!("Failed to read image format: {}", e))?
                .decode()
                .map_err(|e| format!("Failed to decode image: {}", e))?;

            // 1. CALCULATE BLURHASH
            let small_img = img.resize(50, 50, image::imageops::FilterType::Triangle);
            let (width, height) = small_img.dimensions();
            let blurhash_string = encode(4, 3, width, height, &small_img.to_rgba8().into_raw())
                .map_err(|e| format!("Failed to calculate blurhash: {}", e))?;

            // 2. SETUP FORMATS
            let target_format = match format_ext.to_lowercase().as_str() {
                "png" => ImageFormat::Png,
                "webp" => ImageFormat::WebP,
                _ => ImageFormat::Jpeg, 
            };
            let new_ext = format_ext.to_lowercase();
            let file_stem = file_id.to_string();

            // 3. SAVE MAIN IMAGE (Resized + Compressed Quality)
            let mut main_img = img.clone();
            let (original_width, original_height) = img.dimensions();
            let max_dimension = original_width.max(original_height);
            
            // Only resize if image is larger than resize_max
            if compression_enabled && max_dimension > resize_max as u32 {
                main_img = main_img.resize(resize_max as u32, resize_max as u32, image::imageops::FilterType::Lanczos3);
            }

            let output_filename = format!("{}.{}", file_stem, new_ext);
            let output_path = format!("{}/{}", folder_path, output_filename);
            
            Self::save_variant(&main_img, &output_path, target_format, compression_enabled)
                .map_err(|e| format!("Failed to save main image: {}", e))?;

            // 4. GENERATE RESPONSIVE VARIANTS (srcset)
            let variants = vec![("sm", 480), ("md", 800), ("lg", 1200)];
            let original_width = img.width();

            for (suffix, target_w) in variants {
                if original_width > target_w {
                    let resized = img.resize(target_w, target_w, image::imageops::FilterType::Lanczos3);
                    let variant_path = format!("{}/{}_{}.{}", folder_path, file_stem, suffix, new_ext);
                    
                    // Save variant with compression applied
                    let _ = Self::save_variant(&resized, &variant_path, target_format, compression_enabled);
                }
            }
            
            let metadata = fs::metadata(&output_path).map_err(|e| e.to_string())?;
            return Ok((metadata.len() as i64, format!("image/{}", new_ext), Some(blurhash_string)));
        }

        // Non-Image Logic (remains unchanged)
        let ext = Path::new(original_filename).extension().and_then(|e| e.to_str()).unwrap_or("bin");
        let output_filename = format!("{}.{}", file_id, ext);
        let file_path = format!("{}/{}", folder_path, output_filename);

        fs::write(&file_path, data).map_err(|e| e.to_string())?;
        
        Ok((data.len() as i64, "raw".to_string(), None))
    }

    pub fn delete_file(file_path: &str) -> Result<(), String> {
        if Path::new(file_path).exists() {
            fs::remove_file(file_path).map_err(|e| format!("Failed to delete file: {}", e))?;
        }

        let path_obj = Path::new(file_path);
        if let Some(stem) = path_obj.file_stem().and_then(|s| s.to_str()) {
            if let Some(ext) = path_obj.extension().and_then(|s| s.to_str()) {
                let parent = path_obj.parent().unwrap_or(Path::new("."));
                for suffix in ["sm", "md", "lg"] {
                    let variant_path = parent.join(format!("{}_{}.{}", stem, suffix, ext));
                    if variant_path.exists() { let _ = fs::remove_file(variant_path); }
                }
            }
        }
        Ok(())
    }

    pub fn get_physical_path(storage_root: &str, storage_id: Uuid, file_id: Uuid, file_type: &str, original_name: &str) -> String {
        if file_type.starts_with("image/") {
             let ext = file_type.strip_prefix("image/").unwrap_or("jpeg");
             return format!("{}/{}/{}.{}", storage_root, storage_id, file_id, ext);
        }
        let ext = Path::new(original_name).extension().and_then(|e| e.to_str()).unwrap_or("bin");
        format!("{}/{}/{}.{}", storage_root, storage_id, file_id, ext)
    }
}
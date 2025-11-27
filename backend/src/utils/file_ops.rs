use std::fs;
use std::path::Path;
use image::{ImageReader, ImageFormat, GenericImageView, ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder}; 
use std::io::{Cursor, BufWriter, Write};
use uuid::Uuid;
use blurhash::encode; 
use webp; 
// Removed std::thread

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
        let file = fs::File::create(path).map_err(|e| format!("Failed to create file at {}: {}", path, e))?;
        let mut writer = BufWriter::new(file);

        match format {
            ImageFormat::Jpeg => {
                let quality = if compression_enabled { 75 } else { 100 };
                let encoder = JpegEncoder::new_with_quality(&mut writer, quality);
                let rgb_img = img.to_rgb8(); 
                encoder.write_image(
                    rgb_img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgb8
                ).map_err(|e| format!("JPEG encoding failed: {}", e))?;
            },
            ImageFormat::WebP => {
                let rgba_img = img.to_rgba8();
                let (width, height) = (img.width(), img.height());
                let quality = if compression_enabled { 70.0 } else { 95.0 };

                let webp_data = webp::Encoder::from_rgba(&rgba_img, width, height)
                    .encode(quality);
                
                writer.write_all(&webp_data.to_vec()).map_err(|e| format!("WebP write failed: {}", e))?;
            },
            ImageFormat::Png => {
                let encoder = PngEncoder::new(&mut writer);
                let rgba_img = img.to_rgba8();
                encoder.write_image(
                    rgba_img.as_raw(),
                    img.width(),
                    img.height(),
                    image::ExtendedColorType::Rgba8
                ).map_err(|e| format!("PNG encoding failed: {}", e))?;
            },
            _ => {
                img.save_with_format(path, format)
                    .map_err(|e| format!("Image encoding failed: {}", e))?;
                return Ok(());
            }
        }

        writer.flush().map_err(|e| format!("Failed to flush file buffer: {}", e))?;
        Ok(())
    }

    pub fn save_file(
        storage_root: String,
        storage_id: Uuid,
        file_id: Uuid,
        original_filename: String,
        data: Vec<u8>,
        compression_enabled: bool,
        resize_max: i32,
        format_ext: String, 
    ) -> Result<(i64, String, Option<String>), String> {
        
        let folder_path = format!("{}/{}", storage_root, storage_id);
        fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;

        let is_image = original_filename.to_lowercase().ends_with(".jpg") 
                    || original_filename.to_lowercase().ends_with(".jpeg") 
                    || original_filename.to_lowercase().ends_with(".png") 
                    || original_filename.to_lowercase().ends_with(".webp");
        
        if is_image {
            // 1. DECODE IMAGE
            let img = ImageReader::new(Cursor::new(&data))
                .with_guessed_format()
                .map_err(|e| format!("Failed to read image format: {}", e))?
                .decode()
                .map_err(|e| format!("Failed to decode image: {}", e))?;

            // 2. CALCULATE BLURHASH
            let small_img = img.thumbnail(50, 50);
            let (width, height) = small_img.dimensions();
            let blurhash_string = encode(4, 3, width, height, &small_img.to_rgba8().into_raw())
                .map_err(|e| format!("Failed to calculate blurhash: {}", e))?;

            // 3. DETERMINE FORMATS
            let target_format = if compression_enabled {
                match format_ext.to_lowercase().as_str() {
                    "png" => ImageFormat::Png,
                    "webp" => ImageFormat::WebP,
                    _ => ImageFormat::Jpeg, 
                }
            } else {
                let ext = Path::new(&original_filename)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("jpg")
                    .to_lowercase();
                
                match ext.as_str() {
                    "png" => ImageFormat::Png,
                    "webp" => ImageFormat::WebP,
                    _ => ImageFormat::Jpeg,
                }
            };

            let new_ext = match target_format {
                ImageFormat::Png => "png",
                ImageFormat::WebP => "webp",
                _ => "jpeg",
            };
            
            let file_stem = file_id.to_string();
            let output_filename = format!("{}.{}", file_stem, new_ext);
            let output_path = format!("{}/{}", folder_path, output_filename);

            // 4. SAVE MAIN IMAGE
            // We use an Option to hold the resized image if we created one, 
            // so we can use it as the starting point for variants (optimization).
            let mut resized_main_img: Option<image::DynamicImage> = None;

            if !compression_enabled {
                // OPTIMIZATION: Write RAW bytes directly.
                fs::write(&output_path, &data).map_err(|e| e.to_string())?;
            } else {
                // Resize (if needed) and Re-encode
                let mut main_img_ref = &img;
                
                if img.width() > resize_max as u32 || img.height() > resize_max as u32 {
                    // Use Thumbnail (Fastest downscaling)
                    let resized = img.thumbnail(resize_max as u32, resize_max as u32);
                    resized_main_img = Some(resized);
                    main_img_ref = resized_main_img.as_ref().unwrap();
                }

                Self::save_variant(main_img_ref, &output_path, target_format, compression_enabled)
                    .map_err(|e| format!("Failed to save main image: {}", e))?;
            }

            // Capture metadata
            let metadata = fs::metadata(&output_path).map_err(|e| format!("Failed to read metadata for {}: {}", output_path, e))?;
            let file_size = metadata.len() as i64;
            let mime_type = format!("image/{}", new_ext);

            // 5. GENERATE VARIANTS (SEQUENTIAL / SAME THREAD)
            // Start the cascade from the already resized image (if it exists) to save processing, 
            // otherwise start from the full original `img`.
            let start_source = resized_main_img.as_ref().unwrap_or(&img);
            
            // We clone the start_source to begin the modification chain. 
            // This is a RAM copy, which is fast.
            let mut current_source = start_source.clone(); 

            let variants = vec![("lg", 1200), ("md", 800), ("sm", 480)];

            for (suffix, target_w) in variants {
                if current_source.width() > target_w {
                    let resized = current_source.thumbnail(target_w, target_w);
                    let variant_path = format!("{}/{}_{}.{}", folder_path, file_stem, suffix, new_ext);
                    
                    // Generate variant
                    let _ = Self::save_variant(&resized, &variant_path, target_format, compression_enabled);
                    
                    // Cascade: Use this smaller image as the source for the next smaller variant
                    current_source = resized;
                }
            }

            return Ok((file_size, mime_type, Some(blurhash_string)));
        }

        // Non-Image Logic
        let ext = Path::new(&original_filename).extension().and_then(|e| e.to_str()).unwrap_or("bin");
        let output_filename = format!("{}.{}", file_id, ext);
        let file_path = format!("{}/{}", folder_path, output_filename);

        fs::write(&file_path, &data).map_err(|e| e.to_string())?;
        
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

    pub fn delete_folder(folder_path: &str) -> Result<(), String> {
        if Path::new(folder_path).exists() {
            fs::remove_dir_all(folder_path).map_err(|e| format!("Failed to delete folder {}: {}", folder_path, e))?;
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
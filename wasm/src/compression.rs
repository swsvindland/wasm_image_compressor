use crate::error::ConvertError;
use crate::media_type::MediaType;
use exif::{In, Tag};
use image::{DynamicImage, ImageFormat, ImageReader};
use js_sys::Uint8Array;
use pixlzr::{FilterType, Pixlzr};
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

const MAX_ICO_SIZE: u32 = 256;
const THUMBNAIL_SIZE: u32 = 64;
const DEFAULT_COMPRESSION: f32 = 0.8;

#[derive(Clone, Copy)]
pub(super) enum CompressionFactor {
    Value(f32),
    Skip,
}

pub fn parse_compression_factor(compression_factor: &JsValue) -> CompressionFactor {
    match compression_factor.as_f64() {
        Some(1.0) => CompressionFactor::Skip,
        Some(v) => CompressionFactor::Value(v as f32),
        None => CompressionFactor::Value(DEFAULT_COMPRESSION),
    }
}

pub async fn convert_image_internal(
    file_input: &JsValue,
    src_type: &str,
    target_type: &str,
    compression_factor: CompressionFactor,
    max_size: Option<u32>,
    max_file_size_kb: Option<u32>,
) -> Result<Vec<u8>, JsValue> {
    let file_data = match file_input {
        v if v.is_string() => fetch_image(&v.as_string().unwrap()).await?,
        v if v.is_instance_of::<Uint8Array>() => Uint8Array::new(v).to_vec(),
        _ => {
            return Err(JsValue::from_str(
                "Invalid input type. Must be a URL or Uint8Array.",
            ))
        }
    };

    let src_media_type = MediaType::from_mime_type(src_type);
    let img = load_image(&file_data, src_media_type)
        .map_err(|_| JsValue::from_str("Failed to load image"))?;

    let src_format = ImageFormat::from_mime_type(src_type);
    let target_format = ImageFormat::from_mime_type(target_type);
    let mut processed_img = process_image(img, src_format, target_format, max_size);

    let mut current_compression = compression_factor;
    let mut output = parallel_write_image(
        &processed_img,
        target_format,
        current_compression,
        &file_data,
    )
    .map_err(|_| JsValue::from_str("Error writing image"))?;

    if let Some(max_kb) = max_file_size_kb {
        let max_bytes = (max_kb as usize) * 1024;
        while output.len() > max_bytes {
            // Apply 0.8 compression factor and 10% off width/height
            match current_compression {
                CompressionFactor::Value(v) => current_compression = CompressionFactor::Value(v * 0.8),
                CompressionFactor::Skip => current_compression = CompressionFactor::Value(0.8),
            }

            let new_width = (processed_img.width() as f32 * 0.9).round() as u32;
            let new_height = (processed_img.height() as f32 * 0.9).round() as u32;

            if new_width == 0 || new_height == 0 {
                break;
            }

            processed_img = processed_img.resize_exact(
                new_width,
                new_height,
                image::imageops::FilterType::Lanczos3,
            );

            output = parallel_write_image(
                &processed_img,
                target_format,
                current_compression,
                &file_data,
            )
            .map_err(|_| JsValue::from_str("Error writing image"))?;
        }
    }

    Ok(output)
}

async fn fetch_image(url: &str) -> Result<Vec<u8>, JsValue> {
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(url, &opts)?;
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window available"))?;
    let resp: Response = JsFuture::from(window.fetch_with_request(&request))
        .await?
        .dyn_into()?;
    let data = JsFuture::from(resp.array_buffer()?).await?;
    Ok(Uint8Array::new(&data).to_vec())
}

fn load_image(file: &[u8], source_type: Option<MediaType>) -> Result<DynamicImage, ConvertError> {
    let mut reader = ImageReader::new(Cursor::new(file));
    if let Some(MediaType::Raster(file_type)) = source_type {
        reader.set_format(file_type);
    }
    let mut img = reader.decode().map_err(|e| {
        ConvertError::UnknownFileType(format!("Failed to load image: {}", e))
    })?;

    if let Ok(exif) = exif::Reader::new().read_from_container(&mut Cursor::new(file)) {
        if let Some(field) = exif.get_field(Tag::Orientation, In::PRIMARY) {
            if let Some(v) = field.value.get_uint(0) {
                match v {
                    2 => img = img.flipv(),
                    3 => img = img.rotate180(),
                    4 => img = img.fliph(),
                    5 => img = img.rotate90().flipv(),
                    6 => img = img.rotate90(),
                    7 => img = img.rotate270().flipv(),
                    8 => img = img.rotate270(),
                    _ => {}
                }
            }
        }
    }

    Ok(img)
}

fn process_image(
    img: DynamicImage,
    source_type: Option<ImageFormat>,
    target_type: Option<ImageFormat>,
    max_size: Option<u32>,
) -> DynamicImage {
    let target = target_type.unwrap_or(ImageFormat::WebP);
    let mut img = if source_type == Some(ImageFormat::Hdr) {
        DynamicImage::ImageRgba8(img.to_rgba8())
    } else {
        img
    };

    if let Some(max_size) = max_size {
        let (width, height) = (img.width(), img.height());
        let (n_width, n_height) = if width > height {
            let n_height = ((max_size as f64 / width as f64) * height as f64).round() as u32;
            (max_size, n_height.max(1))
        } else {
            let n_width = ((max_size as f64 / height as f64) * width as f64).round() as u32;
            (n_width.max(1), max_size)
        };

        img = img.resize(n_width, n_height, image::imageops::FilterType::Lanczos3);
    }

    match target {
        ImageFormat::Jpeg
        | ImageFormat::Qoi
        | ImageFormat::Farbfeld
        | ImageFormat::Pnm
        | ImageFormat::Tga => DynamicImage::ImageRgb8(img.to_rgb8()),
        ImageFormat::Ico => img.resize_exact(
            MAX_ICO_SIZE,
            MAX_ICO_SIZE,
            image::imageops::FilterType::Lanczos3,
        ),
        ImageFormat::OpenExr => DynamicImage::ImageRgba32F(img.to_rgba32f()),
        _ => img,
    }
}

fn write_image(
    img: &DynamicImage,
    file_type: Option<ImageFormat>,
    compression_factor: CompressionFactor,
    original_data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    let target_type = file_type.unwrap_or(ImageFormat::WebP);
    let mut buffer = Vec::with_capacity(original_data.len());
    let final_img = match compression_factor {
        CompressionFactor::Value(compression) => {
            let mut pix = Pixlzr::from_image(img, THUMBNAIL_SIZE, THUMBNAIL_SIZE);
            pix.shrink_by(FilterType::Lanczos3, compression);
            pix.to_image(FilterType::Nearest)
        }
        CompressionFactor::Skip => img.clone(),
    };

    final_img
        .write_to(&mut Cursor::new(&mut buffer), target_type)
        .map_err(|_| "Failed to write image")?;

    Ok(if buffer.len() > original_data.len() {
        original_data.to_vec()
    } else {
        buffer
    })
}

fn parallel_write_image(
    img: &DynamicImage,
    file_type: Option<ImageFormat>,
    compression_factor: CompressionFactor,
    original_data: &[u8],
) -> Result<Vec<u8>, &'static str> {
    rayon::scope(|s| {
        s.spawn(|_| {
            let _ = write_image(img, file_type, compression_factor, original_data);
        })
    });
    write_image(img, file_type, compression_factor, original_data)
}

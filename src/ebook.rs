use anyhow::Result;
use epub::doc::EpubDoc;
use log::{error, info};
use sanitize_filename::sanitize;
use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

/// Reads the e-book, extracts chapter texts, and saves images to the specified folder
pub fn read_ebook<P: AsRef<Path>>(
    path: P,
    images_dir: &Path,
) -> Result<(
    EpubDoc<BufReader<File>>,
    Vec<String>,
    Vec<Vec<String>>,
    HashMap<String, String>,
)> {
    let file = File::open(&path)?;
    let buf_reader = BufReader::new(file);

    let mut doc = EpubDoc::from_reader(buf_reader)?;

    let mut chapters_content = Vec::new();
    let mut chapters_images = Vec::new();
    let total_chapters = doc.get_num_pages();
    info!("Total chapters: {}", total_chapters);

    // Extract and save images
    let image_map = extract_images(&mut doc, images_dir)?;

    // Reset to the beginning of the document
    doc.set_current_page(0);

    for chapter_index in 0..total_chapters {
        if let Some((chapter_content, _mime)) = doc.get_current_str() {
            // Convert HTML content to plain text
            let text = html2text::from_read(chapter_content.as_bytes(), usize::MAX)?;
            chapters_content.push(text);

            // Get images associated with this chapter
            let chapter_images = image_map.get(&chapter_index).cloned().unwrap_or_default();
            chapters_images.push(chapter_images);
        } else {
            error!(
                "Error getting content of chapter {}",
                doc.get_current_page()
            );
            chapters_images.push(Vec::new());
        }
        doc.go_next();
    }

    let metadata = get_ebook_metadata(&doc);

    Ok((doc, chapters_content, chapters_images, metadata))
}

/// Extracts the table of contents from the e-book
pub fn extract_table_of_contents<R: std::io::Read + std::io::Seek>(
    doc: &EpubDoc<R>,
) -> Vec<String> {
    let mut toc = Vec::new();

    for nav_point in &doc.toc {
        // Get the section title
        let title = &nav_point.label;
        toc.push(title.clone());
    }

    toc
}

/// Extracts images from the e-book and saves them to the specified folder
fn extract_images<R: std::io::Read + std::io::Seek>(
    doc: &mut EpubDoc<R>,
    images_dir: &Path,
) -> Result<HashMap<usize, Vec<String>>> {
    let mut image_map: HashMap<usize, Vec<String>> = HashMap::new();

    // Collect image resources
    let image_resources: Vec<(String, PathBuf)> = doc
        .resources
        .iter()
        .filter_map(|(id, (path, mime))| {
            if mime.starts_with("image/") {
                Some((id.clone(), path.clone()))
            } else {
                None
            }
        })
        .collect();

    for (resource_id, resource_path) in image_resources {
        // Get the image content
        if let Some((data, mime)) = doc.get_resource(&resource_id) {
            // Determine file extension based on MIME type
            let extension = match mime.as_str() {
                "image/jpeg" => "jpg",
                "image/png" => "png",
                "image/gif" => "gif",
                "image/svg+xml" => "svg",
                _ => "bin", // default binary extension
            };

            // Convert PathBuf to String for filename
            let resource_path_str = resource_path.to_string_lossy();
            // Create a safe filename
            let filename = format!("{}.{}", sanitize(&resource_path_str), extension);
            let image_path = images_dir.join(&filename);

            // Create directory if it doesn't exist
            if let Some(parent) = image_path.parent() {
                create_dir_all(parent)?;
            }

            // Save the image to disk
            let mut file = File::create(&image_path)?;
            file.write_all(&data)?;

            // Map image to chapter (simplified mapping)
            let chapter_index = doc.get_current_page();
            image_map
                .entry(chapter_index)
                .or_insert_with(Vec::new)
                .push(filename);
        }
    }

    Ok(image_map)
}

// Add a function to get metadata from the e-book
pub fn get_ebook_metadata<R: std::io::Read + std::io::Seek>(
    doc: &EpubDoc<R>,
) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    if let Some(title) = doc.mdata("title") {
        metadata.insert("title".to_string(), title);
    }
    if let Some(author) = doc.mdata("creator") {
        metadata.insert("author".to_string(), author);
    }
    if let Some(language) = doc.mdata("language") {
        metadata.insert("language".to_string(), language);
    }
    metadata
}

use anyhow::Result;
use epub::doc::EpubDoc;
use log::{error, info};
use sanitize_filename::sanitize;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Write};
use std::path::Path;

/// Reads the e-book, extracts chapter texts, and saves images to the specified folder
pub fn read_ebook<P: AsRef<Path>>(
    path: P,
    images_dir: &Path,
) -> Result<(EpubDoc<BufReader<File>>, Vec<String>)> {
    let file = File::open(&path)?;
    let buf_reader = BufReader::new(file);

    let mut doc = EpubDoc::from_reader(buf_reader)?;

    let mut chapters_content = Vec::new();
    let total_chapters = doc.get_num_pages();
    info!("Total chapters: {}", total_chapters);

    // Extract and save images
    extract_images(&mut doc, images_dir)?;

    // Reset to the beginning of the document
    doc.set_current_page(0);

    for _ in 0..total_chapters {
        if let Some((chapter_content, _mime)) = doc.get_current_str() {
            // Convert HTML content to plain text
            let text = html2text::from_read(chapter_content.as_bytes(), usize::MAX)?;
            chapters_content.push(text);
        } else {
            error!(
                "Error getting content of chapter {}",
                doc.get_current_page()
            );
        }
        doc.go_next();
    }

    Ok((doc, chapters_content))
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
) -> Result<()> {
    // Collect image resources
    let image_resources: Vec<(String, std::path::PathBuf)> = doc
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
            let image_path = images_dir.join(filename);

            // Create directory if it doesn't exist
            if let Some(parent) = image_path.parent() {
                create_dir_all(parent)?;
            }

            // Save the image to disk
            let mut file = File::create(&image_path)?;
            file.write_all(&data)?;
        }
    }

    Ok(())
}

use epub::doc::EpubDoc;
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use eyre::Result;
use std::fs::File;
use std::path::Path;

pub fn extract_text_from_epub(file: &str) -> Result<Vec<String>> {
    let mut epub = EpubDoc::new(file)?;
    let mut chapters = Vec::new();
    let mut progress = 0;

    while let Some(content) = epub.get_current_str() {
        chapters.push(content.0.clone());
        epub.go_next();
        progress += 1;
        println!("Extracting chapter {}...", progress);
    }

    Ok(chapters)
}

pub fn extract_images_from_epub(file: &str, output_folder: &str) -> Result<()> {
    let mut epub = EpubDoc::new(file)?;

    for id in epub.resources.keys().cloned().collect::<Vec<_>>() {
        if let Some(res) = epub.get_resource(&id) {
            if res.0.starts_with(b"image/") {
                let image_path = Path::new(output_folder).join(&res.1);
                std::fs::write(image_path, &res.0)?;
            }
        }
    }

    Ok(())
}

pub fn create_epub(output_dir: &Path, summary_path: &Path) -> Result<()> {
    let file = File::create(output_dir.join("summary.epub"))?;
    let mut epub = EpubBuilder::new(ZipLibrary::new()?)?;
    epub.metadata("Title", "Pocket Book Summary")?;
    epub.metadata("Author", "AI Generated")?;

    let summary_content = std::fs::read_to_string(summary_path)?;

    epub.add_content(
        EpubContent::new("summary.html", std::io::Cursor::new(summary_content))
            .title("Summary")
    )?;
    
    epub.generate(file)?;
    Ok(())
}

use regex::Regex;

/// Highlights keywords in the summary
pub fn highlight_keywords(summary: &str, keywords: &[String]) -> String {
    let mut highlighted = summary.to_string();
    for keyword in keywords {
        let regex = Regex::new(&regex::escape(keyword)).unwrap();
        highlighted = regex
            .replace_all(&highlighted, format!("**{}**", keyword))
            .to_string();
    }
    highlighted
}

/// Formats a section of content
pub fn format_section(title: &str, content: &str, format: &str) -> String {
    match format {
        "markdown" => format!("## {}\n\n{}\n", title, content),
        "html" => format!("<h2>{}</h2>\n\n{}\n", title, content),
        _ => content.to_string(),
    }
}

/// Formats the main title
pub fn format_title(title: &str, format: &str) -> String {
    match format {
        "markdown" => format!("# {}\n\n", title),
        "html" => format!("<h1>{}</h1>\n\n", title),
        _ => title.to_string(),
    }
}

/// Formats the glossary
pub fn format_glossary(glossary: &[String], format: &str) -> String {
    if glossary.is_empty() {
        return String::new();
    }
    match format {
        "markdown" => {
            let entries = glossary.join("\n\n");
            format!("# Glossary\n\n{}\n", entries)
        }
        "html" => {
            let entries = glossary
                .iter()
                .map(|entry| format!("<p>{}</p>", entry))
                .collect::<Vec<String>>()
                .join("\n");
            format!("<h1>Glossary</h1>\n\n{}\n", entries)
        }
        _ => glossary.join("\n\n"),
    }
}

/// Formats the references
pub fn format_references(references: &[String], format: &str) -> String {
    if references.is_empty() {
        return String::new();
    }
    match format {
        "markdown" => {
            let entries = references.join("\n");
            format!("# References\n\n{}\n", entries)
        }
        "html" => {
            let entries = references
                .iter()
                .map(|entry| format!("<p>{}</p>", entry))
                .collect::<Vec<String>>()
                .join("\n");
            format!("<h1>References</h1>\n\n{}\n", entries)
        }
        _ => references.join("\n"),
    }
}

/// Formats the additional resources
pub fn format_additional_resources(resources: &[String], format: &str) -> String {
    if resources.is_empty() {
        return String::new();
    }
    match format {
        "markdown" => {
            let entries = resources.join("\n");
            format!("# Additional Resources\n\n{}\n", entries)
        }
        "html" => {
            let entries = resources
                .iter()
                .map(|entry| format!("<p>{}</p>", entry))
                .collect::<Vec<String>>()
                .join("\n");
            format!("<h1>Additional Resources</h1>\n\n{}\n", entries)
        }
        _ => resources.join("\n"),
    }
}

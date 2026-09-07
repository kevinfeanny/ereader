use anyhow::Result;
use epub::doc::EpubDoc;
use std::path::Path;
use crate::config::Config;

pub struct Book {
    pub title: String,
    pub pages: Vec<Vec<String>>,
}

impl Book {
    pub fn load(path: &Path, config: &Config) -> Result<Self> {
        println!("Loading: {}", path.display());

        let mut doc = EpubDoc::new(path)
            .map_err(|e| anyhow::anyhow!("Failed to open epub: {}", e))?;

        let title = doc.mdata("title")
            .map(|m| m.value.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut paragraphs: Vec<String> = Vec::new();

        while doc.go_next() {
            if let Some((content, _mime)) = doc.get_current_str() {
                let text = strip_html(&content);
                for para in text.split('\n') {
                    let trimmed = para.trim().to_string();
                    if !trimmed.is_empty() {
                        paragraphs.push(trimmed);
                    }
                }
            }
        }

        let pages = paginate(paragraphs, config);
        println!("Loaded {} pages", pages.len());

        Ok(Book { title, pages })
    }

    pub fn total_pages(&self) -> usize {
        self.pages.len()
    }

    pub fn get_page(&self, page: usize) -> Option<&Vec<String>> {
        self.pages.get(page)
    }
}

fn strip_html(html: &str) -> String {
    let mut result  = String::new();
    let mut in_tag  = false;
    let mut in_para = false;

    for ch in html.chars() {
        match ch {
            '<' => {
                in_tag = true;
                if in_para {
                    result.push('\n');
                    in_para = false;
                }
            }
            '>' => {
                in_tag  = false;
                in_para = true;
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}

fn wrap_paragraph(para: &str, chars_per_line: usize) -> Vec<String> {
    let mut lines   = Vec::new();
    let mut current = String::new();

    for word in para.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if current.len() + 1 + word.len() <= chars_per_line {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn paginate(paragraphs: Vec<String>, config: &Config) -> Vec<Vec<String>> {
    let mut pages:        Vec<Vec<String>> = Vec::new();
    let mut current_page: Vec<String>      = Vec::new();
    let max_lines     = config.max_lines();
    let chars_per_line = config.chars_per_line();

    for para in &paragraphs {
        let mut lines = wrap_paragraph(para, chars_per_line);
        lines.push(String::new());

        for line in lines {
            if current_page.len() >= max_lines {
                pages.push(current_page.clone());
                current_page = Vec::new();
            }
            current_page.push(line);
        }
    }

    if !current_page.is_empty() {
        pages.push(current_page);
    }

    pages
}
use regex::Regex;
use std::collections::HashMap;
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone)]
pub struct AssDocument {
    pub sections: Vec<Section>,
    pub script_info: HashMap<String, String>,
    pub styles: Vec<Style>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub range: Range,
    #[allow(dead_code)]
    pub content: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Style {
    pub name: String,
    pub fontname: String,
    pub fontsize: u32,
    pub primary_colour: String,
    #[allow(dead_code)]
    pub secondary_colour: String,
    pub range: Range,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub start_time: String,
    pub end_time: String,
    pub style: String,
    pub actor: String,
    pub text: String,
    pub range: Range,
}

#[derive(Debug)]
pub struct AssParser {
    section_regex: Regex,
}

impl AssParser {
    pub fn new() -> Self {
        Self {
            section_regex: Regex::new(r"^\[([^\]]+)\]").unwrap(),
        }
    }

    pub fn parse(&self, text: &str) -> AssDocument {
        let lines: Vec<&str> = text.lines().collect();
        let mut sections = Vec::new();
        let mut script_info = HashMap::new();
        let mut styles = Vec::new();
        let mut events = Vec::new();

        let mut current_section: Option<String> = None;
        let mut current_section_start = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Check for section headers
            if let Some(captures) = self.section_regex.captures(line) {
                // Finish previous section
                if let Some(section_name) = current_section.take() {
                    sections.push(Section {
                        name: section_name,
                        range: Range {
                            start: Position::new(current_section_start as u32, 0),
                            end: Position::new(line_num as u32, 0),
                        },
                        content: lines[current_section_start..line_num]
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    });
                }

                current_section = Some(captures[1].to_string());
                current_section_start = line_num;
                continue;
            }

            // Parse content based on current section
            match current_section.as_deref() {
                Some("Script Info") => {
                    if let Some((key, value)) = self.parse_key_value(line) {
                        script_info.insert(key, value);
                    }
                }
                Some(section) if section.contains("Styles") => {
                    if line.starts_with("Style:") {
                        if let Some(style) = self.parse_style(line, line_num) {
                            styles.push(style);
                        }
                    }
                }
                Some("Events") => {
                    if line.starts_with("Dialogue:") || line.starts_with("Comment:") {
                        if let Some(event) = self.parse_event(line, line_num) {
                            events.push(event);
                        }
                    }
                }
                _ => {}
            }
        }

        // Finish last section
        if let Some(section_name) = current_section {
            sections.push(Section {
                name: section_name,
                range: Range {
                    start: Position::new(current_section_start as u32, 0),
                    end: Position::new(lines.len() as u32, 0),
                },
                content: lines[current_section_start..]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            });
        }

        AssDocument {
            sections,
            script_info,
            styles,
            events,
        }
    }

    fn parse_key_value(&self, line: &str) -> Option<(String, String)> {
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_string();
            let value = line[colon_pos + 1..].trim().to_string();
            Some((key, value))
        } else {
            None
        }
    }

    fn parse_style(&self, line: &str, line_num: usize) -> Option<Style> {
        let parts: Vec<&str> = line.split_once(':')?.1.split(',').collect();
        if parts.len() >= 4 {
            Some(Style {
                name: parts[0].trim().to_string(),
                fontname: parts.get(1).unwrap_or(&"Arial").trim().to_string(),
                fontsize: parts.get(2).unwrap_or(&"20").trim().parse().unwrap_or(20),
                primary_colour: parts.get(3).unwrap_or(&"&Hffffff").trim().to_string(),
                secondary_colour: parts.get(4).unwrap_or(&"&Hffffff").trim().to_string(),
                range: Range {
                    start: Position::new(line_num as u32, 0),
                    end: Position::new(line_num as u32, line.len() as u32),
                },
            })
        } else {
            None
        }
    }

    fn parse_event(&self, line: &str, line_num: usize) -> Option<Event> {
        let event_type = if line.starts_with("Dialogue:") {
            "Dialogue"
        } else {
            "Comment"
        };
        let parts: Vec<&str> = line.split_once(':')?.1.split(',').collect();

        if parts.len() >= 10 {
            Some(Event {
                event_type: event_type.to_string(),
                start_time: parts.get(1).unwrap_or(&"0:00:00.00").trim().to_string(),
                end_time: parts.get(2).unwrap_or(&"0:00:00.00").trim().to_string(),
                style: parts.get(3).unwrap_or(&"Default").trim().to_string(),
                actor: parts.get(4).unwrap_or(&"").trim().to_string(),
                text: parts[9..].join(",").trim().to_string(),
                range: Range {
                    start: Position::new(line_num as u32, 0),
                    end: Position::new(line_num as u32, line.len() as u32),
                },
            })
        } else {
            None
        }
    }

    pub fn format(&self, text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        let mut formatted_lines = Vec::new();
        let mut in_section = false;

        for line in lines {
            let trimmed = line.trim();

            // Section headers
            if self.section_regex.is_match(trimmed) {
                if in_section {
                    formatted_lines.push("".to_string()); // Add blank line before new section
                }
                formatted_lines.push(trimmed.to_string());
                in_section = true;
            } else if trimmed.is_empty() {
                formatted_lines.push("".to_string());
            } else if trimmed.starts_with(';') {
                // Comments
                formatted_lines.push(trimmed.to_string());
            } else {
                // Other lines - ensure proper formatting
                formatted_lines.push(trimmed.to_string());
            }
        }

        formatted_lines.join("\n")
    }

    #[allow(deprecated)]
    pub fn extract_symbols(&self, text: &str) -> Vec<DocumentSymbol> {
        let document = self.parse(text);
        let mut symbols = Vec::new();

        for section in document.sections {
            let mut children = Vec::new();

            match section.name.as_str() {
                "Script Info" => {
                    for key in document.script_info.keys() {
                        children.push(DocumentSymbol {
                            name: key.clone(),
                            detail: None,
                            kind: SymbolKind::PROPERTY,
                            tags: None,
                            deprecated: None,
                            range: section.range,
                            selection_range: section.range,
                            children: None,
                        });
                    }
                }
                name if name.contains("Styles") => {
                    for style in &document.styles {
                        children.push(DocumentSymbol {
                            name: style.name.clone(),
                            detail: Some(format!("{} {}", style.fontname, style.fontsize)),
                            kind: SymbolKind::CLASS,
                            tags: None,
                            deprecated: None,
                            range: style.range,
                            selection_range: style.range,
                            children: None,
                        });
                    }
                }
                "Events" => {
                    for event in &document.events {
                        children.push(DocumentSymbol {
                            name: if event.actor.is_empty() {
                                format!("{} - {}", event.start_time, event.end_time)
                            } else {
                                format!(
                                    "{}: {} - {}",
                                    event.actor, event.start_time, event.end_time
                                )
                            },
                            detail: Some(event.text.chars().take(50).collect::<String>()),
                            kind: if event.event_type == "Dialogue" {
                                SymbolKind::FUNCTION
                            } else {
                                SymbolKind::VARIABLE
                            },
                            tags: None,
                            deprecated: None,
                            range: event.range,
                            selection_range: event.range,
                            children: None,
                        });
                    }
                }
                _ => {}
            }

            symbols.push(DocumentSymbol {
                name: section.name,
                detail: Some(format!("{} items", children.len())),
                kind: SymbolKind::NAMESPACE,
                tags: None,
                deprecated: None,
                range: section.range,
                selection_range: section.range,
                children: if children.is_empty() {
                    None
                } else {
                    Some(children)
                },
            });
        }

        symbols
    }
}

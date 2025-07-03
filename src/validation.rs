use crate::parser::{AssDocument, Event, Style};
use regex::Regex;
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct ValidationProvider {
    time_regex: Regex,
    color_regex: Regex,
}

impl ValidationProvider {
    pub fn new() -> Self {
        Self {
            time_regex: Regex::new(r"^\d{1,2}:\d{2}:\d{2}\.\d{2}$").unwrap(),
            color_regex: Regex::new(r"^&H[0-9A-Fa-f]{6,8}$|^\d+$").unwrap(),
        }
    }

    pub fn validate(&self, document: &AssDocument) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Validate required sections
        diagnostics.extend(self.validate_required_sections(document));

        // Validate styles
        for style in &document.styles {
            diagnostics.extend(self.validate_style(style));
        }

        // Validate events
        for event in &document.events {
            diagnostics.extend(self.validate_event(event));
        }

        // Check for style references
        diagnostics.extend(self.validate_style_references(document));

        diagnostics
    }

    fn validate_required_sections(&self, document: &AssDocument) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let required_sections = ["Script Info", "Events"];
        let section_names: Vec<&str> = document.sections.iter().map(|s| s.name.as_str()).collect();

        for required in &required_sections {
            if !section_names.iter().any(|&name| name.contains(required)) {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 0),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("missing_section".to_string())),
                    code_description: None,
                    source: Some("ass-lsp".to_string()),
                    message: format!("Missing required section: [{required}]"),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        diagnostics
    }

    fn validate_style(&self, style: &Style) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Validate style name
        if style.name.is_empty() {
            diagnostics.push(Diagnostic {
                range: style.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("empty_style_name".to_string())),
                code_description: None,
                source: Some("ass-lsp".to_string()),
                message: "Style name cannot be empty".to_string(),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        // Validate font size
        if style.fontsize == 0 {
            diagnostics.push(Diagnostic {
                range: style.range,
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("zero_font_size".to_string())),
                code_description: None,
                source: Some("ass-lsp".to_string()),
                message: "Font size should not be zero".to_string(),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        // Validate colors
        if !self.color_regex.is_match(&style.primary_colour) {
            diagnostics.push(Diagnostic {
                range: style.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("invalid_color".to_string())),
                code_description: None,
                source: Some("ass-lsp".to_string()),
                message: format!("Invalid color format: {}", style.primary_colour),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        diagnostics
    }

    fn validate_event(&self, event: &Event) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Validate time format
        if !self.time_regex.is_match(&event.start_time) {
            diagnostics.push(Diagnostic {
                range: event.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("invalid_time_format".to_string())),
                code_description: None,
                source: Some("ass-lsp".to_string()),
                message: format!(
                    "Invalid time format: {} (expected H:MM:SS.CC)",
                    event.start_time
                ),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        if !self.time_regex.is_match(&event.end_time) {
            diagnostics.push(Diagnostic {
                range: event.range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("invalid_time_format".to_string())),
                code_description: None,
                source: Some("ass-lsp".to_string()),
                message: format!(
                    "Invalid time format: {} (expected H:MM:SS.CC)",
                    event.end_time
                ),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        // Validate time order
        if self.parse_time(&event.start_time) >= self.parse_time(&event.end_time) {
            diagnostics.push(Diagnostic {
                range: event.range,
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("invalid_time_order".to_string())),
                code_description: None,
                source: Some("ass-lsp".to_string()),
                message: "Start time should be before end time".to_string(),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        // Validate override tags in dialogue text
        diagnostics.extend(self.validate_override_tags(&event.text, event.range));

        diagnostics
    }

    fn validate_override_tags(&self, text: &str, range: Range) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut brace_count = 0;
        let mut _in_override = false;

        for (i, ch) in text.chars().enumerate() {
            match ch {
                '{' => {
                    brace_count += 1;
                    _in_override = true;
                }
                '}' => {
                    if brace_count == 0 {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position::new(
                                    range.start.line,
                                    range.start.character + i as u32,
                                ),
                                end: Position::new(
                                    range.start.line,
                                    range.start.character + i as u32 + 1,
                                ),
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("unmatched_brace".to_string())),
                            code_description: None,
                            source: Some("ass-lsp".to_string()),
                            message: "Unmatched closing brace".to_string(),
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                    } else {
                        brace_count -= 1;
                        _in_override = false;
                    }
                }
                _ => {}
            }
        }

        if brace_count > 0 {
            diagnostics.push(Diagnostic {
                range,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("unclosed_override".to_string())),
                code_description: None,
                source: Some("ass-lsp".to_string()),
                message: "Unclosed override tag".to_string(),
                related_information: None,
                tags: None,
                data: None,
            });
        }

        diagnostics
    }

    fn validate_style_references(&self, document: &AssDocument) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let style_names: Vec<&str> = document.styles.iter().map(|s| s.name.as_str()).collect();

        for event in &document.events {
            if !style_names.contains(&event.style.as_str()) && event.style != "Default" {
                diagnostics.push(Diagnostic {
                    range: event.range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("undefined_style".to_string())),
                    code_description: None,
                    source: Some("ass-lsp".to_string()),
                    message: format!("Reference to undefined style: {}", event.style),
                    related_information: None,
                    tags: None,
                    data: None,
                });
            }
        }

        diagnostics
    }

    fn parse_time(&self, time_str: &str) -> u32 {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return 0;
        }

        let hours: u32 = parts[0].parse().unwrap_or(0);
        let minutes: u32 = parts[1].parse().unwrap_or(0);
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        let seconds: u32 = seconds_parts[0].parse().unwrap_or(0);
        let centiseconds: u32 = if seconds_parts.len() > 1 {
            seconds_parts[1].parse().unwrap_or(0)
        } else {
            0
        };

        hours * 360000 + minutes * 6000 + seconds * 100 + centiseconds
    }
}

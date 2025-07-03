use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct CompletionProvider {
    override_tags: Vec<&'static str>,
    script_info_keys: Vec<&'static str>,
    style_fields: Vec<&'static str>,
    event_fields: Vec<&'static str>,
}

impl CompletionProvider {
    pub fn new() -> Self {
        Self {
            override_tags: vec![
                "\\pos", "\\move", "\\org", "\\clip", "\\iclip",
                "\\fscx", "\\fscy", "\\fsp", "\\frx", "\\fry", "\\frz", "\\fr",
                "\\fn", "\\fs", "\\fe", "\\b", "\\i", "\\u", "\\s",
                "\\bord", "\\xbord", "\\ybord", "\\shad", "\\xshad", "\\yshad",
                "\\c", "\\1c", "\\2c", "\\3c", "\\4c",
                "\\alpha", "\\1a", "\\2a", "\\3a", "\\4a",
                "\\an", "\\a", "\\q", "\\r", "\\t", "\\fad", "\\fade",
                "\\p", "\\pbo", "\\k", "\\K", "\\kf", "\\ko"
            ],
            script_info_keys: vec![
                "Title", "ScriptType", "WrapStyle", "PlayResX", "PlayResY", 
                "ScaledBorderAndShadow", "Video File", "Video Aspect Ratio",
                "Video Zoom", "Video Position", "Last Style Storage",
                "Audio File", "Video Zoom Percent", "Scroll Position",
                "Active Line", "Video Position"
            ],
            style_fields: vec![
                "Name", "Fontname", "Fontsize", "PrimaryColour", "SecondaryColour",
                "OutlineColour", "BackColour", "Bold", "Italic", "Underline",
                "StrikeOut", "ScaleX", "ScaleY", "Spacing", "Angle", "BorderStyle",
                "Outline", "Shadow", "Alignment", "MarginL", "MarginR", "MarginV",
                "Encoding"
            ],
            event_fields: vec![
                "Layer", "Start", "End", "Style", "Name", "MarginL", "MarginR",
                "MarginV", "Effect", "Text"
            ],
        }
    }

    pub fn provide_completions(&self, text: &str, position: Position) -> Vec<CompletionItem> {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;
        
        if line_idx >= lines.len() {
            return Vec::new();
        }

        let current_line = lines[line_idx];
        let char_idx = position.character as usize;
        let prefix = if char_idx <= current_line.len() {
            &current_line[..char_idx]
        } else {
            current_line
        };

        // Determine context
        let context = self.determine_context(text, position);

        match context {
            CompletionContext::OverrideTags => self.complete_override_tags(prefix),
            CompletionContext::ScriptInfo => self.complete_script_info(prefix),
            CompletionContext::StyleFormat => self.complete_style_format(prefix),
            CompletionContext::EventFormat => self.complete_event_format(prefix),
            CompletionContext::Section => self.complete_sections(prefix),
            CompletionContext::EventType => self.complete_event_types(prefix),
            _ => Vec::new(),
        }
    }

    fn determine_context(&self, text: &str, position: Position) -> CompletionContext {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;
        
        if line_idx >= lines.len() {
            return CompletionContext::None;
        }

        let current_line = lines[line_idx];
        
        // Check if we're in an override tag
        if current_line.contains('{') && !current_line[..position.character as usize].contains('}') {
            return CompletionContext::OverrideTags;
        }

        // Find current section
        let mut current_section = None;
        for i in (0..=line_idx).rev() {
            if lines[i].starts_with('[') && lines[i].ends_with(']') {
                current_section = Some(lines[i]);
                break;
            }
        }

        match current_section {
            Some("[Script Info]") => CompletionContext::ScriptInfo,
            Some(section) if section.contains("Styles") => {
                if current_line.starts_with("Format:") {
                    CompletionContext::StyleFormat
                } else {
                    CompletionContext::None
                }
            }
            Some("[Events]") => {
                if current_line.starts_with("Format:") {
                    CompletionContext::EventFormat
                } else if current_line.is_empty() || current_line.ends_with(':') {
                    CompletionContext::EventType
                } else {
                    CompletionContext::None
                }
            }
            None => {
                if current_line.starts_with('[') || current_line.is_empty() {
                    CompletionContext::Section
                } else {
                    CompletionContext::None
                }
            }
            _ => CompletionContext::None,
        }
    }

    fn complete_override_tags(&self, prefix: &str) -> Vec<CompletionItem> {
        let last_backslash = prefix.rfind('\\').unwrap_or(0);
        let tag_prefix = &prefix[last_backslash..];

        self.override_tags
            .iter()
            .filter(|tag| tag.starts_with(tag_prefix))
            .map(|tag| CompletionItem {
                label: tag.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(self.get_tag_description(tag)),
                documentation: Some(Documentation::String(self.get_tag_documentation(tag))),
                insert_text: Some(self.get_tag_insert_text(tag)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect()
    }

    fn complete_script_info(&self, prefix: &str) -> Vec<CompletionItem> {
        let key_prefix = if let Some(_colon_pos) = prefix.rfind(':') {
            // If there's already a colon, don't suggest keys
            return Vec::new();
        } else {
            prefix.trim()
        };

        self.script_info_keys
            .iter()
            .filter(|key| key.to_lowercase().starts_with(&key_prefix.to_lowercase()))
            .map(|key| CompletionItem {
                label: key.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Script Info Property".to_string()),
                insert_text: Some(format!("{}: $0", key)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect()
    }

    fn complete_style_format(&self, _prefix: &str) -> Vec<CompletionItem> {
        self.style_fields
            .iter()
            .map(|field| CompletionItem {
                label: field.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("Style Field".to_string()),
                ..Default::default()
            })
            .collect()
    }

    fn complete_event_format(&self, _prefix: &str) -> Vec<CompletionItem> {
        self.event_fields
            .iter()
            .map(|field| CompletionItem {
                label: field.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("Event Field".to_string()),
                ..Default::default()
            })
            .collect()
    }

    fn complete_sections(&self, _prefix: &str) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "[Script Info]".to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Script metadata and properties".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "[V4+ Styles]".to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Style definitions".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "[Events]".to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Dialogue and timing events".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "[Fonts]".to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Embedded fonts".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "[Graphics]".to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some("Embedded graphics".to_string()),
                ..Default::default()
            },
        ]
    }

    fn complete_event_types(&self, _prefix: &str) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "Dialogue:".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Dialogue event".to_string()),
                insert_text: Some("Dialogue: 0,${1:0:00:00.00},${2:0:00:05.00},${3:Default},,0,0,0,,${4:Text}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "Comment:".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Comment event".to_string()),
                insert_text: Some("Comment: 0,${1:0:00:00.00},${2:0:00:05.00},${3:Default},,0,0,0,,${4:Comment}".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    fn get_tag_description(&self, tag: &str) -> String {
        match tag {
            "\\pos" => "Position override".to_string(),
            "\\move" => "Movement animation".to_string(),
            "\\c" | "\\1c" => "Primary color".to_string(),
            "\\2c" => "Secondary color".to_string(),
            "\\3c" => "Outline color".to_string(),
            "\\4c" => "Shadow color".to_string(),
            "\\b" => "Bold".to_string(),
            "\\i" => "Italic".to_string(),
            "\\u" => "Underline".to_string(),
            "\\s" => "Strikeout".to_string(),
            "\\fn" => "Font name".to_string(),
            "\\fs" => "Font size".to_string(),
            "\\an" => "Alignment (numpad)".to_string(),
            "\\k" => "Karaoke".to_string(),
            "\\t" => "Transform/animation".to_string(),
            "\\fad" => "Simple fade".to_string(),
            "\\fade" => "Complex fade".to_string(),
            _ => "ASS override tag".to_string(),
        }
    }

    fn get_tag_documentation(&self, tag: &str) -> String {
        match tag {
            "\\pos" => "\\pos(x,y) - Sets the position of the subtitle".to_string(),
            "\\move" => "\\move(x1,y1,x2,y2,t1,t2) - Moves subtitle from (x1,y1) to (x2,y2)".to_string(),
            "\\c" | "\\1c" => "\\c&Hbbggrr& - Sets the primary text color".to_string(),
            "\\b" => "\\b1 or \\b0 - Enable or disable bold formatting".to_string(),
            "\\i" => "\\i1 or \\i0 - Enable or disable italic formatting".to_string(),
            "\\fn" => "\\fnFontName - Changes the font name".to_string(),
            "\\fs" => "\\fsSize - Changes the font size".to_string(),
            "\\an" => "\\anN - Sets alignment using numpad notation (1-9)".to_string(),
            "\\k" => "\\kDuration - Karaoke timing in centiseconds".to_string(),
            "\\t" => "\\t(tags) or \\t(t1,t2,tags) - Transforms tags over time".to_string(),
            _ => format!("{} - ASS override tag", tag),
        }
    }

    fn get_tag_insert_text(&self, tag: &str) -> String {
        match tag {
            "\\pos" => "\\pos(${1:x},${2:y})".to_string(),
            "\\move" => "\\move(${1:x1},${2:y1},${3:x2},${4:y2})".to_string(),
            "\\c" | "\\1c" | "\\2c" | "\\3c" | "\\4c" => format!("{}${{1:&Hffffff&}}", tag),
            "\\fn" => "\\fn${1:Arial}".to_string(),
            "\\fs" => "\\fs${1:20}".to_string(),
            "\\b" | "\\i" | "\\u" | "\\s" => format!("{}${{1:1}}", tag),
            "\\an" => "\\an${1:2}".to_string(),
            "\\k" => "\\k${1:100}".to_string(),
            "\\t" => "\\t(${1:tags})".to_string(),
            "\\fad" => "\\fad(${1:100},${2:100})".to_string(),
            "\\fade" => "\\fade(${1:255},${2:0},${3:255},${4:0},${5:500},${6:1000},${7:1500})".to_string(),
            _ => tag.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum CompletionContext {
    None,
    OverrideTags,
    ScriptInfo,
    StyleFormat,
    EventFormat,
    Section,
    EventType,
}
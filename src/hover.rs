use regex::Regex;
use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub struct HoverProvider {
    time_regex: Regex,
    color_regex: Regex,
}

impl HoverProvider {
    pub fn new() -> Self {
        Self {
            time_regex: Regex::new(r"\d{1,2}:\d{2}:\d{2}\.\d{2}").unwrap(),
            color_regex: Regex::new(r"&H[0-9A-Fa-f]{6,8}").unwrap(),
        }
    }

    pub fn provide_hover(&self, text: &str, position: Position) -> Option<Hover> {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;

        if line_idx >= lines.len() {
            return None;
        }

        let current_line = lines[line_idx];
        let char_idx = position.character as usize;

        // Find the word or token at the cursor position
        let token = self.get_token_at_position(current_line, char_idx)?;

        // Determine what kind of token this is and provide appropriate hover info
        self.get_hover_content(&token, current_line)
            .map(|hover_content| Hover {
                contents: HoverContents::Scalar(MarkedString::String(hover_content)),
                range: Some(Range {
                    start: Position::new(position.line, (char_idx - token.len()) as u32),
                    end: Position::new(position.line, char_idx as u32),
                }),
            })
    }

    fn get_token_at_position(&self, line: &str, char_idx: usize) -> Option<String> {
        if char_idx > line.len() {
            return None;
        }

        // Find word boundaries
        let start = line[..char_idx]
            .rfind(|c: char| c.is_whitespace() || c == ',' || c == ':' || c == '{' || c == '}')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line[char_idx..]
            .find(|c: char| c.is_whitespace() || c == ',' || c == ':' || c == '{' || c == '}')
            .map(|i| char_idx + i)
            .unwrap_or(line.len());

        if start < end {
            Some(line[start..end].to_string())
        } else {
            None
        }
    }

    fn get_hover_content(&self, token: &str, line: &str) -> Option<String> {
        // Check for ASS override tags
        if token.starts_with('\\') {
            return self.get_override_tag_info(token);
        }

        // Check for time values
        if self.time_regex.is_match(token) {
            return self.get_time_info(token);
        }

        // Check for color values
        if self.color_regex.is_match(token) {
            return self.get_color_info(token);
        }

        // Check for section headers
        if token.starts_with('[') && token.ends_with(']') {
            return self.get_section_info(token);
        }

        // Check for script info keys
        if line.contains(':') && !line.starts_with("Dialogue:") && !line.starts_with("Comment:") {
            return self.get_script_info_key_info(token);
        }

        // Check for event types
        if token == "Dialogue" || token == "Comment" {
            return self.get_event_type_info(token);
        }

        None
    }

    fn get_override_tag_info(&self, tag: &str) -> Option<String> {
        let tag_name = if tag.contains('(') {
            tag.split('(').next().unwrap_or(tag)
        } else {
            tag
        };

        match tag_name {
            "\\pos" => Some("**Position Override**\n\n`\\pos(x,y)`\n\nSets the subtitle position in pixels from the top-left corner of the video.".to_string()),
            "\\move" => Some("**Movement Animation**\n\n`\\move(x1,y1,x2,y2[,t1,t2])`\n\nMoves the subtitle from position (x1,y1) to (x2,y2). Optional t1,t2 specify start/end times.".to_string()),
            "\\org" => Some("**Origin Override**\n\n`\\org(x,y)`\n\nSets the origin point for rotations and scaling transformations.".to_string()),
            "\\clip" => Some("**Clipping**\n\n`\\clip(x1,y1,x2,y2)` or `\\clip(drawing)`\n\nLimits the subtitle to only appear within the specified rectangular area or drawing shape.".to_string()),
            "\\c" | "\\1c" => Some("**Primary Color**\n\n`\\c&Hbbggrr&` or `\\1c&Hbbggrr&`\n\nSets the primary text color in BGR (Blue-Green-Red) hexadecimal format.".to_string()),
            "\\2c" => Some("**Secondary Color**\n\n`\\2c&Hbbggrr&`\n\nSets the secondary text color (used for karaoke highlighting).".to_string()),
            "\\3c" => Some("**Outline Color**\n\n`\\3c&Hbbggrr&`\n\nSets the color of the text outline/border.".to_string()),
            "\\4c" => Some("**Shadow Color**\n\n`\\4c&Hbbggrr&`\n\nSets the color of the text shadow.".to_string()),
            "\\alpha" => Some("**Alpha Transparency**\n\n`\\alpha&Haa&`\n\nSets the overall transparency. 00 = opaque, FF = transparent.".to_string()),
            "\\1a" => Some("**Primary Alpha**\n\n`\\1a&Haa&`\n\nSets the transparency of the primary text color.".to_string()),
            "\\2a" => Some("**Secondary Alpha**\n\n`\\2a&Haa&`\n\nSets the transparency of the secondary text color.".to_string()),
            "\\3a" => Some("**Outline Alpha**\n\n`\\3a&Haa&`\n\nSets the transparency of the text outline.".to_string()),
            "\\4a" => Some("**Shadow Alpha**\n\n`\\4a&Haa&`\n\nSets the transparency of the text shadow.".to_string()),
            "\\b" => Some("**Bold**\n\n`\\b1` or `\\b0` or `\\b<weight>`\n\nEnables (1) or disables (0) bold formatting, or sets specific font weight.".to_string()),
            "\\i" => Some("**Italic**\n\n`\\i1` or `\\i0`\n\nEnables (1) or disables (0) italic formatting.".to_string()),
            "\\u" => Some("**Underline**\n\n`\\u1` or `\\u0`\n\nEnables (1) or disables (0) underline formatting.".to_string()),
            "\\s" => Some("**Strikeout**\n\n`\\s1` or `\\s0`\n\nEnables (1) or disables (0) strikethrough formatting.".to_string()),
            "\\fn" => Some("**Font Name**\n\n`\\fn<fontname>`\n\nChanges the font family. Use font names installed on the system.".to_string()),
            "\\fs" => Some("**Font Size**\n\n`\\fs<size>`\n\nChanges the font size in points.".to_string()),
            "\\fscx" => Some("**Font Scale X**\n\n`\\fscx<percent>`\n\nScales the font horizontally. 100 = normal, 200 = double width.".to_string()),
            "\\fscy" => Some("**Font Scale Y**\n\n`\\fscy<percent>`\n\nScales the font vertically. 100 = normal, 200 = double height.".to_string()),
            "\\fsp" => Some("**Font Spacing**\n\n`\\fsp<pixels>`\n\nAdjusts character spacing. Positive values increase spacing.".to_string()),
            "\\frx" => Some("**Rotation X**\n\n`\\frx<degrees>`\n\nRotates text around the X-axis (pitch).".to_string()),
            "\\fry" => Some("**Rotation Y**\n\n`\\fry<degrees>`\n\nRotates text around the Y-axis (yaw).".to_string()),
            "\\frz" | "\\fr" => Some("**Rotation Z**\n\n`\\frz<degrees>` or `\\fr<degrees>`\n\nRotates text around the Z-axis (roll). Positive values rotate counter-clockwise.".to_string()),
            "\\bord" => Some("**Border**\n\n`\\bord<width>`\n\nSets the width of the text outline/border.".to_string()),
            "\\shad" => Some("**Shadow**\n\n`\\shad<depth>`\n\nSets the depth of the text shadow.".to_string()),
            "\\an" => Some("**Alignment (Numpad)**\n\n`\\an<1-9>`\n\nSets text alignment using numpad layout:\n1=bottom-left, 2=bottom-center, 3=bottom-right\n4=middle-left, 5=middle-center, 6=middle-right\n7=top-left, 8=top-center, 9=top-right".to_string()),
            "\\a" => Some("**Alignment (Legacy)**\n\n`\\a<1-11>`\n\nLegacy alignment system. Use \\an instead for new scripts.".to_string()),
            "\\k" => Some("**Karaoke**\n\n`\\k<duration>`\n\nKaraoke timing in centiseconds. Text will be highlighted for the specified duration.".to_string()),
            "\\K" => Some("**Karaoke (Fill)**\n\n`\\K<duration>`\n\nSweeping karaoke effect that fills the text over the specified duration.".to_string()),
            "\\kf" => Some("**Karaoke (Fill)**\n\n`\\kf<duration>`\n\nAlias for \\K. Sweeping karaoke effect.".to_string()),
            "\\ko" => Some("**Karaoke (Outline)**\n\n`\\ko<duration>`\n\nKaraoke effect that sweeps the outline color.".to_string()),
            "\\t" => Some("**Transform**\n\n`\\t([t1,t2,][accel,]tags)`\n\nAnimates the specified tags over time. Optional t1,t2 specify start/end times, accel controls acceleration.".to_string()),
            "\\fad" => Some("**Simple Fade**\n\n`\\fad(fadein,fadeout)`\n\nSimple fade in and fade out effect. Times in milliseconds.".to_string()),
            "\\fade" => Some("**Complex Fade**\n\n`\\fade(a1,a2,a3,t1,t2,t3,t4)`\n\nComplex fade with multiple alpha values and timing points.".to_string()),
            "\\p" => Some("**Drawing Mode**\n\n`\\p<scale>`\n\nEnables drawing mode for vector graphics. Scale factor for coordinates.".to_string()),
            "\\pbo" => Some("**Drawing Baseline Offset**\n\n`\\pbo<offset>`\n\nVertical offset for drawing coordinates.".to_string()),
            "\\q" => Some("**Wrap Style**\n\n`\\q<0-3>`\n\nText wrapping style:\n0=smart wrap, 1=end-of-line wrap, 2=no wrap, 3=smart wrap with lower line wider".to_string()),
            "\\r" => Some("**Reset**\n\n`\\r[style]`\n\nResets all override tags to the style defaults. Optional style name.".to_string()),
            _ => Some(format!("**ASS Override Tag**\n\n`{}`\n\nAdvanced SubStation Alpha formatting tag.", tag)),
        }
    }

    fn get_time_info(&self, time: &str) -> Option<String> {
        // Parse the time and provide duration info
        let parts: Vec<&str> = time.split(':').collect();
        if parts.len() == 3 {
            if let (Ok(hours), Ok(minutes)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                let sec_parts: Vec<&str> = parts[2].split('.').collect();
                if sec_parts.len() == 2 {
                    if let (Ok(seconds), Ok(centiseconds)) =
                        (sec_parts[0].parse::<u32>(), sec_parts[1].parse::<u32>())
                    {
                        let total_ms =
                            hours * 3600000 + minutes * 60000 + seconds * 1000 + centiseconds * 10;
                        return Some(format!(
                            "**Timestamp**\n\n`{}`\n\nTotal duration: {}ms\n{}h {}m {}s {}cs",
                            time, total_ms, hours, minutes, seconds, centiseconds
                        ));
                    }
                }
            }
        }
        Some(format!("**Timestamp**\n\n`{}`\n\nFormat: H:MM:SS.CC", time))
    }

    fn get_color_info(&self, color: &str) -> Option<String> {
        if color.starts_with("&H") && color.len() >= 8 {
            let hex = &color[2..];
            if hex.len() >= 6 {
                // BGR format
                let b = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let r = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let alpha = if hex.len() >= 8 {
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    format!("\nAlpha: {} ({}%)", a, (255 - a) * 100 / 255)
                } else {
                    String::new()
                };

                return Some(format!(
                    "**Color Value**\n\n`{}`\n\nRGB: ({}, {}, {}){}\nBGR Format (Blue-Green-Red)",
                    color, r, g, b, alpha
                ));
            }
        }
        Some(format!(
            "**Color Value**\n\n`{}`\n\nASS color in BGR hexadecimal format",
            color
        ))
    }

    fn get_section_info(&self, section: &str) -> Option<String> {
        match section {
            "[Script Info]" => Some("**Script Info Section**\n\nContains metadata about the subtitle script including title, resolution, and playback settings.".to_string()),
            "[V4 Styles]" | "[V4+ Styles]" | "[v4 Styles]" | "[v4+ Styles]" => Some("**Styles Section**\n\nDefines the visual appearance of subtitle text including fonts, colors, positioning, and effects.".to_string()),
            "[Events]" => Some("**Events Section**\n\nContains the actual subtitle dialogue, comments, and timing information.".to_string()),
            "[Fonts]" => Some("**Fonts Section**\n\nOptional section for embedding font files directly in the subtitle script.".to_string()),
            "[Graphics]" => Some("**Graphics Section**\n\nOptional section for embedding image files directly in the subtitle script.".to_string()),
            _ => Some(format!("**Section Header**\n\n`{}`\n\nCustom section in the ASS script.", section)),
        }
    }

    fn get_script_info_key_info(&self, key: &str) -> Option<String> {
        match key {
            "Title" => Some("**Title**\n\nThe title of the subtitle script.".to_string()),
            "ScriptType" => Some("**Script Type**\n\nSpecifies the script format version (usually 'v4.00+').".to_string()),
            "WrapStyle" => Some("**Wrap Style**\n\nDefault text wrapping behavior:\n0=smart wrap, 1=end-of-line wrap, 2=no wrap, 3=smart wrap (lower line wider)".to_string()),
            "PlayResX" => Some("**Play Resolution X**\n\nHorizontal resolution for subtitle positioning and scaling.".to_string()),
            "PlayResY" => Some("**Play Resolution Y**\n\nVertical resolution for subtitle positioning and scaling.".to_string()),
            "ScaledBorderAndShadow" => Some("**Scaled Border and Shadow**\n\nWhether borders and shadows scale with video resolution (yes/no).".to_string()),
            "Video File" => Some("**Video File**\n\nPath to the associated video file.".to_string()),
            "Audio File" => Some("**Audio File**\n\nPath to the associated audio file.".to_string()),
            _ => Some(format!("**Script Info Property**\n\n`{}`\n\nScript metadata property.", key)),
        }
    }

    fn get_event_type_info(&self, event_type: &str) -> Option<String> {
        match event_type {
            "Dialogue" => Some("**Dialogue Event**\n\nA subtitle line that will be displayed during playback.".to_string()),
            "Comment" => Some("**Comment Event**\n\nA comment line that will not be displayed during playback. Used for notes and disabled subtitles.".to_string()),
            _ => None,
        }
    }
}

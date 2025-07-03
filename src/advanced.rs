use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub parse_time: Duration,
    pub validation_time: Duration,
    pub completion_time: Duration,
    pub total_time: Duration,
    pub file_size: usize,
    pub lines_count: usize,
}

#[derive(Debug, Clone)]
pub struct StyleInheritance {
    pub name: String,
    pub parent: Option<String>,
    pub properties: HashMap<String, String>,
}

#[derive(Debug)]
pub struct TimingOverlap {
    pub line1: usize,
    pub line2: usize,
    pub start_time: String,
    pub end_time: String,
    pub overlap_duration: Duration,
}

static PERFORMANCE_CACHE: Lazy<DashMap<String, PerformanceMetrics>> = Lazy::new(DashMap::new);
static STYLE_CACHE: Lazy<Arc<Mutex<HashMap<String, StyleInheritance>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub struct AdvancedFeatures {
    file_path: String,
    styles: HashMap<String, StyleInheritance>,
    timing_overlaps: Vec<TimingOverlap>,
}

impl AdvancedFeatures {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            styles: HashMap::new(),
            timing_overlaps: Vec::new(),
        }
    }

    pub fn analyze_style_inheritance(&mut self, content: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        self.styles.clear();

        let lines: Vec<&str> = content.lines().collect();
        let mut in_styles_section = false;

        for line in lines {
            let trimmed = line.trim();

            // Check for styles section
            if trimmed.starts_with("[V4") && trimmed.contains("Styles]") {
                in_styles_section = true;
                continue;
            } else if trimmed.starts_with("[") && trimmed.ends_with("]") {
                in_styles_section = false;
                continue;
            }

            if in_styles_section && trimmed.starts_with("Style:") {
                if let Some(style) = self.parse_style_line(trimmed) {
                    self.styles.insert(style.name.clone(), style);
                }
            }
        }

        // Check for circular references and unused properties
        for (name, style) in &self.styles {
            if let Some(parent) = &style.parent {
                if self.has_circular_reference(name, parent, &mut Vec::new()) {
                    warnings.push(format!("Circular style inheritance detected: {name}"));
                }
            }

            // Check for unused properties
            if style.properties.is_empty() {
                warnings.push(format!("Style '{name}' has no properties defined"));
            }
        }

        // Cache styles for future use
        if let Ok(mut cache) = STYLE_CACHE.lock() {
            cache.clear();
            cache.extend(self.styles.clone());
        }

        warnings
    }

    fn parse_style_line(&self, line: &str) -> Option<StyleInheritance> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            return None;
        }

        let name = parts[0].replace("Style:", "").trim().to_string();
        let mut properties = HashMap::new();

        // Parse style properties (simplified for demonstration)
        for (i, part) in parts.iter().enumerate().skip(1) {
            properties.insert(format!("prop_{i}"), part.trim().to_string());
        }

        Some(StyleInheritance {
            name,
            parent: None, // Would need format specification to determine parent
            properties,
        })
    }

    fn has_circular_reference(
        &self,
        current: &str,
        target: &str,
        visited: &mut Vec<String>,
    ) -> bool {
        if visited.contains(&current.to_string()) {
            return current == target;
        }

        visited.push(current.to_string());

        if let Some(style) = self.styles.get(current) {
            if let Some(parent) = &style.parent {
                return self.has_circular_reference(parent, target, visited);
            }
        }

        false
    }

    pub fn detect_timing_overlaps(&mut self, content: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        self.timing_overlaps.clear();

        let mut dialogue_lines = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut in_events_section = false;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("[Events]") {
                in_events_section = true;
                continue;
            } else if trimmed.starts_with("[") && trimmed.ends_with("]") {
                in_events_section = false;
                continue;
            }

            if in_events_section && trimmed.starts_with("Dialogue:") {
                if let Some(timing) = self.parse_dialogue_timing(trimmed, line_num) {
                    dialogue_lines.push(timing);
                }
            }
        }

        // Check for overlaps
        for i in 0..dialogue_lines.len() {
            for j in i + 1..dialogue_lines.len() {
                if let Some(overlap) =
                    self.check_timing_overlap(&dialogue_lines[i], &dialogue_lines[j])
                {
                    warnings.push(format!(
                        "Timing overlap detected between lines {} and {} (duration: {}ms, start: {}, end: {})",
                        overlap.line1 + 1,
                        overlap.line2 + 1,
                        overlap.overlap_duration.as_millis(),
                        overlap.start_time,
                        overlap.end_time
                    ));
                    self.timing_overlaps.push(overlap);
                }
            }
        }

        warnings
    }

    fn parse_dialogue_timing(
        &self,
        line: &str,
        line_num: usize,
    ) -> Option<((String, String), usize)> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 3 {
            return None;
        }

        let start_time = parts[1].trim().to_string();
        let end_time = parts[2].trim().to_string();

        Some(((start_time, end_time), line_num))
    }

    fn check_timing_overlap(
        &self,
        line1: &((String, String), usize),
        line2: &((String, String), usize),
    ) -> Option<TimingOverlap> {
        let ((start1, end1), line_num1) = line1;
        let ((start2, end2), line_num2) = line2;

        // Simplified overlap detection (would need proper time parsing)
        if start1 < end2 && start2 < end1 {
            // Calculate overlap duration (simplified)
            let overlap_duration = if start1 > start2 {
                Duration::from_millis(100) // Placeholder - would calculate actual overlap
            } else {
                Duration::from_millis(50) // Placeholder - would calculate actual overlap
            };

            Some(TimingOverlap {
                line1: *line_num1,
                line2: *line_num2,
                start_time: start1.clone(),
                end_time: end1.clone(),
                overlap_duration,
            })
        } else {
            None
        }
    }

    pub fn record_performance_metrics(&self, metrics: PerformanceMetrics) {
        PERFORMANCE_CACHE.insert(self.file_path.clone(), metrics);
    }

    pub fn get_performance_suggestions(&self) -> Vec<String> {
        let mut suggestions = Vec::new();

        if let Some(metrics) = PERFORMANCE_CACHE.get(&self.file_path) {
            if metrics.parse_time > Duration::from_millis(100) {
                suggestions.push("Consider breaking large files into smaller sections".to_string());
            }

            if metrics.validation_time > Duration::from_millis(50) {
                suggestions.push("File contains complex validation patterns".to_string());
            }

            if metrics.completion_time > Duration::from_millis(200) {
                suggestions.push("Code completion is slow - consider caching".to_string());
            }

            if metrics.total_time > Duration::from_secs(1) {
                suggestions.push("Total processing time is high - optimize workflow".to_string());
            }

            if metrics.file_size > 1024 * 1024 {
                suggestions.push("Large file detected - consider optimization".to_string());
            }

            if metrics.lines_count > 10000 {
                suggestions
                    .push("Many lines detected - indexing may improve performance".to_string());
            }
        }

        suggestions
    }

    pub fn validate_advanced(&self, content: &str) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for common ASS issues
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check for malformed override tags
            if trimmed.contains('{') && !trimmed.contains('}') {
                warnings.push(format!("Line {}: Unclosed override tag", line_num + 1));
            }

            // Check for invalid escape sequences
            if trimmed.contains("\\\\") && !trimmed.contains("\\N") && !trimmed.contains("\\n") {
                warnings.push(format!(
                    "Line {}: Potentially invalid escape sequence",
                    line_num + 1
                ));
            }

            // Check for extremely long lines that might cause rendering issues
            if trimmed.len() > 500 {
                warnings.push(format!(
                    "Line {}: Very long line may cause rendering issues",
                    line_num + 1
                ));
            }
        }

        warnings
    }

    pub fn get_timing_summary(&self) -> String {
        if self.timing_overlaps.is_empty() {
            return "No timing overlaps detected".to_string();
        }

        let mut summary = format!("Found {} timing overlaps:\n", self.timing_overlaps.len());
        for overlap in &self.timing_overlaps {
            summary.push_str(&format!(
                "- Lines {}-{}: {} to {} ({}ms overlap)\n",
                overlap.line1 + 1,
                overlap.line2 + 1,
                overlap.start_time,
                overlap.end_time,
                overlap.overlap_duration.as_millis()
            ));
        }
        summary
    }
}

#[allow(dead_code)]
pub fn clear_caches() {
    PERFORMANCE_CACHE.clear();
    if let Ok(mut cache) = STYLE_CACHE.lock() {
        cache.clear();
    }
}

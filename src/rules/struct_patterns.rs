use std::path::Path;
use syn::{visit::Visit, File, TypeReference, TypeSlice};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;
use crate::utils::{find_line_of_str_non_import, get_position};

pub struct ReferenceAbuseRule;
pub struct BoxAbuseRule;
pub struct SliceAbuseRule;

impl Rule for ReferenceAbuseRule {
    fn name(&self) -> &'static str {
        "reference-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }

        let mut visitor = ReferenceVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

impl Rule for BoxAbuseRule {
    fn name(&self) -> &'static str {
        "box-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }
        let mut visitor = BoxVisitor::new();
        visitor.visit_file(syntax_tree);

        // Check for Box usage in content since ExprBox doesn't exist in syn 2.0
        let box_count = content.matches("Box::new").count() + content.matches("Box<").count();
        if box_count > 8 {
            let messages = if lang == "zh-CN" {
                [
                    "Box 用得比快递还频繁",
                    "这么多 Box，你是在开仓库吗？",
                    "Box 过多，堆内存都要爆炸了",
                    "Box 滥用，建议考虑栈分配",
                    "这么多 Box，内存分配器都累了",
                ]
            } else {
                [
                    "You use Box more than Amazon uses cardboard",
                    "So many Boxes, are you running a warehouse?",
                    "Box overuse - the heap memory is about to file a restraining order",
                    "Box abuse detected - ever heard of stack allocation?",
                    "So many Boxes, even the memory allocator needs a vacation",
                ]
            };

            let candidates = [
                find_line_of_str_non_import(content, "Box::new"),
                find_line_of_str_non_import(content, "Box<"),
            ];
            let line = candidates
                .iter()
                .copied()
                .filter(|&l| l > 1)
                .min()
                .unwrap_or(1);
            visitor.issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line,
                column: 1,
                rule_name: "box-abuse".to_string(),
                message: messages[0].to_string(),
                severity: Severity::Spicy,
            });
        }

        visitor.issues
    }
}

impl Rule for SliceAbuseRule {
    fn name(&self) -> &'static str {
        "slice-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        lang: &str,
        is_test_file: bool,
    ) -> Vec<CodeIssue> {
        if is_test_file {
            return Vec::new();
        }
        let mut visitor = SliceVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

// Reference Visitor
struct ReferenceVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    reference_count: usize,
    lang: String,
}

impl ReferenceVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            reference_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for ReferenceVisitor {
    fn visit_type_reference(&mut self, ref_type: &'ast TypeReference) {
        // Skip &self and &mut self
        if let syn::Type::Path(type_path) = &*ref_type.elem {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Self" {
                    return;
                }
            }
        }

        self.reference_count += 1;

        // Cap to 1 issue per file
        if self.reference_count > 50 && self.issues.is_empty() {
            let messages = if self.lang == "zh-CN" {
                [
                    "引用比我的社交关系还复杂",
                    "这么多引用，你确定不是在写指针迷宫？",
                    "引用过多，小心借用检查器罢工",
                    "引用数量超标，建议重新设计数据结构",
                ]
            } else {
                [
                    "More references than my social circle has connections",
                    "So many references, are you writing a pointer maze?",
                    "Reference overuse - the borrow checker is about to go on strike",
                    "Too many references - time to rethink your data structure",
                ]
            };

            let (line, column) = get_position(ref_type);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "reference-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
            });
        }

        syn::visit::visit_type_reference(self, ref_type);
    }
}

// Box Visitor
struct BoxVisitor {
    issues: Vec<CodeIssue>,
}

impl BoxVisitor {
    fn new() -> Self {
        Self { issues: Vec::new() }
    }
}

impl<'ast> Visit<'ast> for BoxVisitor {
    // Box detection is handled in the rule implementation via content analysis
}

// Slice Visitor
struct SliceVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    slice_count: usize,
    lang: String,
}

impl SliceVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            slice_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for SliceVisitor {
    fn visit_type_slice(&mut self, slice_type: &'ast TypeSlice) {
        self.slice_count += 1;

        // Only report once per file when slice count is very high
        if self.slice_count == 30 {
            let messages = if self.lang == "zh-CN" {
                [
                    "切片比我切菜还频繁",
                    "这么多切片，你是在开水果店吗？",
                    "切片过多，数组都被你切碎了",
                    "Slice 滥用，建议使用 Vec",
                ]
            } else {
                [
                    "You slice more than a professional chef",
                    "So many slices, are you running a fruit stand?",
                    "Slice overuse - the array has been shredded to pieces",
                    "Slice abuse detected - have you considered using Vec?",
                ]
            };

            let (line, column) = get_position(slice_type);
            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line,
                column,
                rule_name: "slice-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
            });
        }

        syn::visit::visit_type_slice(self, slice_type);
    }
}

use std::path::Path;
use syn::{visit::Visit, File, TypeReference, TypeSlice};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;

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
    ) -> Vec<CodeIssue> {
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
    ) -> Vec<CodeIssue> {
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

            visitor.issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line: 1,
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
    ) -> Vec<CodeIssue> {
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
    fn visit_type_reference(&mut self, _ref_type: &'ast TypeReference) {
        self.reference_count += 1;

        if self.reference_count > 20 {
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

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "reference-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
            });
        }

        syn::visit::visit_type_reference(self, _ref_type);
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
    fn visit_type_slice(&mut self, _slice_type: &'ast TypeSlice) {
        self.slice_count += 1;

        if self.slice_count > 15 {
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

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "slice-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
            });
        }

        syn::visit::visit_type_slice(self, _slice_type);
    }
}

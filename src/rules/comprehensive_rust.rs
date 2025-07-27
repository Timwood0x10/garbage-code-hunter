use std::path::Path;
use syn::{
    visit::Visit, ExprAsync, ExprAwait, ExprMatch, ExprUnsafe, File, ForeignItem, ItemFn,
    ItemForeignMod, ItemMod, Macro, PatSlice, PatTuple, TypePath, TypeReference, TypeSlice,
    TypeTraitObject,
};

use crate::analyzer::{CodeIssue, RoastLevel, Severity};
use crate::rules::Rule;

pub struct ChannelAbuseRule;
pub struct AsyncAbuseRule;
pub struct DynTraitAbuseRule;
pub struct UnsafeAbuseRule;
pub struct FFIAbuseRule;
pub struct MacroAbuseRule;
pub struct ModuleComplexityRule;
pub struct PatternMatchingAbuseRule;
pub struct ReferenceAbuseRule;
pub struct BoxAbuseRule;
pub struct SliceAbuseRule;

impl Rule for ChannelAbuseRule {
    fn name(&self) -> &'static str {
        "channel-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = ChannelVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);

        // Also check for channel-related imports and usage in content
        if content.contains("std::sync::mpsc") || content.contains("tokio::sync") {
            visitor.channel_count += content.matches("channel").count();
            visitor.channel_count += content.matches("Sender").count();
            visitor.channel_count += content.matches("Receiver").count();
        }

        visitor.check_channel_overuse();
        visitor.issues
    }
}

impl Rule for AsyncAbuseRule {
    fn name(&self) -> &'static str {
        "async-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = AsyncVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

impl Rule for DynTraitAbuseRule {
    fn name(&self) -> &'static str {
        "dyn-trait-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = DynTraitVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

impl Rule for UnsafeAbuseRule {
    fn name(&self) -> &'static str {
        "unsafe-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = UnsafeVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);

        // 检查内容中的 unsafe 关键字使用
        visitor.check_unsafe_in_content(content);
        visitor.issues
    }
}

impl Rule for FFIAbuseRule {
    fn name(&self) -> &'static str {
        "ffi-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = FFIVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);

        // 检查内容中的 FFI 相关模式
        visitor.check_ffi_patterns_in_content(content);
        visitor.issues
    }
}

impl Rule for MacroAbuseRule {
    fn name(&self) -> &'static str {
        "macro-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = MacroVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

impl Rule for ModuleComplexityRule {
    fn name(&self) -> &'static str {
        "module-complexity"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = ModuleVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

impl Rule for PatternMatchingAbuseRule {
    fn name(&self) -> &'static str {
        "pattern-matching-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = PatternVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

impl Rule for ReferenceAbuseRule {
    fn name(&self) -> &'static str {
        "reference-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        _content: &str,
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = ReferenceVisitor::new(file_path.to_path_buf());
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
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = BoxVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);

        // Check for Box usage in content since ExprBox doesn't exist in syn 2.0
        let box_count = content.matches("Box::new").count() + content.matches("Box<").count();
        if box_count > 8 {
            let messages = [
                "Box 用得比快递还频繁",
                "这么多 Box，你是在开仓库吗？",
                "Box 过多，堆内存都要爆炸了",
                "Box 滥用，建议考虑栈分配",
                "这么多 Box，内存分配器都累了",
            ];

            visitor.issues.push(CodeIssue {
                file_path: file_path.to_path_buf(),
                line: 1,
                column: 1,
                rule_name: "box-abuse".to_string(),
                message: messages[0].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
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
        _lang: &str,
    ) -> Vec<CodeIssue> {
        let mut visitor = SliceVisitor::new(file_path.to_path_buf());
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

// Channel Visitor
struct ChannelVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    channel_count: usize,
}

impl ChannelVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            channel_count: 0,
        }
    }

    fn check_channel_overuse(&mut self) {
        if self.channel_count > 5 {
            let messages = [
                "Channel 用得比我发微信还频繁，你确定不是在写聊天软件？",
                "这么多 Channel，你是想开通讯公司吗？",
                "Channel 滥用！你的程序比电话交换机还复杂",
                "Channel 数量超标，建议重新设计架构",
                "这么多 Channel，我怀疑你在写分布式系统",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "channel-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }
    }
}

impl<'ast> Visit<'ast> for ChannelVisitor {
    fn visit_type_path(&mut self, type_path: &'ast TypePath) {
        let path_str = quote::quote!(#type_path).to_string();
        if path_str.contains("Sender")
            || path_str.contains("Receiver")
            || path_str.contains("channel")
        {
            self.channel_count += 1;
        }
        syn::visit::visit_type_path(self, type_path);
    }
}

// Async Visitor
struct AsyncVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    async_count: usize,
    await_count: usize,
}

impl AsyncVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            async_count: 0,
            await_count: 0,
        }
    }

    fn check_async_abuse(&mut self) {
        if self.async_count > 10 {
            let messages = [
                "Async 函数比我的异步人生还要复杂",
                "这么多 async，你确定不是在写 JavaScript？",
                "Async 滥用！建议学习一下同步编程的美好",
                "异步函数过多，小心把自己绕晕了",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "async-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        if self.await_count > 20 {
            let messages = [
                "Await 用得比我等外卖还频繁",
                "这么多 await，你的程序是在等什么？世界末日吗？",
                "Await 过度使用，建议批量处理",
                "等待次数过多，你的程序比我还有耐心",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "async-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }
    }
}

impl<'ast> Visit<'ast> for AsyncVisitor {
    fn visit_expr_async(&mut self, _async_expr: &'ast ExprAsync) {
        self.async_count += 1;
        self.check_async_abuse();
        syn::visit::visit_expr_async(self, _async_expr);
    }

    fn visit_expr_await(&mut self, _await_expr: &'ast ExprAwait) {
        self.await_count += 1;
        self.check_async_abuse();
        syn::visit::visit_expr_await(self, _await_expr);
    }
}

// Dyn Trait Visitor
struct DynTraitVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    dyn_count: usize,
}

impl DynTraitVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            dyn_count: 0,
        }
    }
}

impl<'ast> Visit<'ast> for DynTraitVisitor {
    fn visit_type_trait_object(&mut self, trait_object: &'ast TypeTraitObject) {
        self.dyn_count += 1;

        if self.dyn_count > 5 {
            let messages = [
                "Dyn trait 用得比我换工作还频繁",
                "这么多动态分发，性能都跑到哪里去了？",
                "Dyn trait 滥用，你确定不是在写 Python？",
                "动态 trait 过多，编译器优化都哭了",
                "这么多 dyn，你的程序比变色龙还善变",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "dyn-trait-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        syn::visit::visit_type_trait_object(self, trait_object);
    }
}

// Unsafe Visitor
struct UnsafeVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    unsafe_count: usize,
    unsafe_fn_count: usize,
    unsafe_impl_count: usize,
    unsafe_trait_count: usize,
}

impl UnsafeVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            unsafe_count: 0,
            unsafe_fn_count: 0,
            unsafe_impl_count: 0,
            unsafe_trait_count: 0,
        }
    }

    fn check_unsafe_in_content(&mut self, content: &str) {
        // 检查 unsafe 函数
        let unsafe_fn_matches = content.matches("unsafe fn").count();
        self.unsafe_fn_count += unsafe_fn_matches;

        // 检查 unsafe impl
        let unsafe_impl_matches = content.matches("unsafe impl").count();
        self.unsafe_impl_count += unsafe_impl_matches;

        // 检查 unsafe trait
        let unsafe_trait_matches = content.matches("unsafe trait").count();
        self.unsafe_trait_count += unsafe_trait_matches;

        // 检查原始指针操作
        let raw_ptr_count = content.matches("*const").count() + content.matches("*mut").count();

        // 检查内存操作函数
        let dangerous_ops = [
            "std::ptr::write",
            "std::ptr::read",
            "std::ptr::copy",
            "std::mem::transmute",
            "std::mem::forget",
            "std::mem::uninitialized",
            "std::slice::from_raw_parts",
            "std::str::from_utf8_unchecked",
            "Box::from_raw",
            "Vec::from_raw_parts",
            "String::from_raw_parts",
        ];

        let mut dangerous_op_count = 0;
        for op in &dangerous_ops {
            dangerous_op_count += content.matches(op).count();
        }

        self.generate_unsafe_issues(raw_ptr_count, dangerous_op_count);
    }

    fn generate_unsafe_issues(&mut self, raw_ptr_count: usize, dangerous_op_count: usize) {
        // 检查 unsafe 函数过多
        if self.unsafe_fn_count > 2 {
            let messages = [
                "Unsafe 函数比我的黑历史还多！你确定这还是 Rust 吗？",
                "这么多 unsafe 函数，Rust 的安全保证都被你玩坏了",
                "Unsafe 函数过多，建议重新考虑设计架构",
                "你的 unsafe 函数让 Rust 编译器都开始怀疑人生了",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "unsafe-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
                roast_level: RoastLevel::Savage,
            });
        }

        // 检查原始指针过多
        if raw_ptr_count > 5 {
            let messages = [
                "原始指针用得比我换手机还频繁，你这是在写 C 语言吗？",
                "这么多原始指针，内存安全已经不在服务区了",
                "原始指针过多，建议使用安全的 Rust 抽象",
                "你的指针操作让 Valgrind 都要加班了",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "unsafe-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
                roast_level: RoastLevel::Savage,
            });
        }

        // 检查危险操作过多
        if dangerous_op_count > 3 {
            let messages = [
                "危险的内存操作比我的危险驾驶还要多！",
                "这些危险操作让我想起了 C++ 的恐怖回忆",
                "内存操作过于危险，建议使用安全替代方案",
                "你的代码比走钢丝还危险，小心内存泄漏！",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "unsafe-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
                roast_level: RoastLevel::Savage,
            });
        }
    }
}

impl<'ast> Visit<'ast> for UnsafeVisitor {
    fn visit_expr_unsafe(&mut self, _unsafe_expr: &'ast ExprUnsafe) {
        self.unsafe_count += 1;

        let messages = [
            "Unsafe 代码！你这是在玩火还是在挑战 Rust 的底线？",
            "又见 unsafe！安全性是什么？能吃吗？",
            "Unsafe 使用者，恭喜你获得了'内存安全破坏者'称号",
            "这个 unsafe 让我想起了 C 语言的恐怖回忆",
            "Unsafe 代码：让 Rust 程序员夜不能寐的存在",
        ];

        let severity = if self.unsafe_count > 3 {
            Severity::Nuclear
        } else {
            Severity::Spicy
        };

        self.issues.push(CodeIssue {
            file_path: self.file_path.clone(),
            line: 1,
            column: 1,
            rule_name: "unsafe-abuse".to_string(),
            message: messages[self.issues.len() % messages.len()].to_string(),
            severity,
            roast_level: RoastLevel::Savage,
        });

        syn::visit::visit_expr_unsafe(self, _unsafe_expr);
    }

    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        if item_fn.sig.unsafety.is_some() {
            self.unsafe_fn_count += 1;
        }
        syn::visit::visit_item_fn(self, item_fn);
    }
}

// FFI Visitor
struct FFIVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    extern_block_count: usize,
    extern_fn_count: usize,
    c_repr_count: usize,
}

impl FFIVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            extern_block_count: 0,
            extern_fn_count: 0,
            c_repr_count: 0,
        }
    }

    fn check_ffi_patterns_in_content(&mut self, content: &str) {
        // 检查 C 表示法
        self.c_repr_count += content.matches("#[repr(C)]").count();

        // 检查 C 字符串操作
        let c_string_ops = [
            "CString",
            "CStr",
            "c_char",
            "c_void",
            "c_int",
            "c_long",
            "std::ffi::",
            "libc::",
            "std::os::raw::",
        ];

        let mut c_ops_count = 0;
        for op in &c_string_ops {
            c_ops_count += content.matches(op).count();
        }

        // 检查动态库加载
        let dll_ops = ["libloading", "dlopen", "LoadLibrary", "GetProcAddress"];
        let mut dll_count = 0;
        for op in &dll_ops {
            dll_count += content.matches(op).count();
        }

        self.generate_ffi_issues(c_ops_count, dll_count);
    }

    fn generate_ffi_issues(&mut self, c_ops_count: usize, dll_count: usize) {
        // 检查 extern 块过多
        if self.extern_block_count > 2 {
            let messages = [
                "Extern 块比我的前任还多，你这是要和多少种语言交互？",
                "这么多 extern 块，你确定不是在写多语言翻译器？",
                "FFI 接口过多，建议封装成统一的抽象层",
                "外部接口比我的社交关系还复杂！",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        // 检查 C 操作过多
        if c_ops_count > 10 {
            let messages = [
                "C 语言操作比我的 C 语言作业还多，你确定这是 Rust 项目？",
                "这么多 C FFI，Rust 的安全性都要哭了",
                "C 接口过多，建议使用更安全的 Rust 绑定",
                "你的 FFI 代码让我想起了指针地狱的恐怖",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
                roast_level: RoastLevel::Savage,
            });
        }

        // 检查动态库加载
        if dll_count > 0 {
            let messages = [
                "动态库加载！你这是在运行时玩杂技吗？",
                "动态加载库，小心加载到病毒！",
                "运行时库加载，调试的时候准备哭吧",
                "动态库操作，你的程序比变形金刚还会变身",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        // 检查 repr(C) 过多
        if self.c_repr_count > 5 {
            let messages = [
                "repr(C) 用得比我说 C 语言还频繁！",
                "这么多 C 表示法，你的结构体都要移民到 C 语言了",
                "C 表示法过多，内存布局都要乱套了",
                "repr(C) 滥用，Rust 的零成本抽象在哭泣",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }
    }
}

impl<'ast> Visit<'ast> for FFIVisitor {
    fn visit_item_foreign_mod(&mut self, foreign_mod: &'ast ItemForeignMod) {
        self.extern_block_count += 1;

        // 统计外部函数数量
        for item in &foreign_mod.items {
            if matches!(item, ForeignItem::Fn(_)) {
                self.extern_fn_count += 1;
            }
        }

        // 检查 extern 函数过多
        if self.extern_fn_count > 10 {
            let messages = [
                "外部函数比我的外卖订单还多！",
                "这么多 extern 函数，你是在开联合国大会吗？",
                "外部接口过多，建议分模块管理",
                "FFI 函数数量超标，小心接口管理混乱",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        syn::visit::visit_item_foreign_mod(self, foreign_mod);
    }

    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        // 检查是否是 extern "C" 函数
        if let Some(abi) = &item_fn.sig.abi {
            if let Some(abi_name) = &abi.name {
                if abi_name.value() == "C" {
                    self.extern_fn_count += 1;
                }
            }
        }
        syn::visit::visit_item_fn(self, item_fn);
    }
}

// Macro Visitor
struct MacroVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    macro_count: usize,
}

impl MacroVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            macro_count: 0,
        }
    }
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, _macro: &'ast Macro) {
        self.macro_count += 1;

        if self.macro_count > 10 {
            let messages = [
                "宏定义比我的借口还多",
                "这么多宏，你确定不是在写 C 语言？",
                "宏滥用！编译时间都被你搞长了",
                "宏过多，调试的时候准备哭吧",
                "这么多宏，IDE 都要罢工了",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "macro-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }

        syn::visit::visit_macro(self, _macro);
    }
}

// Module Visitor
struct ModuleVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    module_depth: usize,
    max_depth: usize,
}

impl ModuleVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            module_depth: 0,
            max_depth: 0,
        }
    }
}

impl<'ast> Visit<'ast> for ModuleVisitor {
    fn visit_item_mod(&mut self, _module: &'ast ItemMod) {
        self.module_depth += 1;
        self.max_depth = self.max_depth.max(self.module_depth);

        if self.module_depth > 5 {
            let messages = [
                "模块嵌套比俄罗斯套娃还深",
                "这模块结构比我的家族关系还复杂",
                "模块嵌套过深，建议重新组织代码结构",
                "这么深的模块，找个函数比找宝藏还难",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "module-complexity".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }

        syn::visit::visit_item_mod(self, _module);
        self.module_depth -= 1;
    }
}

// Pattern Visitor
struct PatternVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    complex_pattern_count: usize,
}

impl PatternVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            complex_pattern_count: 0,
        }
    }

    fn check_pattern_complexity(&mut self, pattern_type: &str) {
        self.complex_pattern_count += 1;

        if self.complex_pattern_count > 15 {
            let messages = [
                format!("{pattern_type}模式匹配比我的感情生活还复杂"),
                format!("这么多{pattern_type}模式，你是在写解谜游戏吗？"),
                format!("{pattern_type}模式过多，建议简化逻辑"),
                format!("复杂的{pattern_type}模式让代码可读性直线下降"),
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "pattern-matching-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }
    }
}

impl<'ast> Visit<'ast> for PatternVisitor {
    fn visit_pat_tuple(&mut self, _tuple_pat: &'ast PatTuple) {
        self.check_pattern_complexity("元组");
        syn::visit::visit_pat_tuple(self, _tuple_pat);
    }

    fn visit_pat_slice(&mut self, _slice_pat: &'ast PatSlice) {
        self.check_pattern_complexity("切片");
        syn::visit::visit_pat_slice(self, _slice_pat);
    }

    fn visit_expr_match(&mut self, match_expr: &'ast ExprMatch) {
        if match_expr.arms.len() > 10 {
            let messages = [
                "Match 分支比我的人生选择还多",
                "这么多 match 分支，你确定不是在写状态机？",
                "Match 分支过多，建议重构",
                "这个 match 比电视遥控器的按钮还多",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "pattern-matching-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
                roast_level: RoastLevel::Sarcastic,
            });
        }
        syn::visit::visit_expr_match(self, match_expr);
    }
}

// Reference Visitor
struct ReferenceVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    reference_count: usize,
}

impl ReferenceVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            reference_count: 0,
        }
    }
}

impl<'ast> Visit<'ast> for ReferenceVisitor {
    fn visit_type_reference(&mut self, _ref_type: &'ast TypeReference) {
        self.reference_count += 1;

        if self.reference_count > 20 {
            let messages = [
                "引用比我的社交关系还复杂",
                "这么多引用，你确定不是在写指针迷宫？",
                "引用过多，小心借用检查器罢工",
                "引用数量超标，建议重新设计数据结构",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "reference-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }

        syn::visit::visit_type_reference(self, _ref_type);
    }
}

// Box Visitor
struct BoxVisitor {
    #[allow(dead_code)]
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
}

impl BoxVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
        }
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
}

impl SliceVisitor {
    fn new(file_path: std::path::PathBuf) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            slice_count: 0,
        }
    }
}

impl<'ast> Visit<'ast> for SliceVisitor {
    fn visit_type_slice(&mut self, _slice_type: &'ast TypeSlice) {
        self.slice_count += 1;

        if self.slice_count > 15 {
            let messages = [
                "切片比我切菜还频繁",
                "这么多切片，你是在开水果店吗？",
                "切片过多，数组都被你切碎了",
                "Slice 滥用，建议使用 Vec",
            ];

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "slice-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
                roast_level: RoastLevel::Gentle,
            });
        }

        syn::visit::visit_type_slice(self, _slice_type);
    }
}

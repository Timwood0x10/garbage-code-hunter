use std::path::Path;
use syn::{
    visit::Visit, ExprAsync, ExprAwait, ExprMatch, ExprUnsafe, File, ForeignItem, ItemFn,
    ItemForeignMod, ItemMod, Macro, PatSlice, PatTuple, TypePath, TypeTraitObject,
};

use crate::analyzer::{CodeIssue, Severity};
use crate::rules::Rule;

pub struct ChannelAbuseRule;
pub struct AsyncAbuseRule;
pub struct DynTraitAbuseRule;
pub struct UnsafeAbuseRule;
pub struct FFIAbuseRule;
pub struct MacroAbuseRule;
pub struct ModuleComplexityRule;
pub struct PatternMatchingAbuseRule;

impl Rule for ChannelAbuseRule {
    fn name(&self) -> &'static str {
        "channel-abuse"
    }

    fn check(
        &self,
        file_path: &Path,
        syntax_tree: &File,
        content: &str,
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = ChannelVisitor::new(file_path.to_path_buf(), lang);
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
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = AsyncVisitor::new(file_path.to_path_buf(), lang);
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
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = DynTraitVisitor::new(file_path.to_path_buf(), lang);
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
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = UnsafeVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);

        // Check unsafe keyword usage in content
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
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = FFIVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);

        // Check FFI-related patterns in content
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
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = MacroVisitor::new(file_path.to_path_buf(), lang);
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
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = ModuleVisitor::new(file_path.to_path_buf(), lang);
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
        lang: &str,
        _is_test_file: bool,
    ) -> Vec<CodeIssue> {
        let mut visitor = PatternVisitor::new(file_path.to_path_buf(), lang);
        visitor.visit_file(syntax_tree);
        visitor.issues
    }
}

// Channel Visitor
struct ChannelVisitor {
    file_path: std::path::PathBuf,
    issues: Vec<CodeIssue>,
    channel_count: usize,
    lang: String,
}

impl ChannelVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            channel_count: 0,
            lang: lang.to_string(),
        }
    }

    fn check_channel_overuse(&mut self) {
        if self.channel_count > 5 {
            let messages = if self.lang == "zh-CN" {
                [
                    "Channel 用得比我发微信还频繁，你确定不是在写聊天软件？",
                    "这么多 Channel，你是想开通讯公司吗？",
                    "Channel 滥用！你的程序比电话交换机还复杂",
                    "Channel 数量超标，建议重新设计架构",
                    "这么多 Channel，我怀疑你在写分布式系统",
                ]
            } else {
                [
                    "You use channels more than I text - are you writing a chat app?",
                    "This many channels, planning to start a telecom company?",
                    "Channel abuse! Your program is more complex than a telephone switchboard",
                    "Channel count exceeds limits - consider redesigning the architecture",
                    "So many channels, I suspect you're building a distributed system",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "channel-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
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
    lang: String,
}

impl AsyncVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            async_count: 0,
            await_count: 0,
            lang: lang.to_string(),
        }
    }

    fn check_async_abuse(&mut self) {
        if self.async_count > 10 {
            let messages = if self.lang == "zh-CN" {
                [
                    "Async 函数比我的异步人生还要复杂",
                    "这么多 async，你确定不是在写 JavaScript？",
                    "Async 滥用！建议学习一下同步编程的美好",
                    "异步函数过多，小心把自己绕晕了",
                ]
            } else {
                [
                    "Async functions are more complex than my async life",
                    "So much async - are you sure you're not writing JavaScript?",
                    "Async abuse! Consider the beauty of synchronous programming",
                    "Too many async functions - careful not to confuse yourself",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "async-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        if self.await_count > 20 {
            let messages = if self.lang == "zh-CN" {
                [
                    "Await 用得比我等外卖还频繁",
                    "这么多 await，你的程序是在等什么？世界末日吗？",
                    "Await 过度使用，建议批量处理",
                    "等待次数过多，你的程序比我还有耐心",
                ]
            } else {
                [
                    "You await more than I wait for food delivery",
                    "So many awaits - what is your program waiting for, the apocalypse?",
                    "Excessive await usage - consider batching operations",
                    "Your program is more patient than I'll ever be",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "async-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
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
    lang: String,
}

impl DynTraitVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            dyn_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for DynTraitVisitor {
    fn visit_type_trait_object(&mut self, trait_object: &'ast TypeTraitObject) {
        self.dyn_count += 1;

        if self.dyn_count > 5 {
            let messages = if self.lang == "zh-CN" {
                [
                    "Dyn trait 用得比我换工作还频繁",
                    "这么多动态分发，性能都跑到哪里去了？",
                    "Dyn trait 滥用，你确定不是在写 Python？",
                    "动态 trait 过多，编译器优化都哭了",
                    "这么多 dyn，你的程序比变色龙还善变",
                ]
            } else {
                [
                    "You use dyn traits more often than I change jobs",
                    "So much dynamic dispatch - where did all the performance go?",
                    "dyn trait abuse - are you sure you're not writing Python?",
                    "Too many dynamic traits - the compiler optimizer is crying",
                    "So many dyns, your program is more wishy-washy than a chameleon",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "dyn-trait-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
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
    lang: String,
}

impl UnsafeVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            unsafe_count: 0,
            unsafe_fn_count: 0,
            unsafe_impl_count: 0,
            unsafe_trait_count: 0,
            lang: lang.to_string(),
        }
    }

    fn check_unsafe_in_content(&mut self, content: &str) {
        // Check unsafe functions
        let unsafe_fn_matches = content.matches("unsafe fn").count();
        self.unsafe_fn_count += unsafe_fn_matches;

        // Check unsafe impl
        let unsafe_impl_matches = content.matches("unsafe impl").count();
        self.unsafe_impl_count += unsafe_impl_matches;

        // Check unsafe trait
        let unsafe_trait_matches = content.matches("unsafe trait").count();
        self.unsafe_trait_count += unsafe_trait_matches;

        // Check raw pointer operations
        let raw_ptr_count = content.matches("*const").count() + content.matches("*mut").count();

        // Check memory operation functions
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
        // Check for excessive unsafe functions
        if self.unsafe_fn_count > 2 {
            let messages = if self.lang == "zh-CN" {
                [
                    "Unsafe 函数比我的黑历史还多！你确定这还是 Rust 吗？",
                    "这么多 unsafe 函数，Rust 的安全保证都被你玩坏了",
                    "Unsafe 函数过多，建议重新考虑设计架构",
                    "你的 unsafe 函数让 Rust 编译器都开始怀疑人生了",
                ]
            } else {
                [
                    "More unsafe functions than my dark secrets! Are you sure this is still Rust?",
                    "So many unsafe functions - Rust's safety guarantees are in shambles",
                    "Too many unsafe functions - consider redesigning the architecture",
                    "Your unsafe functions are making the Rust compiler question its life choices",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "unsafe-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
            });
        }

        // Check for excessive raw pointers
        if raw_ptr_count > 5 {
            let messages = if self.lang == "zh-CN" {
                [
                    "原始指针用得比我换手机还频繁，你这是在写 C 语言吗？",
                    "这么多原始指针，内存安全已经不在服务区了",
                    "原始指针过多，建议使用安全的 Rust 抽象",
                    "你的指针操作让 Valgrind 都要加班了",
                ]
            } else {
                [
                    "You use raw pointers more than I change phones - are you writing C?",
                    "So many raw pointers, memory safety has left the building",
                    "Too many raw pointers - consider using safe Rust abstractions",
                    "Your pointer operations are making Valgrind work overtime",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "unsafe-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
            });
        }

        // Check for excessive dangerous operations
        if dangerous_op_count > 3 {
            let messages = if self.lang == "zh-CN" {
                [
                    "危险的内存操作比我的危险驾驶还要多！",
                    "这些危险操作让我想起了 C++ 的恐怖回忆",
                    "内存操作过于危险，建议使用安全替代方案",
                    "你的代码比走钢丝还危险，小心内存泄漏！",
                ]
            } else {
                [
                    "More dangerous memory ops than my reckless driving!",
                    "These dangerous ops bring back terrifying C++ memories",
                    "Memory operations are too dangerous - use safe alternatives",
                    "Your code is more dangerous than tightrope walking - watch for memory leaks!",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "unsafe-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
            });
        }
    }
}

impl<'ast> Visit<'ast> for UnsafeVisitor {
    fn visit_expr_unsafe(&mut self, _unsafe_expr: &'ast ExprUnsafe) {
        self.unsafe_count += 1;

        let messages = if self.lang == "zh-CN" {
            [
                "Unsafe 代码！你这是在玩火还是在挑战 Rust 的底线？",
                "又见 unsafe！安全性是什么？能吃吗？",
                "Unsafe 使用者，恭喜你获得了'内存安全破坏者'称号",
                "这个 unsafe 让我想起了 C 语言的恐怖回忆",
                "Unsafe 代码：让 Rust 程序员夜不能寐的存在",
            ]
        } else {
            [
                "Unsafe code! Playing with fire or challenging Rust's limits?",
                "Unsafe again! What is safety, can you eat it?",
                "Unsafe user - congratulations, you've earned the 'Memory Safety Destroyer' title",
                "This unsafe brings back terrifying memories of C programming",
                "Unsafe code: the thing that keeps Rust programmers up at night",
            ]
        };

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
    lang: String,
}

impl FFIVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            extern_block_count: 0,
            extern_fn_count: 0,
            c_repr_count: 0,
            lang: lang.to_string(),
        }
    }

    fn check_ffi_patterns_in_content(&mut self, content: &str) {
        // Check C representation
        self.c_repr_count += content.matches("#[repr(C)]").count();

        // Check C string operations
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

        // Check dynamic library loading
        let dll_ops = ["libloading", "dlopen", "LoadLibrary", "GetProcAddress"];
        let mut dll_count = 0;
        for op in &dll_ops {
            dll_count += content.matches(op).count();
        }

        self.generate_ffi_issues(c_ops_count, dll_count);
    }

    fn generate_ffi_issues(&mut self, c_ops_count: usize, dll_count: usize) {
        // Check for excessive extern blocks
        if self.extern_block_count > 2 {
            let messages = if self.lang == "zh-CN" {
                [
                    "Extern 块比我的前任还多，你这是要和多少种语言交互？",
                    "这么多 extern 块，你确定不是在写多语言翻译器？",
                    "FFI 接口过多，建议封装成统一的抽象层",
                    "外部接口比我的社交关系还复杂！",
                ]
            } else {
                [
                    "More extern blocks than my exes - how many languages are you interfacing with?",
                    "So many extern blocks - are you building a multilingual translator?",
                    "Too many FFI interfaces - consider wrapping them in a unified abstraction layer",
                    "References are more complex than my social life!",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        // Check for excessive C operations
        if c_ops_count > 10 {
            let messages = if self.lang == "zh-CN" {
                [
                    "C 语言操作比我的 C 语言作业还多，你确定这是 Rust 项目？",
                    "这么多 C FFI，Rust 的安全性都要哭了",
                    "C 接口过多，建议使用更安全的 Rust 绑定",
                    "你的 FFI 代码让我想起了指针地狱的恐怖",
                ]
            } else {
                [
                    "More C operations than my C homework - are you sure this is a Rust project?",
                    "So much C FFI, Rust's safety guarantees are crying",
                    "Too many C interfaces - consider using safer Rust bindings",
                    "Your FFI code brings back the horrors of pointer hell",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Nuclear,
            });
        }

        // Check dynamic library loading
        if dll_count > 0 {
            let messages = if self.lang == "zh-CN" {
                [
                    "动态库加载！你这是在运行时玩杂技吗？",
                    "动态加载库，小心加载到病毒！",
                    "运行时库加载，调试的时候准备哭吧",
                    "动态库操作，你的程序比变形金刚还会变身",
                ]
            } else {
                [
                    "Dynamic library loading! Are you performing acrobatics at runtime?",
                    "Dynamic loading - careful not to load a virus!",
                    "Runtime library loading - get ready to cry during debugging",
                    "Dynamic library ops - your program transforms more than a Transformer",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        // Check for excessive repr(C)
        if self.c_repr_count > 5 {
            let messages = if self.lang == "zh-CN" {
                [
                    "repr(C) 用得比我说 C 语言还频繁！",
                    "这么多 C 表示法，你的结构体都要移民到 C 语言了",
                    "C 表示法过多，内存布局都要乱套了",
                    "repr(C) 滥用，Rust 的零成本抽象在哭泣",
                ]
            } else {
                [
                    "You use repr(C) more than I speak C!",
                    "So many C representations, your structs are emigrating to C",
                    "Too many C representations - memory layout is a mess",
                    "repr(C) abuse - Rust's zero-cost abstractions are weeping",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }
    }
}

impl<'ast> Visit<'ast> for FFIVisitor {
    fn visit_item_foreign_mod(&mut self, foreign_mod: &'ast ItemForeignMod) {
        self.extern_block_count += 1;

        // Count external functions
        for item in &foreign_mod.items {
            if matches!(item, ForeignItem::Fn(_)) {
                self.extern_fn_count += 1;
            }
        }

        // Check for excessive extern functions
        if self.extern_fn_count > 10 {
            let messages = if self.lang == "zh-CN" {
                [
                    "外部函数比我的外卖订单还多！",
                    "这么多 extern 函数，你是在开联合国大会吗？",
                    "外部接口过多，建议分模块管理",
                    "FFI 函数数量超标，小心接口管理混乱",
                ]
            } else {
                [
                    "More extern functions than my food delivery orders!",
                    "So many extern functions - are you hosting a UN summit?",
                    "Too many external interfaces - consider modular management",
                    "FFI function count exceeds limits - watch out for interface chaos",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "ffi-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }

        syn::visit::visit_item_foreign_mod(self, foreign_mod);
    }

    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        // Check if it's an extern "C" function
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
    lang: String,
}

impl MacroVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            macro_count: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for MacroVisitor {
    fn visit_macro(&mut self, _macro: &'ast Macro) {
        self.macro_count += 1;

        if self.macro_count > 10 {
            let messages = if self.lang == "zh-CN" {
                [
                    "宏定义比我的借口还多",
                    "这么多宏，你确定不是在写 C 语言？",
                    "宏滥用！编译时间都被你搞长了",
                    "宏过多，调试的时候准备哭吧",
                    "这么多宏，IDE 都要罢工了",
                ]
            } else {
                [
                    "More macros than my excuses",
                    "So many macros - are you sure you're not writing C?",
                    "Macro abuse! You've inflated the compile times",
                    "Too many macros - get ready to cry during debugging",
                    "So many macros, your IDE is about to go on strike",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "macro-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
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
    lang: String,
}

impl ModuleVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            module_depth: 0,
            max_depth: 0,
            lang: lang.to_string(),
        }
    }
}

impl<'ast> Visit<'ast> for ModuleVisitor {
    fn visit_item_mod(&mut self, _module: &'ast ItemMod) {
        self.module_depth += 1;
        self.max_depth = self.max_depth.max(self.module_depth);

        if self.module_depth > 5 {
            let messages = if self.lang == "zh-CN" {
                [
                    "模块嵌套比俄罗斯套娃还深",
                    "这模块结构比我的家族关系还复杂",
                    "模块嵌套过深，建议重新组织代码结构",
                    "这么深的模块，找个函数比找宝藏还难",
                ]
            } else {
                [
                    "Module nesting is deeper than Russian dolls",
                    "This module structure is more complex than my family tree",
                    "Module nesting too deep - consider restructuring your code",
                    "Modules so deep, finding a function is harder than finding treasure",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "module-complexity".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
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
    lang: String,
}

impl PatternVisitor {
    fn new(file_path: std::path::PathBuf, lang: &str) -> Self {
        Self {
            file_path,
            issues: Vec::new(),
            complex_pattern_count: 0,
            lang: lang.to_string(),
        }
    }

    fn check_pattern_complexity(&mut self, pattern_type: &str) {
        self.complex_pattern_count += 1;

        if self.complex_pattern_count > 15 {
            let messages = if self.lang == "zh-CN" {
                [
                    format!("{pattern_type}模式匹配比我的感情生活还复杂"),
                    format!("这么多{pattern_type}模式，你是在写解谜游戏吗？"),
                    format!("{pattern_type}模式过多，建议简化逻辑"),
                    format!("复杂的{pattern_type}模式让代码可读性直线下降"),
                ]
            } else {
                let en_type = match pattern_type {
                    "元组" => "tuple",
                    "切片" => "slice",
                    _ => pattern_type,
                };
                [
                    format!("{en_type} pattern matching is more complex than my love life"),
                    format!("So many {en_type} patterns - are you writing a puzzle game?"),
                    format!("Too many {en_type} patterns - consider simplifying the logic"),
                    format!("Complex {en_type} patterns tanking your code readability"),
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "pattern-matching-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Mild,
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
            let messages = if self.lang == "zh-CN" {
                [
                    "Match 分支比我的人生选择还多",
                    "这么多 match 分支，你确定不是在写状态机？",
                    "Match 分支过多，建议重构",
                    "这个 match 比电视遥控器的按钮还多",
                ]
            } else {
                [
                    "More match arms than life choices I've made",
                    "So many match branches - are you building a state machine?",
                    "Too many match arms - consider refactoring",
                    "This match has more arms than a TV remote has buttons",
                ]
            };

            self.issues.push(CodeIssue {
                file_path: self.file_path.clone(),
                line: 1,
                column: 1,
                rule_name: "pattern-matching-abuse".to_string(),
                message: messages[self.issues.len() % messages.len()].to_string(),
                severity: Severity::Spicy,
            });
        }
        syn::visit::visit_expr_match(self, match_expr);
    }
}

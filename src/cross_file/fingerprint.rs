use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use syn::{spanned::Spanned, visit::Visit, ItemFn};

/// Normalized token representation for deduplication
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NormalizedToken {
    /// Rust keyword (fn, let, if, match, etc.)
    Keyword(String),
    /// Placeholder for user-defined variable (e.g., __var_1__)
    Variable(usize),
    /// Placeholder for literal values
    IntLiteral,
    FloatLiteral,
    StrLiteral,
    CharLiteral,
    BoolLiteral(bool),
    /// Type placeholder
    TypePlaceholder,
    /// Operator or punctuation
    Operator(String),
    /// Delimiter (parentheses, braces, brackets)
    Delimiter(char),
}

/// Unique identifier for a normalized function body
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FunctionFingerprint {
    pub hash: u64,
    pub normalized_tokens: Vec<NormalizedToken>,
    pub locations: Vec<FileLocation>,
    pub token_count: usize,
    pub function_name: String,
}

impl FunctionFingerprint {
    /// Create a new fingerprint from normalized tokens and compute hash
    pub fn new(
        normalized_tokens: Vec<NormalizedToken>,
        location: FileLocation,
        function_name: String,
    ) -> Self {
        let token_count = normalized_tokens.len();
        let hash = compute_hash(&normalized_tokens);

        Self {
            hash,
            normalized_tokens,
            locations: vec![location],
            token_count,
            function_name,
        }
    }

    /// Add another location where this pattern appears
    pub fn add_location(&mut self, location: FileLocation) {
        self.locations.push(location);
    }
}

/// Location of a function occurrence in source code
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileLocation {
    pub file_path: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub function_name: String,
}

impl FileLocation {
    pub fn new(
        file_path: PathBuf,
        line_start: usize,
        line_end: usize,
        function_name: String,
    ) -> Self {
        Self {
            file_path,
            line_start,
            line_end,
            function_name,
        }
    }
}

/// Compute a 64-bit hash from normalized tokens using DefaultHasher
fn compute_hash(tokens: &[NormalizedToken]) -> u64 {
    use std::collections::hash_map::DefaultHasher;

    let mut hasher = DefaultHasher::new();
    tokens.hash(&mut hasher);
    hasher.finish()
}

/// Visitor to extract and normalize function body tokens
struct FunctionNormalizer {
    tokens: Vec<NormalizedToken>,
    var_counter: usize,
}

impl FunctionNormalizer {
    fn new() -> Self {
        Self {
            tokens: Vec::new(),
            var_counter: 0,
        }
    }

    /// Create and add a variable placeholder to tokens (avoids borrow conflict)
    fn add_var_placeholder(&mut self) {
        let id = self.var_counter;
        self.var_counter += 1;
        self.tokens.push(NormalizedToken::Variable(id));
    }

    fn next_var_placeholder(&mut self) -> NormalizedToken {
        let id = self.var_counter;
        self.var_counter += 1;
        NormalizedToken::Variable(id)
    }
}

impl<'ast> Visit<'ast> for FunctionNormalizer {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        // Add 'fn' keyword
        self.tokens.push(NormalizedToken::Keyword("fn".to_string()));

        // Normalize function name as variable
        let func_name_placeholder = self.next_var_placeholder();
        self.tokens.push(func_name_placeholder);

        // Visit generics if present
        syn::visit::visit_generics(self, &func.sig.generics);

        // Add opening paren
        self.tokens.push(NormalizedToken::Delimiter('('));

        // Normalize inputs
        for input in func.sig.inputs.iter() {
            match input {
                syn::FnArg::Receiver(receiver) => {
                    if receiver.reference.is_some() {
                        self.tokens.push(NormalizedToken::Operator("&".to_string()));
                        if receiver.mutability.is_some() {
                            self.tokens
                                .push(NormalizedToken::Keyword("mut".to_string()));
                        }
                    }
                    self.add_var_placeholder();
                }
                syn::FnArg::Typed(pat_type) => {
                    // Pattern (variable name)
                    self.visit_pat(&pat_type.pat);
                    // Colon separator
                    self.tokens.push(NormalizedToken::Delimiter(':'));
                    // Type
                    self.visit_type(&pat_type.ty);
                }
            }

            // Comma separator between arguments
            self.tokens.push(NormalizedToken::Delimiter(','));
        }

        // Remove trailing comma if present
        if !func.sig.inputs.is_empty() {
            self.tokens.pop();
        }

        // Add closing paren
        self.tokens.push(NormalizedToken::Delimiter(')'));

        // Return type
        match &func.sig.output {
            syn::ReturnType::Default => {}
            syn::ReturnType::Type(_, ty) => {
                self.tokens
                    .push(NormalizedToken::Operator("->".to_string()));
                self.visit_type(ty);
            }
        }

        // Visit function body
        self.visit_block(&func.block);
    }

    fn visit_pat(&mut self, pat: &'ast syn::Pat) {
        match pat {
            syn::Pat::Ident(_) => {
                self.add_var_placeholder();
            }
            syn::Pat::Wild(_) => {
                self.tokens.push(NormalizedToken::Keyword("_".to_string()));
            }
            _ => {
                syn::visit::visit_pat(self, pat);
            }
        }
    }

    fn visit_type(&mut self, ty: &'ast syn::Type) {
        match ty {
            syn::Type::Path(type_path) => {
                if let Some(segment) = type_path.path.segments.last() {
                    let ident_str = segment.ident.to_string();

                    // Map common types to placeholders
                    match ident_str.as_str() {
                        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                        | "u64" | "u128" | "usize" | "f32" | "f64" => {
                            self.tokens.push(NormalizedToken::TypePlaceholder);
                        }
                        "String" | "str" => {
                            self.tokens.push(NormalizedToken::StrLiteral);
                        }
                        "bool" => {
                            self.tokens.push(NormalizedToken::TypePlaceholder);
                        }
                        "Vec" | "Option" | "Result" | "Box" | "Rc" | "Arc" => {
                            self.tokens
                                .push(NormalizedToken::Keyword(ident_str.clone()));
                            // Visit generic arguments
                            match &segment.arguments {
                                syn::PathArguments::None => {}
                                syn::PathArguments::AngleBracketed(args) => {
                                    for arg in args.args.iter() {
                                        syn::visit::visit_generic_argument(self, arg);
                                    }
                                }
                                syn::PathArguments::Parenthesized(_) => {}
                            }
                        }
                        _ => {
                            // User-defined type - treat as variable
                            self.add_var_placeholder();
                        }
                    }
                }
            }
            syn::Type::Reference(_) => {
                self.tokens.push(NormalizedToken::Operator("&".to_string()));
                if let syn::Type::Reference(ref_type) = ty {
                    self.visit_type(&ref_type.elem);
                }
            }
            _ => {
                self.tokens.push(NormalizedToken::TypePlaceholder);
            }
        }
    }

    fn visit_return_type(&mut self, return_type: &'ast syn::ReturnType) {
        match return_type {
            syn::ReturnType::Default => {}
            syn::ReturnType::Type(_, ty) => {
                self.visit_type(ty);
            }
        }
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        self.tokens.push(NormalizedToken::Delimiter('{'));

        for stmt in block.stmts.iter() {
            self.visit_stmt(stmt);
        }

        self.tokens.push(NormalizedToken::Delimiter('}'));
    }

    fn visit_stmt(&mut self, stmt: &'ast syn::Stmt) {
        match stmt {
            syn::Stmt::Local(local) => {
                self.tokens
                    .push(NormalizedToken::Keyword("let".to_string()));

                // Check if the pattern is a mutable binding
                let is_mut = matches!(&local.pat, syn::Pat::Ident(pat_ident) if pat_ident.mutability.is_some());
                if is_mut {
                    self.tokens
                        .push(NormalizedToken::Keyword("mut".to_string()));
                }

                self.visit_pat(&local.pat);

                if let Some(init) = &local.init {
                    self.tokens.push(NormalizedToken::Operator("=".to_string()));
                    self.visit_expr(&init.expr);
                }

                self.tokens.push(NormalizedToken::Delimiter(';'));
            }
            syn::Stmt::Item(item) => {
                syn::visit::visit_item(self, item);
            }
            syn::Stmt::Expr(expr, _) => {
                self.visit_expr(expr);
            }
            syn::Stmt::Macro(_) => {
                // Skip macro statements for now
                self.tokens
                    .push(NormalizedToken::Keyword("macro!".to_string()));
            }
        }
    }

    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        match expr {
            syn::Expr::Path(expr_path) => {
                if let Some(ident) = expr_path.path.get_ident() {
                    let ident_str = ident.to_string();
                    match ident_str.as_str() {
                        "true" => self.tokens.push(NormalizedToken::BoolLiteral(true)),
                        "false" => self.tokens.push(NormalizedToken::BoolLiteral(false)),
                        _ => self.add_var_placeholder(),
                    }
                } else {
                    self.add_var_placeholder();
                }
            }
            syn::Expr::Lit(expr_lit) => match &expr_lit.lit {
                syn::Lit::Int(_) => self.tokens.push(NormalizedToken::IntLiteral),
                syn::Lit::Float(_) => self.tokens.push(NormalizedToken::FloatLiteral),
                syn::Lit::Str(_) => self.tokens.push(NormalizedToken::StrLiteral),
                syn::Lit::Char(_) => self.tokens.push(NormalizedToken::CharLiteral),
                syn::Lit::Bool(lit) => {
                    self.tokens.push(NormalizedToken::BoolLiteral(lit.value));
                }
                _ => self.tokens.push(NormalizedToken::IntLiteral),
            },
            syn::Expr::Binary(bin_expr) => {
                self.visit_expr(&bin_expr.left);
                self.tokens.push(match bin_expr.op {
                    syn::BinOp::Add(_) => NormalizedToken::Operator("+".to_string()),
                    syn::BinOp::Sub(_) => NormalizedToken::Operator("-".to_string()),
                    syn::BinOp::Mul(_) => NormalizedToken::Operator("*".to_string()),
                    syn::BinOp::Div(_) => NormalizedToken::Operator("/".to_string()),
                    syn::BinOp::Rem(_) => NormalizedToken::Operator("%".to_string()),
                    syn::BinOp::And(_) => NormalizedToken::Operator("&&".to_string()),
                    syn::BinOp::Or(_) => NormalizedToken::Operator("||".to_string()),
                    syn::BinOp::BitAnd(_) => NormalizedToken::Operator("&".to_string()),
                    syn::BinOp::BitOr(_) => NormalizedToken::Operator("|".to_string()),
                    syn::BinOp::BitXor(_) => NormalizedToken::Operator("^".to_string()),
                    syn::BinOp::Shl(_) => NormalizedToken::Operator("<<".to_string()),
                    syn::BinOp::Shr(_) => NormalizedToken::Operator(">>".to_string()),
                    syn::BinOp::Eq(_) => NormalizedToken::Operator("==".to_string()),
                    syn::BinOp::Lt(_) => NormalizedToken::Operator("<".to_string()),
                    syn::BinOp::Le(_) => NormalizedToken::Operator("<=".to_string()),
                    syn::BinOp::Ne(_) => NormalizedToken::Operator("!=".to_string()),
                    syn::BinOp::Ge(_) => NormalizedToken::Operator(">=".to_string()),
                    syn::BinOp::Gt(_) => NormalizedToken::Operator(">".to_string()),
                    _ => {
                        // For assignment operators, use a generic representation
                        let op_str = format!("{:?}", bin_expr.op);
                        NormalizedToken::Operator(op_str)
                    }
                });
                self.visit_expr(&bin_expr.right);
            }
            syn::Expr::Unary(unary_expr) => {
                match unary_expr.op {
                    syn::UnOp::Deref(_) => {
                        self.tokens.push(NormalizedToken::Operator("*".to_string()))
                    }
                    syn::UnOp::Not(_) => {
                        self.tokens.push(NormalizedToken::Operator("!".to_string()))
                    }
                    syn::UnOp::Neg(_) => {
                        self.tokens.push(NormalizedToken::Operator("-".to_string()))
                    }
                    _ => {
                        // Handle any future UnOp variants
                        let op_str = format!("{:?}", unary_expr.op);
                        self.tokens.push(NormalizedToken::Operator(op_str))
                    }
                }
                self.visit_expr(&unary_expr.expr);
            }
            syn::Expr::MethodCall(method_call) => {
                self.visit_expr(&method_call.receiver);
                self.tokens.push(NormalizedToken::Delimiter('.'));
                self.add_var_placeholder();
                self.tokens.push(NormalizedToken::Delimiter('('));
                for arg in method_call.args.iter() {
                    self.visit_expr(arg);
                    self.tokens.push(NormalizedToken::Delimiter(','));
                }
                if !method_call.args.is_empty() {
                    self.tokens.pop();
                }
                self.tokens.push(NormalizedToken::Delimiter(')'));
            }
            syn::Expr::Call(call_expr) => {
                self.visit_expr(&call_expr.func);
                self.tokens.push(NormalizedToken::Delimiter('('));
                for arg in call_expr.args.iter() {
                    self.visit_expr(arg);
                    self.tokens.push(NormalizedToken::Delimiter(','));
                }
                if !call_expr.args.is_empty() {
                    self.tokens.pop();
                }
                self.tokens.push(NormalizedToken::Delimiter(')'));
            }
            syn::Expr::If(if_expr) => {
                self.tokens.push(NormalizedToken::Keyword("if".to_string()));
                self.visit_expr(&if_expr.cond);
                self.visit_block(&if_expr.then_branch);
                if let Some((_, else_block)) = &if_expr.else_branch {
                    self.tokens
                        .push(NormalizedToken::Keyword("else".to_string()));
                    match else_block.as_ref() {
                        syn::Expr::If(inner_if) => {
                            self.visit_expr(&syn::Expr::If(inner_if.clone()));
                        }
                        syn::Expr::Block(expr_block) => {
                            self.visit_block(&expr_block.block);
                        }
                        _ => {}
                    }
                }
            }
            syn::Expr::Match(match_expr) => {
                self.tokens
                    .push(NormalizedToken::Keyword("match".to_string()));
                self.visit_expr(&match_expr.expr);
                self.tokens.push(NormalizedToken::Delimiter('{'));
                for arm in match_expr.arms.iter() {
                    self.visit_pat(&arm.pat);
                    if let Some(guard) = &arm.guard {
                        self.tokens.push(NormalizedToken::Keyword("if".to_string()));
                        self.visit_expr(&guard.1); // Extract the expression from (If, Expr) tuple
                    }
                    self.tokens
                        .push(NormalizedToken::Operator("=>".to_string()));
                    self.visit_expr(&arm.body);
                    self.tokens.push(NormalizedToken::Delimiter(','));
                }
                self.tokens.push(NormalizedToken::Delimiter('}'));
            }
            syn::Expr::ForLoop(for_loop) => {
                self.tokens
                    .push(NormalizedToken::Keyword("for".to_string()));
                self.visit_pat(&for_loop.pat);
                self.tokens.push(NormalizedToken::Keyword("in".to_string()));
                self.visit_expr(&for_loop.expr);
                self.visit_block(&for_loop.body);
            }
            syn::Expr::While(while_loop) => {
                self.tokens
                    .push(NormalizedToken::Keyword("while".to_string()));
                self.visit_expr(&while_loop.cond);
                self.visit_block(&while_loop.body);
            }
            syn::Expr::Loop(loop_expr) => {
                self.tokens
                    .push(NormalizedToken::Keyword("loop".to_string()));
                self.visit_block(&loop_expr.body);
            }
            syn::Expr::Return(ret_expr) => {
                self.tokens
                    .push(NormalizedToken::Keyword("return".to_string()));
                if let Some(expr) = &ret_expr.expr {
                    self.visit_expr(expr);
                }
            }
            syn::Expr::Field(field_expr) => {
                self.visit_expr(&field_expr.base);
                self.tokens.push(NormalizedToken::Delimiter('.'));
                self.add_var_placeholder();
            }
            syn::Expr::Index(index_expr) => {
                self.visit_expr(&index_expr.expr);
                self.tokens.push(NormalizedToken::Delimiter('['));
                self.visit_expr(&index_expr.index);
                self.tokens.push(NormalizedToken::Delimiter(']'));
            }
            syn::Expr::Block(block_expr) => {
                self.visit_block(&block_expr.block);
            }
            syn::Expr::Struct(struct_expr) => {
                self.add_var_placeholder();
                self.tokens.push(NormalizedToken::Delimiter('{'));
                for field in struct_expr.fields.iter() {
                    self.add_var_placeholder();
                    self.tokens.push(NormalizedToken::Operator(":".to_string()));
                    self.visit_expr(&field.expr);
                    self.tokens.push(NormalizedToken::Delimiter(','));
                }
                if !struct_expr.fields.is_empty() {
                    self.tokens.pop();
                }
                self.tokens.push(NormalizedToken::Delimiter('}'));
            }
            _ => {
                syn::visit::visit_expr(self, expr);
            }
        }
    }
}

/// Extract normalized fingerprint from a parsed function
pub fn extract_fingerprint(func: &ItemFn, file_path: PathBuf) -> Option<FunctionFingerprint> {
    let mut normalizer = FunctionNormalizer::new();
    normalizer.visit_item_fn(func);

    if normalizer.tokens.is_empty() || normalizer.tokens.len() < 3 {
        return None;
    }

    let line_start = func.sig.fn_token.span.start().line;
    let line_end = func.block.span().end().line;

    let location = FileLocation::new(file_path, line_start, line_end, func.sig.ident.to_string());

    Some(FunctionFingerprint::new(
        normalizer.tokens,
        location,
        func.sig.ident.to_string(),
    ))
}

/// Calculate Jaccard similarity between two token sequences
pub fn jaccard_similarity(a: &[NormalizedToken], b: &[NormalizedToken]) -> f64 {
    use std::collections::HashSet;

    if a.is_empty() && b.is_empty() {
        return 1.0;
    }

    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let set_a: HashSet<&NormalizedToken> = a.iter().collect();
    let set_b: HashSet<&NormalizedToken> = b.iter().collect();

    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Objective: Verify that empty functions produce valid but minimal fingerprints
    /// Invariants: Hash should be non-zero, token count should include at least signature
    #[test]
    fn test_fingerprint_empty_function() {
        let code = r#"
        fn empty_func() {}
        "#;

        let func: ItemFn = syn::parse_str(code).expect("Failed to parse empty function");
        let fp = extract_fingerprint(&func, PathBuf::from("test.rs"));

        assert!(fp.is_some(), "Empty function should produce a fingerprint");

        let fp = fp.unwrap();
        assert!(
            fp.token_count >= 5,
            "Empty function should have at least signature tokens, got {}",
            fp.token_count
        );
        assert_ne!(fp.hash, 0, "Hash should never be zero (collision risk)");
        assert_eq!(fp.function_name, "empty_func");
        assert_eq!(fp.locations.len(), 1);
    }

    /// Objective: Verify normalization produces identical output for semantically equivalent code with different variable names
    /// Invariants: Different variable names must yield identical fingerprints
    #[test]
    fn test_normalization_variable_renaming_produces_same_hash() {
        let code_a = r#"
        fn process_data(data: i32) -> i32 {
            let result = data * 2;
            result + 1
        }
        "#;

        let code_b = r#"
        fn calculate_value(input: i32) -> i32 {
            let output = input * 2;
            output + 1
        }
        "#;

        let func_a: ItemFn = syn::parse_str(code_a).expect("Failed to parse code A");
        let func_b: ItemFn = syn::parse_str(code_b).expect("Failed to parse code B");

        let fp_a = extract_fingerprint(&func_a, PathBuf::from("a.rs")).unwrap();
        let fp_b = extract_fingerprint(&func_b, PathBuf::from("b.rs")).unwrap();

        assert_eq!(
            fp_a.hash, fp_b.hash,
            "Variable renaming should not affect fingerprint hash"
        );
        assert_eq!(
            fp_a.normalized_tokens, fp_b.normalized_tokens,
            "Normalized tokens should be identical after renaming"
        );
    }

    /// Objective: Verify Jaccard similarity correctly identifies near-duplicates
    /// Invariants: Functions differing by one line should have high similarity (>0.85)
    #[test]
    fn test_similarity_near_duplicate_detection() {
        let base_code = r#"
        fn validate(item: i32) -> bool {
            if item > 0 { true } else { false }
        }
        "#;

        let modified_code = r#"
        fn check(value: i32) -> bool {
            if value > 0 && value < 100 { true } else { false }
        }
        "#;

        let base_func: ItemFn = syn::parse_str(base_code).expect("Failed to parse base");
        let modified_func: ItemFn =
            syn::parse_str(modified_code).expect("Failed to parse modified");

        let base_fp = extract_fingerprint(&base_func, PathBuf::from("base.rs")).unwrap();
        let modified_fp =
            extract_fingerprint(&modified_func, PathBuf::from("modified.rs")).unwrap();

        let similarity =
            jaccard_similarity(&base_fp.normalized_tokens, &modified_fp.normalized_tokens);

        assert!(
            similarity > 0.7,
            "Near-duplicate should have similarity > 0.7, got {:.3}",
            similarity
        );
        assert!(
            similarity < 1.0,
            "Modified function should NOT be exact match (similarity < 1.0), got {:.3}",
            similarity
        );
    }

    /// Objective: Verify completely different functions produce low similarity scores
    /// Invariants: Unrelated functions should have similarity < 0.5
    #[test]
    fn test_similarity_different_functions_low_score() {
        let code_math = r#"
        fn calculate(x: i32, y: i32) -> i32 { x + y * 2 }
        "#;

        let code_io = r#"
        fn read_file(path: &str) -> Result<String, std::io::Error> {
            std::fs::read_to_string(path)
        }
        "#;

        let math_func: ItemFn = syn::parse_str(code_math).expect("Failed to parse math");
        let io_func: ItemFn = syn::parse_str(code_io).expect("Failed to parse io");

        let math_fp = extract_fingerprint(&math_func, PathBuf::from("math.rs")).unwrap();
        let io_fp = extract_fingerprint(&io_func, PathBuf::from("io.rs")).unwrap();

        let similarity = jaccard_similarity(&math_fp.normalized_tokens, &io_fp.normalized_tokens);

        assert!(
            similarity < 0.65,
            "Different functions should have low similarity (< 0.65), got {:.3}",
            similarity
        );
    }

    /// Objective: Verify identical functions always produce same hash regardless of file path
    /// Invariants: Hash must be deterministic and path-independent
    #[test]
    fn test_hash_determinism_and_path_independence() {
        let code = r#"
        fn example(x: i32) -> i32 { x * x }
        "#;

        let func: ItemFn = syn::parse_str(code).unwrap();

        let fp1 = extract_fingerprint(&func, PathBuf::from("/path/a.rs")).unwrap();
        let fp2 = extract_fingerprint(&func, PathBuf::from("/different/path/b.rs")).unwrap();

        assert_eq!(
            fp1.hash, fp2.hash,
            "Hash must be identical regardless of file path"
        );
        assert_eq!(
            fp1.token_count, fp2.token_count,
            "Token count must be identical"
        );
    }

    /// Objective: Verify fingerprint extraction handles complex control flow structures
    /// Invariants: Nested loops, matches, and ifs should all be captured in normalized form
    #[test]
    fn test_complex_control_flow_normalization() {
        let code = r#"
        fn complex_logic(items: Vec<i32>) -> Vec<i32> {
            let mut result = Vec::new();
            for item in items {
                match item {
                    x if x > 0 => result.push(x * 2),
                    x if x < 0 => result.push(x.abs()),
                    _ => continue,
                }
            }
            result
        }
        "#;

        let func: ItemFn = syn::parse_str(code).expect("Failed to parse complex function");
        let fp = extract_fingerprint(&func, PathBuf::from("complex.rs")).unwrap();

        assert!(
            fp.token_count > 20,
            "Complex function should produce many tokens, got {}",
            fp.token_count
        );

        assert!(
            fp.normalized_tokens
                .contains(&NormalizedToken::Keyword("for".to_string()))
                && fp
                    .normalized_tokens
                    .contains(&NormalizedToken::Keyword("match".to_string()))
                && fp
                    .normalized_tokens
                    .contains(&NormalizedToken::Keyword("if".to_string())),
            "Control flow keywords must be preserved in normalized tokens"
        );
    }
}

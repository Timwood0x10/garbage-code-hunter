use syn::spanned::Spanned;

/// Get line number from syn AST node
/// Note: Due to proc_macro2::Span limitations, we can only return the default value
pub fn get_line_number<T: Spanned>(_node: &T) -> usize {
    // proc_macro2::Span does not provide position info in non proc-macro context
    // This is a known limitation, we return the default value
    1
}

/// Get column number from syn AST node
pub fn get_column_number<T: Spanned>(_node: &T) -> usize {
    // Same limitation, return default value
    1
}

/// Get position info (line, column) from syn AST node
/// Note: In the current implementation, this returns (1, 1) as the default value
pub fn get_position<T: Spanned>(node: &T) -> (usize, usize) {
    (get_line_number(node), get_column_number(node))
}

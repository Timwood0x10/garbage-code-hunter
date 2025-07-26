use syn::spanned::Spanned;

/// 从 syn AST 节点获取行号
/// 注意：由于 proc_macro2::Span 的限制，我们只能返回默认值
pub fn get_line_number<T: Spanned>(_node: &T) -> usize {
    // proc_macro2::Span 在非 proc-macro 上下文中不提供位置信息
    // 这是一个已知限制，我们返回默认值
    1
}

/// 从 syn AST 节点获取列号
pub fn get_column_number<T: Spanned>(_node: &T) -> usize {
    // 同样的限制，返回默认值
    1
}

/// 从 syn AST 节点获取位置信息 (行号, 列号)
/// 注意：在当前实现中，这将返回 (1, 1) 作为默认值
pub fn get_position<T: Spanned>(node: &T) -> (usize, usize) {
    (get_line_number(node), get_column_number(node))
}
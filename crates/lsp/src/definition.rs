use syntax::ast::Expression;

use crate::offset_in_span;
use crate::snapshot::AnalysisSnapshot;
use crate::traversal::find_expression_at;

pub(crate) fn get_root_expression(e: &Expression) -> &Expression {
    let mut current = e;
    while let Expression::DotAccess { expression, .. } = current {
        current = expression;
    }
    current
}

pub(crate) fn find_struct_field_span(
    type_id: &str,
    field_name: &str,
    snapshot: &AnalysisSnapshot,
) -> Option<syntax::ast::Span> {
    use syntax::program::{Definition, DefinitionBody};

    if let Some(Definition {
        body: DefinitionBody::Struct { fields, .. },
        ..
    }) = snapshot.definitions().get(type_id)
    {
        fields
            .iter()
            .find(|f| f.name == field_name)
            .map(|f| f.name_span)
    } else {
        None
    }
}

/// True when the span points into a generated `go:` typedef file. Used by rename
/// to refuse edits to typedefs, which would diverge from the regenerated content.
pub(crate) fn is_go_typedef_span(snapshot: &AnalysisSnapshot, span: &syntax::ast::Span) -> bool {
    snapshot
        .files()
        .get(&span.file_id)
        .is_some_and(|f| f.module_id.starts_with("go:"))
}

/// Resolve the definition span at the given cursor offset.
///
/// Checks binding definitions first, then falls back to expression-based resolution.
/// `extra_match` handles caller-specific arms (e.g. DotAccess for references, rename guards).
pub(crate) fn resolve_definition_span(
    snapshot: &AnalysisSnapshot,
    file: &syntax::program::File,
    file_id: u32,
    offset: u32,
    extra_match: impl FnOnce(&Expression) -> Option<syntax::ast::Span>,
) -> Option<syntax::ast::Span> {
    // Resolution facts lead for usages (the token references a definition
    // elsewhere). The binding/decl-site arms below cover definition sites, which
    // the ref table doesn't record.
    if let Some(def_span) = snapshot.ref_target_at(file_id, offset) {
        return Some(def_span);
    }

    snapshot
        .facts()
        .bindings
        .values()
        .find_map(|binding| {
            if binding.span.file_id == file_id && offset_in_span(offset, &binding.span) {
                Some(binding.span)
            } else {
                None
            }
        })
        .or_else(|| {
            let expression = find_expression_at(&file.items, offset)?;
            match expression {
                Expression::Identifier {
                    binding_id: Some(id),
                    ..
                } => snapshot.facts().bindings.get(id).map(|b| b.span),

                Expression::Function { name_span, .. }
                | Expression::Interface { name_span, .. }
                | Expression::TypeAlias { name_span, .. } => Some(*name_span),

                Expression::Struct {
                    name,
                    name_span,
                    fields,
                    ..
                } => fields
                    .iter()
                    .find(|f| offset_in_span(offset, &f.name_span))
                    .and_then(|f| {
                        let qualified = format!("{}.{}", file.module_id, name);
                        find_struct_field_span(&qualified, &f.name, snapshot)
                    })
                    .or(Some(*name_span)),

                Expression::Enum {
                    name,
                    name_span,
                    variants,
                    ..
                } => variants
                    .iter()
                    .find(|v| offset_in_span(offset, &v.name_span))
                    .and_then(|v| {
                        let qualified = format!("{}.{}.{}", file.module_id, name, v.name);
                        snapshot
                            .definitions()
                            .get(qualified.as_str())
                            .and_then(|d| d.name_span())
                    })
                    .or(Some(*name_span)),

                Expression::Const {
                    identifier_span, ..
                } => Some(*identifier_span),

                Expression::VariableDeclaration { name_span, .. } => Some(*name_span),

                other => extra_match(other),
            }
        })
}

use crate::passes::walk::NodeCtx;
use syntax::ast::{Expression, MatchArm, Span};

pub(crate) fn check(expression: &Expression, ctx: &NodeCtx) {
    let Expression::Function {
        attributes,
        name,
        name_span,
        params,
        return_type,
        body,
        ..
    } = expression
    else {
        return;
    };

    if !attributes.iter().any(|a| a.name == "tailcall") {
        return;
    }

    if return_type.is_unit() {
        ctx.sink
            .push(diagnostics::infer::tailcall_unit_return(*name_span, name));
        return;
    }

    let mut analysis = Analysis::default();
    walk(body, true, name, params.len(), &mut analysis);

    if !analysis.has_tail_call && analysis.non_tail_calls.is_empty() {
        if let Some(call_span) = find_method_form_self_call(body, name) {
            ctx.sink
                .push(diagnostics::infer::tailcall_method_form_unsupported(
                    call_span, name,
                ));
        } else {
            ctx.sink
                .push(diagnostics::infer::tailcall_no_self_call(*name_span, name));
        }
        return;
    }

    for span in &analysis.non_tail_calls {
        ctx.sink
            .push(diagnostics::infer::tailcall_not_in_tail_position(
                *span, name,
            ));
    }
}

fn find_method_form_self_call(expr: &Expression, name: &str) -> Option<Span> {
    if let Expression::Call {
        expression, span, ..
    } = expr
        && let Expression::DotAccess { member, .. } = expression.as_ref()
        && member.as_str() == name
    {
        return Some(*span);
    }
    expr.children()
        .iter()
        .find_map(|child| find_method_form_self_call(child, name))
}

#[derive(Default)]
struct Analysis {
    has_tail_call: bool,
    non_tail_calls: Vec<Span>,
}

/// Walks the function body collecting self-call sites with their tail/non-tail
/// classification. Invariant: only called on the function body root (or via
/// recursion into a tail-propagating child). Never called on arbitrary
/// fragments, so `Return.expression` may safely re-mark its inner as tail —
/// `return X` is always in tail position from the function's perspective.
fn walk(expr: &Expression, tail: bool, name: &str, param_count: usize, out: &mut Analysis) {
    if let Some(args) = expr.self_call_to(name, param_count) {
        if tail {
            out.has_tail_call = true;
        } else {
            out.non_tail_calls.push(expr.get_span());
        }
        for arg in args {
            walk(arg, false, name, param_count, out);
        }
        return;
    }

    match expr {
        Expression::Block { items, .. } => {
            if let Some((last, rest)) = items.split_last() {
                for item in rest {
                    walk(item, false, name, param_count, out);
                }
                walk(last, tail, name, param_count, out);
            }
        }
        Expression::If {
            condition,
            consequence,
            alternative,
            ..
        } => {
            walk(condition, false, name, param_count, out);
            walk(consequence, tail, name, param_count, out);
            walk(alternative, tail, name, param_count, out);
        }
        Expression::Match { subject, arms, .. } => {
            walk(subject, false, name, param_count, out);
            for MatchArm {
                expression, guard, ..
            } in arms
            {
                if let Some(g) = guard {
                    walk(g, false, name, param_count, out);
                }
                walk(expression, tail, name, param_count, out);
            }
        }
        Expression::IfLet {
            scrutinee,
            consequence,
            alternative,
            ..
        } => {
            walk(scrutinee, false, name, param_count, out);
            walk(consequence, tail, name, param_count, out);
            walk(alternative, tail, name, param_count, out);
        }
        Expression::WhileLet {
            scrutinee, body, ..
        } => {
            walk(scrutinee, false, name, param_count, out);
            walk(body, false, name, param_count, out);
        }
        Expression::While {
            condition, body, ..
        } => {
            walk(condition, false, name, param_count, out);
            walk(body, false, name, param_count, out);
        }
        Expression::Loop { body, .. } => {
            walk(body, false, name, param_count, out);
        }
        Expression::For { iterable, body, .. } => {
            walk(iterable, false, name, param_count, out);
            walk(body, false, name, param_count, out);
        }
        Expression::Let { value, .. } => {
            walk(value, false, name, param_count, out);
        }
        Expression::Return { expression, .. } => {
            walk(expression, true, name, param_count, out);
        }
        Expression::Paren { expression, .. } => {
            walk(expression, tail, name, param_count, out);
        }
        Expression::Call {
            expression, args, ..
        } => {
            walk(expression, false, name, param_count, out);
            for arg in args {
                walk(arg, false, name, param_count, out);
            }
        }
        Expression::Binary { left, right, .. } => {
            walk(left, false, name, param_count, out);
            walk(right, false, name, param_count, out);
        }
        Expression::Unary { expression, .. } => {
            walk(expression, false, name, param_count, out);
        }
        Expression::DotAccess { expression, .. } => {
            walk(expression, false, name, param_count, out);
        }
        Expression::Tuple { elements, .. } => {
            for e in elements {
                walk(e, false, name, param_count, out);
            }
        }
        Expression::StructCall {
            field_assignments, ..
        } => {
            for fa in field_assignments {
                walk(&fa.value, false, name, param_count, out);
            }
        }
        Expression::TryBlock { items, .. } | Expression::RecoverBlock { items, .. } => {
            for item in items {
                walk(item, false, name, param_count, out);
            }
        }
        Expression::Task { expression, .. } | Expression::Defer { expression, .. } => {
            walk(expression, false, name, param_count, out);
        }
        // Conservative fallback for variants that wrap or consume their children
        // (Cast, IndexedAccess, Reference, Propagate, Range, Assignment, …). The
        // inner value is consumed by the wrapping expression, so children are
        // never in tail position from the function's perspective.
        other => {
            for child in other.children().iter() {
                walk(child, false, name, param_count, out);
            }
        }
    }
}

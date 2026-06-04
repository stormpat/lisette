use crate::passes::walk::NodeCtx;
use syntax::ast::{Expression, MatchArm, Span};

pub(crate) fn check(expression: &Expression, ctx: &NodeCtx) {
    let Expression::Function {
        attributes,
        name,
        name_span,
        params,
        body,
        ..
    } = expression
    else {
        return;
    };

    if !attributes.iter().any(|a| a.name == "tailcall") {
        return;
    }

    let mut analysis = Analysis::default();
    walk(body, true, name, params.len(), &mut analysis);

    if analysis.tail_calls.is_empty() && analysis.non_tail_calls.is_empty() {
        ctx.sink
            .push(diagnostics::infer::tailcall_no_self_call(name_span, name));
        return;
    }

    for span in &analysis.non_tail_calls {
        ctx.sink
            .push(diagnostics::infer::tailcall_not_in_tail_position(
                span, name,
            ));
    }
}

#[derive(Default)]
struct Analysis {
    tail_calls: Vec<Span>,
    non_tail_calls: Vec<Span>,
}

fn walk(expr: &Expression, tail: bool, name: &str, param_count: usize, out: &mut Analysis) {
    if let Some(span) = self_call_span(expr, name, param_count) {
        if tail {
            out.tail_calls.push(span);
        } else {
            out.non_tail_calls.push(span);
        }
        if let Expression::Call { args, .. } = expr {
            for arg in args {
                walk(arg, false, name, param_count, out);
            }
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
        _ => {}
    }
}

fn self_call_span(expr: &Expression, name: &str, param_count: usize) -> Option<Span> {
    let Expression::Call {
        expression,
        args,
        span,
        ..
    } = expr
    else {
        return None;
    };
    let Expression::Identifier { value, .. } = expression.as_ref() else {
        return None;
    };
    if value.as_str() != name || args.len() != param_count {
        return None;
    }
    Some(*span)
}

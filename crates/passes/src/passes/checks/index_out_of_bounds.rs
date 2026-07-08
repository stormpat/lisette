use crate::passes::walk::NodeCtx;
use syntax::ast::{Expression, Literal, UnaryOperator};
use syntax::types::Type;

pub(crate) fn check(expression: &Expression, ctx: &NodeCtx) {
    let Expression::IndexedAccess {
        expression: receiver,
        index,
        from_colon_syntax,
        span,
        ..
    } = expression
    else {
        return;
    };

    if *from_colon_syntax {
        return;
    }

    let receiver_ty = ctx.store.peel_alias(&receiver.get_type());

    if let Some(negative) = negative_index_literal(index)
        && (matches!(receiver_ty, Type::Array { .. }) || receiver_ty.is_slice())
    {
        ctx.sink.push(diagnostics::infer::negative_index(
            span,
            &negative.to_string(),
        ));
        return;
    }

    if let Type::Array { length, .. } = &receiver_ty
        && let Some(value) = index.as_integer()
        && value >= *length
    {
        ctx.sink.push(diagnostics::infer::index_out_of_bounds(
            span,
            &value.to_string(),
        ));
        return;
    }

    if !receiver_ty.is_slice() {
        return;
    }

    if let Expression::Literal {
        literal: Literal::Slice(elements),
        ..
    } = receiver.unwrap_parens()
        && let Some(value) = index.as_integer()
        && value >= elements.len() as u64
    {
        ctx.sink.push(diagnostics::infer::index_out_of_bounds(
            span,
            &value.to_string(),
        ));
        return;
    }

    if let Expression::Call {
        expression: callee,
        args,
        ..
    } = index.unwrap_parens()
        && args.is_empty()
        && let Expression::DotAccess {
            expression: call_receiver,
            member,
            ..
        } = callee.unwrap_parens()
        && member == "length"
        && expressions_equivalent(receiver, call_receiver)
    {
        let receiver_text = receiver.root_identifier().unwrap_or("xs");
        ctx.sink.push(diagnostics::infer::index_out_of_bounds(
            span,
            &format!("{receiver_text}.length()"),
        ));
    }
}

fn negative_index_literal(index: &Expression) -> Option<i128> {
    let Expression::Unary {
        operator: UnaryOperator::Negative,
        expression,
        ..
    } = index.unwrap_parens()
    else {
        return None;
    };
    let value = expression.as_integer()?;
    (value > 0).then_some(-(value as i128))
}

fn expressions_equivalent(a: &Expression, b: &Expression) -> bool {
    let a = a.unwrap_parens();
    let b = b.unwrap_parens();
    match (a, b) {
        (Expression::Identifier { value: av, .. }, Expression::Identifier { value: bv, .. }) => {
            av == bv
        }
        (
            Expression::DotAccess {
                expression: ae,
                member: am,
                ..
            },
            Expression::DotAccess {
                expression: be,
                member: bm,
                ..
            },
        ) => am == bm && expressions_equivalent(ae, be),
        _ => false,
    }
}

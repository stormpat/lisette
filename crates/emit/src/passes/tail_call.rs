use syntax::ast::{Attribute, Binding, Expression, Pattern};

use crate::EmitEffects;
use crate::Planner;
use crate::context::expression::ExpressionContext;

pub(crate) fn has_tailcall_attribute(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|a| a.name == "tailcall")
}

/// If the planner is in tail-call mode and `call` is a self-call with matching
/// arity, return its args. Otherwise None.
pub(crate) fn match_tail_self_call<'a>(
    planner: &Planner<'_>,
    call: &'a Expression,
) -> Option<&'a [Expression]> {
    let state = planner.function_state.tail_call()?;
    let Expression::Call {
        expression, args, ..
    } = call
    else {
        return None;
    };
    let Expression::Identifier { value, .. } = expression.as_ref() else {
        return None;
    };
    if value.as_str() != state.function_name || args.len() != state.param_count {
        return None;
    }
    Some(args.as_slice())
}

/// Emit the Go for "reassign params + continue" given the recursive args.
pub(crate) fn emit_reassign_and_continue(
    planner: &mut Planner<'_>,
    args: &[Expression],
    fx: &mut EmitEffects,
) -> String {
    let mut setup = String::new();
    let arg_strs: Vec<String> = args
        .iter()
        .map(|arg| planner.emit_value(&mut setup, arg, ExpressionContext::value(), fx))
        .collect();

    let param_names = planner
        .function_state
        .tail_call()
        .expect("tail-call state must be set when calling emit_reassign_and_continue")
        .param_go_names
        .clone();

    let mut out = setup;
    out.push_str(&format!(
        "{} = {}\n",
        param_names.join(", "),
        arg_strs.join(", ")
    ));
    out.push_str("continue\n");
    out
}

pub(crate) fn resolve_param_go_names(planner: &Planner<'_>, params: &[Binding]) -> Vec<String> {
    params
        .iter()
        .map(|p| match &p.pattern {
            Pattern::Identifier { identifier, .. } => planner
                .scope
                .resolve_binding_go_name(identifier)
                .map(str::to_string)
                .unwrap_or_else(|| identifier.to_string()),
            _ => "_".to_string(),
        })
        .collect()
}

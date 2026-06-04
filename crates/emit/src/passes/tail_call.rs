use syntax::ast::{Attribute, Binding, Expression, Pattern, TypedPattern};
use syntax::types::Type;

use crate::EmitEffects;
use crate::Planner;
use crate::context::expression::ExpressionContext;

pub(crate) type DeferredParamDestructure = (String, Pattern, Option<TypedPattern>, Type);

pub(crate) fn has_tailcall_attribute(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|a| a.name == "tailcall")
}

pub(crate) fn match_tail_self_call<'a>(
    planner: &Planner<'_>,
    call: &'a Expression,
) -> Option<&'a [Expression]> {
    let state = planner.function_state.tail_call()?;
    call.self_call_to(&state.function_name, state.param_count)
}

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

pub(crate) fn resolve_param_go_names(
    planner: &Planner<'_>,
    params: &[Binding],
    deferred: &[DeferredParamDestructure],
) -> Vec<String> {
    let mut deferred_iter = deferred.iter();
    params
        .iter()
        .map(|p| match &p.pattern {
            Pattern::Identifier { identifier, .. } => planner
                .scope
                .resolve_binding_go_name(identifier)
                .map(str::to_string)
                .unwrap_or_else(|| identifier.to_string()),
            Pattern::WildCard { .. } => "_".to_string(),
            _ => deferred_iter
                .next()
                .map(|d| d.0.clone())
                .unwrap_or_else(|| "_".to_string()),
        })
        .collect()
}

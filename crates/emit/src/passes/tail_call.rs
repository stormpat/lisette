use syntax::ast::{Binding, Expression, Pattern, TypedPattern};
use syntax::types::Type;

use crate::EmitEffects;
use crate::Planner;
use crate::context::expression::ExpressionContext;

pub(crate) type DeferredParamDestructure = (String, Pattern, Option<TypedPattern>, Type);

pub(crate) struct TailSelfCallMatch<'a> {
    pub args: &'a [Expression],
    pub param_go_names: Vec<String>,
}

pub(crate) fn match_tail_self_call<'a>(
    planner: &Planner<'_>,
    call: &'a Expression,
) -> Option<TailSelfCallMatch<'a>> {
    let state = planner.function_state.tail_call()?;
    let args = call.self_call_to(&state.function_name, state.param_count)?;
    Some(TailSelfCallMatch {
        args,
        param_go_names: state.param_go_names.clone(),
    })
}

pub(crate) fn emit_reassign_and_continue(
    planner: &mut Planner<'_>,
    args: &[Expression],
    param_go_names: &[String],
    fx: &mut EmitEffects,
) -> String {
    let mut setup = String::new();
    let arg_strs: Vec<String> = args
        .iter()
        .map(|arg| planner.emit_value(&mut setup, arg, ExpressionContext::value(), fx))
        .collect();

    let mut out = setup;
    out.push_str(&format!(
        "{} = {}\n",
        param_go_names.join(", "),
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

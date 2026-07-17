//! Information-criterion model comparison and selection.

use super::models::TransientModelKind;
use crate::results::transient::{
    FitStatus, TransientFitResult, TransientWarning, TransientWarningKind,
};
use crate::transient_config::SelectionCriterion;

#[derive(Debug, Clone, Copy, Default)]
pub struct SelectionOutcome {
    pub selected_index: Option<usize>,
    pub selected_model: Option<TransientModelKind>,
}

pub fn select_model(
    fits: &mut [TransientFitResult],
    criterion: SelectionCriterion,
) -> SelectionOutcome {
    let values = fits
        .iter()
        .enumerate()
        .filter_map(|(index, fit)| {
            if fit.status != FitStatus::Converged {
                return None;
            }
            let value = match criterion {
                SelectionCriterion::Aic => fit.statistics.aic,
                SelectionCriterion::Bic => fit.statistics.bic,
            }?;
            value.is_finite().then_some((index, value))
        })
        .collect::<Vec<_>>();
    let Some((best_index, best_value)) = values.iter().copied().min_by(|left, right| {
        left.1
            .total_cmp(&right.1)
            .then_with(|| left.0.cmp(&right.0))
    }) else {
        return SelectionOutcome::default();
    };

    let weights = values
        .iter()
        .map(|(_, value)| (-0.5 * (*value - best_value)).exp())
        .collect::<Vec<_>>();
    let weight_sum = weights.iter().sum::<f64>();
    for fit in fits.iter_mut() {
        if fit.status != FitStatus::Converged {
            continue;
        }
        let value = match criterion {
            SelectionCriterion::Aic => fit.statistics.aic,
            SelectionCriterion::Bic => fit.statistics.bic,
        };
        if let Some(value) = value.filter(|value| value.is_finite()) {
            fit.statistics.criterion_delta = Some(value - best_value);
            if weight_sum.is_finite() && weight_sum > 0.0 {
                let weight = (-0.5 * (value - best_value)).exp() / weight_sum;
                fit.statistics.model_weight = weight.is_finite().then_some(weight);
            }
        }
        if fit
            .statistics
            .criterion_delta
            .is_some_and(|delta| delta > 10.0)
        {
            fit.warnings.push(TransientWarning::new(
                TransientWarningKind::NotIdentifiable,
                "candidate model has weak information-criterion support relative to the selected model",
            ));
        }
    }

    SelectionOutcome {
        selected_index: Some(best_index),
        selected_model: Some(fits[best_index].model),
    }
}

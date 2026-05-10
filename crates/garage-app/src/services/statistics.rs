use chrono::{DateTime, Utc};
use garage_domain::{Currency, Money, SignedMoney};

use crate::{AppResult, RepairRepository};

/// Profit summary in one currency.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfitSummary {
    pub currency: Currency,
    pub completed_repairs: usize,
    pub revenue: Money,
    pub parts_cost: Money,
    pub expected_profit: SignedMoney,
    pub actual_profit: SignedMoney,
}

/// Use cases for statistics.
pub struct StatisticsService<R> {
    repairs: R,
}

impl<R> StatisticsService<R>
where
    R: RepairRepository,
{
    pub fn new(repairs: R) -> Self {
        Self { repairs }
    }

    pub async fn profit_summary(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        currency: Currency,
    ) -> AppResult<ProfitSummary> {
        let repairs = self.repairs.list_completed_between(from, to).await?;
        let mut completed_repairs = 0;
        let mut revenue = Money::zero(currency);
        let mut parts_cost = Money::zero(currency);
        let mut expected_profit = SignedMoney::zero(currency);
        let mut actual_profit = SignedMoney::zero(currency);

        for repair in repairs
            .into_iter()
            .filter(|repair| repair.currency() == currency)
        {
            completed_repairs += 1;
            revenue = revenue.checked_add(repair.total_price()?)?;
            parts_cost = parts_cost.checked_add(repair.parts_cost())?;
            expected_profit = expected_profit.checked_add(repair.expected_profit()?)?;
            actual_profit = actual_profit.checked_add(repair.actual_profit()?)?;
        }

        Ok(ProfitSummary {
            currency,
            completed_repairs,
            revenue,
            parts_cost,
            expected_profit,
            actual_profit,
        })
    }
}

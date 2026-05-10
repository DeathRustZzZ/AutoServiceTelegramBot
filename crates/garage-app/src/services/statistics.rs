//! Сценарии статистики.
//!
//! Статистика строится по ремонтам, а не по booking. Booking описывает план
//! визита, но не содержит цен, себестоимости и оплат.

use chrono::{DateTime, Utc};
use garage_domain::{Currency, Money, SignedMoney};

use crate::{AppResult, RepairRepository};

/// Финансовая сводка в одной валюте.
///
/// До появления провайдера курсов валют статистика не конвертирует BYN/USD, а
/// фильтрует ремонты по запрошенной валюте.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfitSummary {
    /// Валюта сводки.
    pub currency: Currency,
    /// Количество завершенных ремонтов, попавших в расчет.
    pub completed_repairs: usize,
    /// Выручка по клиентским ценам.
    pub revenue: Money,
    /// Себестоимость запчастей для сервиса.
    pub parts_cost: Money,
    /// Ожидаемая прибыль: полная цена ремонта минус себестоимость.
    pub expected_profit: SignedMoney,
    /// Фактическая прибыль: полученные оплаты минус себестоимость.
    pub actual_profit: SignedMoney,
}

/// Application service для статистики.
pub struct StatisticsService<R> {
    repairs: R,
}

impl<R> StatisticsService<R>
where
    R: RepairRepository,
{
    /// Создает сервис статистики поверх репозитория ремонтов.
    pub fn new(repairs: R) -> Self {
        Self { repairs }
    }

    /// Считает прибыль по завершенным ремонтам за период.
    ///
    /// Репозиторий возвращает completed repairs за диапазон, сервис фильтрует
    /// валюту и суммирует доменные финансовые вычисления через checked-методы.
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

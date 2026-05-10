//! Запчасть, использованная в конкретном ремонте.
//!
//! `RepairPart` фиксирует факт расхода складской позиции в рамках ремонта:
//! какую запчасть использовали, в каком количестве, по какой себестоимости и
//! по какой цене для клиента. Это отдельная доменная сущность, а не поле внутри
//! `Repair`, потому что у одного ремонта может быть несколько строк запчастей,
//! и каждая строка имеет собственную историю фиксации.
//!
//! Модель намеренно не хранит название, SKU или поставщика. Это связь с
//! конкретной складской позицией `Part`: отображение человекочитаемых данных и
//! исторические снапшоты будут решаться прикладным или инфраструктурным слоем,
//! если бизнесу это понадобится.
//!
//! Важно разделять ответственность:
//! - `RepairPart` не списывает остаток со склада;
//! - `RepairPart` не пересчитывает и не меняет `Repair`;
//! - `RepairPart` только валидирует и считает собственные финансовые итоги.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{
    Currency, Money, MoneyError, PartId, PartQuantity, RepairId, RepairPartId, SignedMoney,
};

/// Запчасть, использованная в конкретном ремонте.
///
/// Поля закрыты, чтобы нельзя было создать строку с нулевым количеством или с
/// разными валютами себестоимости и цены. Изменяющих методов здесь нет:
/// использование запчасти является зафиксированным фактом. Если позже появится
/// сценарий корректировки, его лучше моделировать отдельным app use case, а не
/// незаметно переписывать историю.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairPart {
    /// Стабильный идентификатор строки использованной запчасти.
    id: RepairPartId,
    /// Ремонт, в котором использовали запчасть.
    repair_id: RepairId,
    /// Складская позиция, которая была использована.
    part_id: PartId,
    /// Количество использованных единиц. Ноль запрещен.
    quantity: PartQuantity,
    /// Себестоимость одной единицы для автосервиса.
    unit_cost: Money,
    /// Цена одной единицы для клиента.
    unit_price: Money,
    /// Момент фиксации факта использования.
    created_at: DateTime<Utc>,
}

impl RepairPart {
    /// Создает новую строку использованной запчасти.
    ///
    /// Алгоритм:
    /// 1. Проверяем, что количество не равно нулю. Нулевая строка не несет
    ///    бизнес-смысла и загрязняет историю ремонта.
    /// 2. Проверяем, что себестоимость и клиентская цена выражены в одной
    ///    валюте. Конвертация валют не относится к domain layer.
    /// 3. Сохраняем `now` как момент фиксации факта использования.
    ///
    /// Метод не трогает складской остаток и не меняет агрегированные суммы
    /// `Repair`: эти операции должны быть скоординированы use case-ом в
    /// `garage-app`.
    pub fn new(
        id: RepairPartId,
        repair_id: RepairId,
        part_id: PartId,
        quantity: PartQuantity,
        unit_cost: Money,
        unit_price: Money,
        now: DateTime<Utc>,
    ) -> Result<Self, RepairPartError> {
        Self::restore(id, repair_id, part_id, quantity, unit_cost, unit_price, now)
    }

    /// Восстанавливает строку использованной запчасти из сохраненного состояния.
    ///
    /// Восстановление проверяет те же инварианты, что и создание: нулевая
    /// строка и смешанные валюты не становятся валидными только потому, что
    /// пришли из базы или другого внешнего источника.
    pub fn restore(
        id: RepairPartId,
        repair_id: RepairId,
        part_id: PartId,
        quantity: PartQuantity,
        unit_cost: Money,
        unit_price: Money,
        created_at: DateTime<Utc>,
    ) -> Result<Self, RepairPartError> {
        if quantity.is_zero() {
            return Err(RepairPartError::ZeroQuantity);
        }

        if unit_cost.currency() != unit_price.currency() {
            return Err(RepairPartError::CurrencyMismatch {
                cost: unit_cost.currency(),
                price: unit_price.currency(),
            });
        }

        Ok(Self {
            id,
            repair_id,
            part_id,
            quantity,
            unit_cost,
            unit_price,
            created_at,
        })
    }

    /// Возвращает идентификатор строки использованной запчасти.
    pub fn id(&self) -> RepairPartId {
        self.id
    }

    /// Возвращает идентификатор ремонта.
    pub fn repair_id(&self) -> RepairId {
        self.repair_id
    }

    /// Возвращает идентификатор складской позиции.
    pub fn part_id(&self) -> PartId {
        self.part_id
    }

    /// Возвращает использованное количество.
    pub fn quantity(&self) -> PartQuantity {
        self.quantity
    }

    /// Возвращает себестоимость одной единицы.
    pub fn unit_cost(&self) -> Money {
        self.unit_cost
    }

    /// Возвращает цену одной единицы для клиента.
    pub fn unit_price(&self) -> Money {
        self.unit_price
    }

    /// Возвращает момент фиксации факта использования.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Считает полную себестоимость строки.
    ///
    /// Формула: `unit_cost * quantity`.
    pub fn total_cost(&self) -> Result<Money, RepairPartError> {
        Ok(self.unit_cost.checked_mul_u32(self.quantity.value())?)
    }

    /// Считает полную цену строки для клиента.
    ///
    /// Формула: `unit_price * quantity`.
    pub fn total_price(&self) -> Result<Money, RepairPartError> {
        Ok(self.unit_price.checked_mul_u32(self.quantity.value())?)
    }

    /// Считает прибыль по строке использованной запчасти.
    ///
    /// Формула: `total_price - total_cost`.
    ///
    /// Результат возвращается как `SignedMoney`, потому что цена для клиента
    /// может быть ниже себестоимости. Это допустимый бизнес-факт, а не ошибка.
    pub fn profit(&self) -> Result<SignedMoney, RepairPartError> {
        let total_price = SignedMoney::from(self.total_price()?);
        let total_cost = SignedMoney::from(self.total_cost()?);

        Ok(total_price.checked_sub(total_cost)?)
    }
}

/// Ошибка строки использованной запчасти.
///
/// Ошибки отражают только правила самой сущности: количество, валюту и
/// checked-арифметику денежных расчетов.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepairPartError {
    /// Нельзя фиксировать использование нулевого количества запчастей.
    #[error("repair part quantity cannot be zero")]
    ZeroQuantity,

    /// Себестоимость и цена для клиента выражены в разных валютах.
    #[error("repair part currency mismatch: cost={cost:?}, price={price:?}")]
    CurrencyMismatch { cost: Currency, price: Currency },

    /// Ошибка денежной арифметики.
    #[error(transparent)]
    Money(#[from] MoneyError),
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{RepairPart, RepairPartError};
    use crate::{
        Currency, Money, MoneyError, PartId, PartQuantity, RepairId, RepairPartId, SignedMoney,
    };

    fn fixed_time(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn repair_part_id() -> RepairPartId {
        RepairPartId::from_uuid(Uuid::from_u128(1))
    }

    fn repair_id() -> RepairId {
        RepairId::from_uuid(Uuid::from_u128(2))
    }

    fn part_id() -> PartId {
        PartId::from_uuid(Uuid::from_u128(3))
    }

    fn repair_part(unit_cost: Money, unit_price: Money, quantity: u32) -> RepairPart {
        RepairPart::new(
            repair_part_id(),
            repair_id(),
            part_id(),
            PartQuantity::new(quantity),
            unit_cost,
            unit_price,
            fixed_time(1_700_000_000),
        )
        .unwrap()
    }

    #[test]
    fn repair_part_new_accepts_valid_values() {
        let created_at = fixed_time(1_700_000_000);
        let part = RepairPart::new(
            repair_part_id(),
            repair_id(),
            part_id(),
            PartQuantity::new(2),
            Money::byn_minor(1000).unwrap(),
            Money::byn_minor(1500).unwrap(),
            created_at,
        )
        .unwrap();

        assert_eq!(part.id(), repair_part_id());
        assert_eq!(part.repair_id(), repair_id());
        assert_eq!(part.part_id(), part_id());
        assert_eq!(part.quantity(), PartQuantity::new(2));
        assert_eq!(part.unit_cost(), Money::byn_minor(1000).unwrap());
        assert_eq!(part.unit_price(), Money::byn_minor(1500).unwrap());
        assert_eq!(*part.created_at(), created_at);
    }

    #[test]
    fn repair_part_new_rejects_zero_quantity() {
        let error = RepairPart::new(
            repair_part_id(),
            repair_id(),
            part_id(),
            PartQuantity::zero(),
            Money::byn_minor(1000).unwrap(),
            Money::byn_minor(1500).unwrap(),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, RepairPartError::ZeroQuantity);
    }

    #[test]
    fn repair_part_restore_rejects_zero_quantity() {
        let error = RepairPart::restore(
            repair_part_id(),
            repair_id(),
            part_id(),
            PartQuantity::zero(),
            Money::byn_minor(1000).unwrap(),
            Money::byn_minor(1500).unwrap(),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, RepairPartError::ZeroQuantity);
    }

    #[test]
    fn repair_part_new_rejects_currency_mismatch() {
        let error = RepairPart::new(
            repair_part_id(),
            repair_id(),
            part_id(),
            PartQuantity::new(1),
            Money::byn_minor(1000).unwrap(),
            Money::usd_minor(1500).unwrap(),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(
            error,
            RepairPartError::CurrencyMismatch {
                cost: Currency::Byn,
                price: Currency::Usd,
            }
        );
    }

    #[test]
    fn repair_part_total_cost_multiplies_unit_cost_by_quantity() {
        let part = repair_part(
            Money::byn_minor(1200).unwrap(),
            Money::byn_minor(1500).unwrap(),
            3,
        );

        let total = part.total_cost().unwrap();

        assert_eq!(total, Money::byn_minor(3600).unwrap());
    }

    #[test]
    fn repair_part_total_price_multiplies_unit_price_by_quantity() {
        let part = repair_part(
            Money::byn_minor(1200).unwrap(),
            Money::byn_minor(1500).unwrap(),
            3,
        );

        let total = part.total_price().unwrap();

        assert_eq!(total, Money::byn_minor(4500).unwrap());
    }

    #[test]
    fn repair_part_profit_can_be_positive() {
        let part = repair_part(
            Money::byn_minor(1200).unwrap(),
            Money::byn_minor(1500).unwrap(),
            3,
        );

        let profit = part.profit().unwrap();

        assert_eq!(profit, SignedMoney::new(900, Currency::Byn));
        assert!(profit.is_positive());
    }

    #[test]
    fn repair_part_profit_can_be_negative() {
        let part = repair_part(
            Money::byn_minor(1500).unwrap(),
            Money::byn_minor(1200).unwrap(),
            3,
        );

        let profit = part.profit().unwrap();

        assert_eq!(profit, SignedMoney::new(-900, Currency::Byn));
        assert!(profit.is_negative());
    }

    #[test]
    fn repair_part_total_cost_returns_money_error_on_overflow() {
        let part = repair_part(
            Money::new(i64::MAX, Currency::Byn).unwrap(),
            Money::new(i64::MAX, Currency::Byn).unwrap(),
            2,
        );

        let error = part.total_cost().unwrap_err();

        assert_eq!(error, RepairPartError::Money(MoneyError::Overflow));
    }
}

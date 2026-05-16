//! Черновики пользовательского ввода.
//!
//! В draft'ах хранятся строки из Telegram, а не domain value objects. Это дает
//! handler'ам возможность показывать экран подтверждения с исходным вводом и
//! выполнять доменную валидацию только в момент сохранения сценария.

use garage_domain::{BookingId, CarId, ClientId, PartId, RepairId};

/// Черновик формы создания клиента.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientDraft {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

/// Черновик формы создания автомобиля.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CarDraft {
    pub client_id: Option<ClientId>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<String>,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
}

impl CarDraft {
    /// Очищает черновик автомобиля до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Черновик формы создания записи на обслуживание.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BookingDraft {
    pub client_id: Option<ClientId>,
    pub car_id: Option<CarId>,
    pub scheduled_at: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

impl BookingDraft {
    /// Очищает черновик записи до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl ClientDraft {
    /// Очищает черновик клиента до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Черновик формы создания складской позиции.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartDraft {
    pub name: Option<String>,
    pub sku: Option<String>,
    pub quantity: Option<String>,
    pub min_quantity: Option<String>,
    pub unit_price: Option<String>,
    pub notes: Option<String>,
}

impl PartDraft {
    /// Очищает черновик складской позиции до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Черновик ручного изменения фактического остатка.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetPartStockDraft {
    pub part_id: Option<PartId>,
}

impl SetPartStockDraft {
    /// Очищает черновик изменения остатка до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Черновик запуска ремонта из записи.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StartRepairDraft {
    pub booking_id: Option<BookingId>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

/// Черновик регистрации оплаты.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordPaymentDraft {
    pub repair_id: Option<RepairId>,
    pub amount: Option<String>,
    pub method: Option<String>,
    pub comment: Option<String>,
}

impl RecordPaymentDraft {
    /// Очищает черновик оплаты до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Черновик списания запчасти в ремонт.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UseRepairPartDraft {
    pub repair_id: Option<RepairId>,
    pub part_id: Option<PartId>,
    pub quantity: Option<String>,
    pub unit_price: Option<String>,
    pub comment: Option<String>,
}

/// Черновик изменения стоимости работ.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetRepairLaborDraft {
    pub repair_id: Option<RepairId>,
}

impl SetRepairLaborDraft {
    /// Очищает черновик стоимости работ до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl UseRepairPartDraft {
    /// Очищает черновик списания запчасти до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl StartRepairDraft {
    /// Очищает черновик запуска ремонта до начального состояния.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

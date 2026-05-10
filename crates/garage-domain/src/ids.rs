//! Типобезопасные идентификаторы доменных сущностей.
//!
//! Все идентификаторы внутри хранят обычный `Uuid`, но наружу отдаются как
//! отдельные типы. Это не дает случайно передать, например, `ClientId` туда,
//! где ожидается `CarId`, хотя технически оба значения представлены UUID.
//!
//! Внутренний UUID намеренно скрыт. В доменной модели лучше работать с
//! конкретным типом идентификатора, а к сырому UUID обращаться только на
//! границах системы: в базе данных, API, логировании и интеграциях.

use uuid::Uuid;

/// Идентификатор клиента.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(Uuid);

impl ClientId {
    /// Создает новый идентификатор для клиента, который рождается внутри домена.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор клиента из UUID, полученного из внешнего слоя.
    ///
    /// Метод не проверяет, существует ли такой клиент и имеет ли текущий сценарий
    /// право с ним работать. Эти проверки принадлежат репозиториям и сервисам
    /// приложения, у которых есть доступ к данным и контексту операции.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для слоев инфраструктуры, не разрушая
    /// типобезопасность внутри доменной модели.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор автомобиля.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CarId(Uuid);

impl CarId {
    /// Создает новый идентификатор для автомобиля, который рождается внутри домена.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор автомобиля из UUID, полученного из внешнего слоя.
    ///
    /// Метод не проверяет, существует ли такой автомобиль и привязан ли он к
    /// нужному клиенту. Эти проверки принадлежат репозиториям и сервисам
    /// приложения, у которых есть доступ к данным и контексту операции.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для слоев инфраструктуры, не разрушая
    /// типобезопасность внутри доменной модели.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CarId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор записи на обслуживание.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookingId(Uuid);

impl BookingId {
    /// Создает новый идентификатор для записи, которая рождается внутри домена.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор записи из UUID, полученного из внешнего слоя.
    ///
    /// Метод не проверяет, существует ли такая запись и находится ли она в
    /// допустимом состоянии для текущей операции. Эти проверки принадлежат
    /// репозиториям и сервисам приложения.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для слоев инфраструктуры, не разрушая
    /// типобезопасность внутри доменной модели.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for BookingId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор поставки запчасти.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartSupplyId(Uuid);

impl PartSupplyId {
    /// Создает новый идентификатор для поставки запчасти.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор поставки из UUID, полученного из внешнего слоя.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для слоев инфраструктуры.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PartSupplyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор запчасти или расходника.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PartId(Uuid);

impl PartId {
    /// Создает новый идентификатор для запчасти, которая рождается внутри домена.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор запчасти из UUID, полученного из внешнего слоя.
    ///
    /// Метод не проверяет, существует ли такая запчасть и доступна ли она для
    /// текущего ремонта. Эти проверки принадлежат репозиториям и сервисам
    /// приложения.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для слоев инфраструктуры, не разрушая
    /// типобезопасность внутри доменной модели.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PartId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор ремонта.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RepairId(Uuid);

impl RepairId {
    /// Создает новый идентификатор для ремонта, который рождается внутри домена.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор ремонта из UUID, полученного из внешнего слоя.
    ///
    /// Метод не проверяет, существует ли такой ремонт и можно ли выполнить над ним
    /// текущую операцию. Эти проверки принадлежат репозиториям и сервисам
    /// приложения, у которых есть доступ к данным и бизнес-контексту.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для слоев инфраструктуры, не разрушая
    /// типобезопасность внутри доменной модели.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for RepairId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор факта использования запчасти в ремонте.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RepairPartId(Uuid);

impl RepairPartId {
    /// Создает новый идентификатор для строки использованной запчасти.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор строки ремонта из UUID внешнего слоя.
    ///
    /// Метод не проверяет, существует ли связанный ремонт или складская позиция.
    /// Эти проверки принадлежат прикладному слою и репозиториям.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для инфраструктуры и интеграций.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for RepairPartId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор отдельной оплаты по ремонту.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaymentId(Uuid);

impl PaymentId {
    /// Создает новый идентификатор оплаты, которая рождается внутри домена.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор оплаты из UUID внешнего слоя.
    ///
    /// Метод не проверяет, существует ли связанный ремонт и была ли оплата уже
    /// учтена в агрегированной сумме ремонта. Эти проверки принадлежат будущему
    /// application-layer сценарию.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для инфраструктуры и интеграций.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for PaymentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Идентификатор исторического движения складского остатка.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StockMovementId(Uuid);

impl StockMovementId {
    /// Создает новый идентификатор движения склада.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Восстанавливает идентификатор движения из UUID внешнего слоя.
    ///
    /// Метод не проверяет, существует ли связанная складская позиция и было ли
    /// движение уже отражено в текущем остатке `Part`. Эти проверки принадлежат
    /// application-layer сценарию.
    pub fn from_uuid(value: Uuid) -> Self {
        Self(value)
    }

    /// Возвращает сырой UUID для инфраструктуры и интеграций.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for StockMovementId {
    fn default() -> Self {
        Self::new()
    }
}

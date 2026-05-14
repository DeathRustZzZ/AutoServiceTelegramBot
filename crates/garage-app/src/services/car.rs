//! Сценарии работы с автомобилями клиента.
//!
//! Автомобиль является отдельным агрегатом, но почти все пользовательские
//! сценарии приходят в форме `client_id + car_id`. Поэтому сервис отвечает за
//! cross-aggregate проверку: машина должна принадлежать указанному клиенту.
//! Сам домен `Car` не ходит в репозитории и не может проверить существование
//! владельца.

use chrono::{DateTime, Utc};
use garage_domain::{
    Car, CarDocumentPhotoRef, CarId, CarMake, CarModel, CarNotes, CarYear, ClientId, LicensePlate,
    Vin,
};

use crate::{AppResult, CarRepository, ClientRepository};

use super::common::{ensure_car_belongs_to_client, require_car, require_client};

/// Application service для автомобилей.
pub struct CarService<Clients, Cars> {
    clients: Clients,
    cars: Cars,
}

impl<Clients, Cars> CarService<Clients, Cars>
where
    Clients: ClientRepository,
    Cars: CarRepository,
{
    /// Создает сервис поверх портов клиентов и автомобилей.
    pub fn new(clients: Clients, cars: Cars) -> Self {
        Self { clients, cars }
    }

    /// Создает автомобиль для существующего клиента.
    ///
    /// Алгоритм:
    /// 1. Проверяем, что клиент существует.
    /// 2. Создаем `Car` через доменную модель.
    /// 3. Сохраняем автомобиль.
    ///
    /// Значения марки, модели, номера, VIN и заметок уже должны быть
    /// проверенными value objects.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_car(
        &self,
        client_id: ClientId,
        make: CarMake,
        model: CarModel,
        year: Option<CarYear>,
        license_plate: Option<LicensePlate>,
        vin: Option<Vin>,
        notes: Option<CarNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Car> {
        require_client(&self.clients, client_id).await?;
        let car = Car::new(
            CarId::new(),
            client_id,
            make,
            model,
            year,
            license_plate,
            vin,
            notes,
            now,
        );
        self.cars.save(&car).await?;
        Ok(car)
    }

    /// Возвращает автомобиль по id или `CarNotFound`.
    pub async fn get_car(&self, car_id: CarId) -> AppResult<Car> {
        require_car(&self.cars, car_id).await
    }

    /// Обновляет идентификационные данные автомобиля.
    ///
    /// В domain слой изменения разделены на методы `update_identity`,
    /// `update_license_plate` и `update_vin`. Здесь они объединены в один
    /// пользовательский сценарий редактирования карточки авто.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_identity(
        &self,
        car_id: CarId,
        make: CarMake,
        model: CarModel,
        year: Option<CarYear>,
        license_plate: Option<LicensePlate>,
        vin: Option<Vin>,
        now: DateTime<Utc>,
    ) -> AppResult<Car> {
        let mut car = require_car(&self.cars, car_id).await?;
        car.update_identity(make, model, year, now)?;
        car.update_license_plate(license_plate, now)?;
        car.update_vin(vin, now)?;
        self.cars.save(&car).await?;
        Ok(car)
    }

    /// Возвращает автомобили клиента.
    ///
    /// Сначала проверяется существование клиента, чтобы UI получил точную
    /// ошибку `ClientNotFound`, а не пустой список для несуществующего клиента.
    pub async fn list_client_cars(&self, client_id: ClientId) -> AppResult<Vec<Car>> {
        require_client(&self.clients, client_id).await?;
        self.cars.list_by_client(client_id).await
    }

    /// Сохраняет ссылку на фото техпаспорта / СТС автомобиля.
    ///
    /// Сервис принимает нейтральный `CarDocumentPhotoRef`: это может быть
    /// Telegram `file_id`, ключ объектного хранилища или URL. App-layer не знает
    /// и не должен знать источник этой строки.
    pub async fn set_registration_document_photo(
        &self,
        client_id: ClientId,
        car_id: CarId,
        photo: CarDocumentPhotoRef,
        now: DateTime<Utc>,
    ) -> AppResult<Car> {
        require_client(&self.clients, client_id).await?;
        let mut car = require_car(&self.cars, car_id).await?;
        ensure_car_belongs_to_client(&car, client_id)?;

        car.set_registration_document_photo(photo, now)?;
        self.cars.save(&car).await?;
        Ok(car)
    }

    /// Удаляет ссылку на фото регистрационного документа автомобиля.
    ///
    /// Проверки клиента и принадлежности машины выполняются до мутации доменной
    /// сущности, поэтому чужую машину нельзя изменить даже при валидном `car_id`.
    pub async fn remove_registration_document_photo(
        &self,
        client_id: ClientId,
        car_id: CarId,
        now: DateTime<Utc>,
    ) -> AppResult<Car> {
        require_client(&self.clients, client_id).await?;
        let mut car = require_car(&self.cars, car_id).await?;
        ensure_car_belongs_to_client(&car, client_id)?;

        car.remove_registration_document_photo(now)?;
        self.cars.save(&car).await?;
        Ok(car)
    }

    /// Архивирует автомобиль без физического удаления.
    pub async fn archive_car(&self, car_id: CarId, now: DateTime<Utc>) -> AppResult<Car> {
        let mut car = require_car(&self.cars, car_id).await?;
        car.archive(now)?;
        self.cars.save(&car).await?;
        Ok(car)
    }

    /// Возвращает автомобиль из архива.
    pub async fn restore_car_from_archive(
        &self,
        car_id: CarId,
        now: DateTime<Utc>,
    ) -> AppResult<Car> {
        let mut car = require_car(&self.cars, car_id).await?;
        car.restore_from_archive(now)?;
        self.cars.save(&car).await?;
        Ok(car)
    }
}

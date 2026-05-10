use chrono::{DateTime, Utc};
use garage_domain::{
    Car, CarId, CarMake, CarModel, CarNotes, CarYear, ClientId, LicensePlate, Vin,
};

use crate::{AppResult, CarRepository, ClientRepository};

use super::common::{require_car, require_client};

/// Use cases for cars.
pub struct CarService<Clients, Cars> {
    clients: Clients,
    cars: Cars,
}

impl<Clients, Cars> CarService<Clients, Cars>
where
    Clients: ClientRepository,
    Cars: CarRepository,
{
    pub fn new(clients: Clients, cars: Cars) -> Self {
        Self { clients, cars }
    }

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

    pub async fn list_client_cars(&self, client_id: ClientId) -> AppResult<Vec<Car>> {
        require_client(&self.clients, client_id).await?;
        self.cars.list_by_client(client_id).await
    }
}

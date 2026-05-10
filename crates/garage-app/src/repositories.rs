use async_trait::async_trait;
use chrono::{DateTime, Utc};
use garage_domain::{
    Booking, BookingId, Car, CarId, Client, ClientId, Part, PartId, PartSupply, PartSupplyId,
    Repair, RepairId,
};
use std::sync::Arc;

use crate::AppResult;

#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>>;
    async fn save(&self, client: &Client) -> AppResult<()>;
}

#[async_trait]
pub trait CarRepository: Send + Sync {
    async fn get(&self, id: CarId) -> AppResult<Option<Car>>;
    async fn save(&self, car: &Car) -> AppResult<()>;
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Car>>;
}

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn get(&self, id: BookingId) -> AppResult<Option<Booking>>;
    async fn save(&self, booking: &Booking) -> AppResult<()>;
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Booking>>;
    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Booking>>;
}

#[async_trait]
pub trait PartRepository: Send + Sync {
    async fn get(&self, id: PartId) -> AppResult<Option<Part>>;
    async fn save(&self, part: &Part) -> AppResult<()>;
    async fn list_low_stock(&self) -> AppResult<Vec<Part>>;
}

#[async_trait]
pub trait PartSupplyRepository: Send + Sync {
    async fn get(&self, id: PartSupplyId) -> AppResult<Option<PartSupply>>;
    async fn save(&self, supply: &PartSupply) -> AppResult<()>;
    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<PartSupply>>;
}

#[async_trait]
pub trait RepairRepository: Send + Sync {
    async fn get(&self, id: RepairId) -> AppResult<Option<Repair>>;
    async fn save(&self, repair: &Repair) -> AppResult<()>;
    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Repair>>;
    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Repair>>;
    async fn list_completed_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Repair>>;
}

#[async_trait]
impl<T> ClientRepository for Arc<T>
where
    T: ClientRepository + ?Sized,
{
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>> {
        (**self).get(id).await
    }

    async fn save(&self, client: &Client) -> AppResult<()> {
        (**self).save(client).await
    }
}

#[async_trait]
impl<T> CarRepository for Arc<T>
where
    T: CarRepository + ?Sized,
{
    async fn get(&self, id: CarId) -> AppResult<Option<Car>> {
        (**self).get(id).await
    }

    async fn save(&self, car: &Car) -> AppResult<()> {
        (**self).save(car).await
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Car>> {
        (**self).list_by_client(client_id).await
    }
}

#[async_trait]
impl<T> BookingRepository for Arc<T>
where
    T: BookingRepository + ?Sized,
{
    async fn get(&self, id: BookingId) -> AppResult<Option<Booking>> {
        (**self).get(id).await
    }

    async fn save(&self, booking: &Booking) -> AppResult<()> {
        (**self).save(booking).await
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Booking>> {
        (**self).list_by_client(client_id).await
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Booking>> {
        (**self).list_by_car(car_id).await
    }
}

#[async_trait]
impl<T> PartRepository for Arc<T>
where
    T: PartRepository + ?Sized,
{
    async fn get(&self, id: PartId) -> AppResult<Option<Part>> {
        (**self).get(id).await
    }

    async fn save(&self, part: &Part) -> AppResult<()> {
        (**self).save(part).await
    }

    async fn list_low_stock(&self) -> AppResult<Vec<Part>> {
        (**self).list_low_stock().await
    }
}

#[async_trait]
impl<T> PartSupplyRepository for Arc<T>
where
    T: PartSupplyRepository + ?Sized,
{
    async fn get(&self, id: PartSupplyId) -> AppResult<Option<PartSupply>> {
        (**self).get(id).await
    }

    async fn save(&self, supply: &PartSupply) -> AppResult<()> {
        (**self).save(supply).await
    }

    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<PartSupply>> {
        (**self).list_by_part(part_id).await
    }
}

#[async_trait]
impl<T> RepairRepository for Arc<T>
where
    T: RepairRepository + ?Sized,
{
    async fn get(&self, id: RepairId) -> AppResult<Option<Repair>> {
        (**self).get(id).await
    }

    async fn save(&self, repair: &Repair) -> AppResult<()> {
        (**self).save(repair).await
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Repair>> {
        (**self).list_by_client(client_id).await
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Repair>> {
        (**self).list_by_car(car_id).await
    }

    async fn list_completed_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Repair>> {
        (**self).list_completed_between(from, to).await
    }
}

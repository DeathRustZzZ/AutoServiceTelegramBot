use garage_app::AppResult;
use garage_domain::{
    Car, CarDocumentPhotoRef, CarId, CarMake, CarModel, CarNotes, CarYear, ClientId, LicensePlate,
    Vin,
};

use crate::mappers::{invalid_row_error, map_car_status};
use crate::models::CarRow;

pub fn to_domain(row: &CarRow) -> AppResult<Car> {
    let year = row
        .year
        .map(|year| {
            u16::try_from(year)
                .map_err(|_| invalid_row_error("car", "year", year))
                .and_then(|year| CarYear::new(year).map_err(Into::into))
        })
        .transpose()?;

    let registration_document_photo = row
        .registration_document_photo_ref
        .as_deref()
        .map(CarDocumentPhotoRef::new)
        .transpose()
        .map_err(|error| invalid_row_error("car", "registration_document_photo_ref", error))?;

    Car::restore_with_registration_document_photo(
        CarId::from_uuid(row.id),
        ClientId::from_uuid(row.client_id),
        CarMake::parse(&row.make)?,
        CarModel::parse(&row.model)?,
        year,
        row.license_plate
            .as_deref()
            .map(LicensePlate::parse)
            .transpose()?
            .flatten(),
        row.vin.as_deref().map(Vin::parse).transpose()?.flatten(),
        row.notes
            .as_deref()
            .map(CarNotes::parse)
            .transpose()?
            .flatten(),
        registration_document_photo,
        map_car_status(&row.status)?,
        row.created_at,
        row.updated_at,
    )
    .map_err(Into::into)
}

use garage_app::AppResult;
use garage_domain::{Client, ClientId, ClientName, ClientNotes, PhoneNumber};

use crate::mappers::map_client_status;
use crate::models::ClientRow;

pub fn to_domain(row: &ClientRow) -> AppResult<Client> {
    Client::restore(
        ClientId::from_uuid(row.id),
        ClientName::parse(&row.name)?,
        PhoneNumber::parse(&row.phone)?,
        row.notes
            .as_deref()
            .map(ClientNotes::parse)
            .transpose()?
            .flatten(),
        map_client_status(&row.status)?,
        row.created_at,
        row.updated_at,
    )
    .map_err(Into::into)
}

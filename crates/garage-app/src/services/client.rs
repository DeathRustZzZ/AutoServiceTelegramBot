use chrono::{DateTime, Utc};
use garage_domain::{Client, ClientId, ClientName, ClientNotes, PhoneNumber};

use crate::{AppResult, ClientRepository};

use super::common::require_client;

/// Use cases for clients.
pub struct ClientService<R> {
    clients: R,
}

impl<R> ClientService<R>
where
    R: ClientRepository,
{
    pub fn new(clients: R) -> Self {
        Self { clients }
    }

    pub async fn create_client(
        &self,
        name: ClientName,
        phone: PhoneNumber,
        notes: Option<ClientNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let client = Client::new(ClientId::new(), name, phone, notes, now);
        self.clients.save(&client).await?;
        Ok(client)
    }

    pub async fn rename_client(
        &self,
        client_id: ClientId,
        name: ClientName,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.rename(name, now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }

    pub async fn change_phone(
        &self,
        client_id: ClientId,
        phone: PhoneNumber,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.change_phone(phone, now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }

    pub async fn update_notes(
        &self,
        client_id: ClientId,
        notes: Option<ClientNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.update_notes(notes, now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }
}

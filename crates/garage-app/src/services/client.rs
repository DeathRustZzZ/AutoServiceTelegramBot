//! Сценарии работы с клиентами.
//!
//! Клиент - самостоятельный агрегат. Этот сервис не загружает автомобили,
//! записи или ремонты вместе с клиентом: связанные списки принадлежат отдельным
//! use case'ам. Такой подход удерживает сценарии маленькими и не превращает
//! `ClientService` в универсальный фасад над всей системой.

use chrono::{DateTime, Utc};
use garage_domain::{Client, ClientId, ClientName, ClientNotes, PhoneNumber};

use crate::{AppResult, ClientRepository};

use super::common::require_client;

/// Application service для клиентов.
///
/// Сервис зависит только от `ClientRepository`. Это важная граница: создание и
/// редактирование клиента не требует знания о PostgreSQL, Telegram или других
/// агрегатах.
pub struct ClientService<R> {
    clients: R,
}

impl<R> ClientService<R>
where
    R: ClientRepository,
{
    /// Создает сервис поверх repository port.
    pub fn new(clients: R) -> Self {
        Self { clients }
    }

    /// Создает клиента и сохраняет его.
    ///
    /// Метод принимает уже проверенные domain value objects. Разбор строк из
    /// Telegram-команд должен происходить выше, до входа в application layer.
    /// Здесь остается только orchestration: создать агрегат и сохранить его.
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

    /// Переименовывает существующего клиента.
    ///
    /// Алгоритм:
    /// 1. Загружаем клиента или возвращаем `ClientNotFound`.
    /// 2. Передаем изменение в домен через `Client::rename`.
    /// 3. Сохраняем агрегат целиком.
    ///
    /// Если домен отклонит timestamp, `save` не будет вызван.
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

    /// Меняет телефон клиента.
    ///
    /// Нормализацию белорусского номера выполняет `PhoneNumber`; сервис не
    /// должен повторять эти правила.
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

    /// Обновляет заметки клиента.
    ///
    /// Пустая строка должна быть превращена в `None` через `ClientNotes::parse`
    /// до вызова сервиса.
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

    /// Архивирует клиента без физического удаления.
    pub async fn archive_client(
        &self,
        client_id: ClientId,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.archive(now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }

    /// Возвращает клиента из архива.
    pub async fn restore_client_from_archive(
        &self,
        client_id: ClientId,
        now: DateTime<Utc>,
    ) -> AppResult<Client> {
        let mut client = require_client(&self.clients, client_id).await?;
        client.restore_from_archive(now)?;
        self.clients.save(&client).await?;
        Ok(client)
    }
}

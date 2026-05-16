//! Handler'ы записей на обслуживание.
//!
//! Модуль ведет пользователя через выбор клиента, автомобиля, даты и причины
//! визита. Время в UI вводится локально, а в `garage-app` передается UTC.

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use garage_app::{AppError, BookingDetails};
use garage_domain::{
    Booking, BookingId, BookingNotes, BookingReason, Car, CarId, Client, ClientId,
};
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::keyboards;
use crate::messages;
use crate::state::{AddBookingStep, DialogState, HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};

const SEARCH_LIMIT: u32 = 5;

/// Показывает меню записей и сбрасывает активный диалог.
pub async fn show_menu(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.reset_dialog();

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(messages::bookings::menu(), keyboards::bookings::menu()),
    )
    .await
}

/// Показывает записи на текущий локальный день автосервиса.
pub async fn show_today(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
) -> HandlerResult {
    let (from, to) = local_day_bounds(Utc::now(), container.timezone_offset_hours(), 0);
    show_period_list(
        bot,
        dialogue,
        chat_id,
        container,
        session,
        PeriodWindow {
            from,
            to,
            kind: PeriodKind::Today,
        },
    )
    .await
}

/// Показывает записи на следующий локальный день автосервиса.
pub async fn show_tomorrow(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
) -> HandlerResult {
    let (from, to) = local_day_bounds(Utc::now(), container.timezone_offset_hours(), 1);
    show_period_list(
        bot,
        dialogue,
        chat_id,
        container,
        session,
        PeriodWindow {
            from,
            to,
            kind: PeriodKind::Tomorrow,
        },
    )
    .await
}

/// Показывает список записей за уже рассчитанное UTC-окно.
pub async fn show_period_list(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    window: PeriodWindow,
) -> HandlerResult {
    let items = match container
        .booking_service()
        .list_booking_details_between(window.from, window.to)
        .await
    {
        Ok(items) => items,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    let text = match (window.kind, items.is_empty()) {
        (PeriodKind::Today, true) => messages::bookings::empty_today().to_string(),
        (PeriodKind::Tomorrow, true) => messages::bookings::empty_tomorrow().to_string(),
        (PeriodKind::Today, false) => {
            messages::bookings::list_today(&items, container.timezone_offset_hours())
        }
        (PeriodKind::Tomorrow, false) => {
            messages::bookings::list_tomorrow(&items, container.timezone_offset_hours())
        }
    };

    let keyboard = if items.is_empty() {
        keyboards::bookings::empty_list()
    } else {
        keyboards::bookings::list(&items)
    };

    render_screen(bot, dialogue, chat_id, session, Screen::new(text, keyboard)).await
}

/// Начинает форму создания записи с поиска клиента.
pub async fn begin_add(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    mut session: SessionData,
) -> HandlerResult {
    session.booking_draft.reset();
    session.dialog = DialogState::AddBooking(AddBookingStep::AwaitingClientSearch);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::bookings::ask_client_query(),
            keyboards::bookings::back_to_menu(),
        ),
    )
    .await
}

/// Обрабатывает текстовые шаги формы создания записи.
pub async fn handle_add_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    step: AddBookingStep,
    text: String,
) -> HandlerResult {
    match step {
        AddBookingStep::AwaitingClientSearch => {
            let query = text.trim().to_string();
            if query.is_empty() {
                return render_screen(
                    &bot,
                    &dialogue,
                    msg.chat.id,
                    session,
                    Screen::new(
                        messages::bookings::ask_client_query(),
                        keyboards::bookings::back_to_menu(),
                    ),
                )
                .await;
            }

            let clients = match container
                .client_service()
                .search_clients(&query, SEARCH_LIMIT, 0)
                .await
            {
                Ok(clients) => clients,
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            let text = if clients.is_empty() {
                messages::bookings::no_client_results(&query)
            } else {
                messages::bookings::client_search_results(&query, &clients)
            };

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(text, keyboards::bookings::client_search_results(&clients)),
            )
            .await
        }
        AddBookingStep::AwaitingCarSelection => {
            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(
                    "Выберите автомобиль кнопкой.",
                    keyboards::bookings::back_to_menu(),
                ),
            )
            .await
        }
        AddBookingStep::AwaitingDateTime => {
            session.booking_draft.scheduled_at = Some(text);
            session.dialog = DialogState::AddBooking(AddBookingStep::AwaitingReason);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(
                    messages::bookings::ask_reason(),
                    keyboards::bookings::back_to_menu(),
                ),
            )
            .await
        }
        AddBookingStep::AwaitingReason => {
            session.booking_draft.reason = Some(text);
            session.dialog = DialogState::AddBooking(AddBookingStep::AwaitingNotes);

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session,
                Screen::new(
                    messages::bookings::ask_notes(),
                    keyboards::bookings::back_to_menu(),
                ),
            )
            .await
        }
        AddBookingStep::AwaitingNotes => {
            session.booking_draft.notes = optional_string(text);
            session.dialog = DialogState::AddBooking(AddBookingStep::Confirm);

            let (client, car) = match load_draft_client_car(&container, &session).await {
                Ok(Some(value)) => value,
                Ok(None) => {
                    return render_broken_draft(&bot, &dialogue, msg.chat.id, session).await
                }
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session.clone(),
                Screen::new(
                    messages::bookings::confirm(
                        &client,
                        &car,
                        &session.booking_draft,
                        container.timezone_offset_hours(),
                    ),
                    keyboards::bookings::confirm(),
                ),
            )
            .await
        }
        AddBookingStep::Confirm => {
            let (client, car) = match load_draft_client_car(&container, &session).await {
                Ok(Some(value)) => value,
                Ok(None) => {
                    return render_broken_draft(&bot, &dialogue, msg.chat.id, session).await
                }
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            render_screen(
                &bot,
                &dialogue,
                msg.chat.id,
                session.clone(),
                Screen::new(
                    messages::bookings::confirm(
                        &client,
                        &car,
                        &session.booking_draft,
                        container.timezone_offset_hours(),
                    ),
                    keyboards::bookings::confirm(),
                ),
            )
            .await
        }
    }
}

/// Выбирает клиента для новой записи и переводит форму к выбору автомобиля.
pub async fn select_client(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    client_id: ClientId,
) -> HandlerResult {
    let client = match container.client_service().get_client(client_id).await {
        Ok(client) => client,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };
    let cars = match container.car_service().list_client_cars(client_id).await {
        Ok(cars) => cars,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.booking_draft.client_id = Some(client_id);
    session.booking_draft.car_id = None;
    session.dialog = DialogState::AddBooking(AddBookingStep::AwaitingCarSelection);

    let screen = if cars.is_empty() {
        Screen::new(
            messages::bookings::no_cars_for_client(&client),
            keyboards::bookings::no_cars(client_id),
        )
    } else {
        Screen::new(
            messages::bookings::select_car(&client, &cars),
            keyboards::bookings::select_car(&cars),
        )
    };

    render_screen(bot, dialogue, chat_id, session, screen).await
}

/// Выбирает автомобиль для новой записи и переводит форму к вводу даты.
pub async fn select_car(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    car_id: CarId,
) -> HandlerResult {
    let Some(client_id) = session.booking_draft.client_id else {
        return render_broken_draft(bot, dialogue, chat_id, session).await;
    };
    let car = match container.car_service().get_car(car_id).await {
        Ok(car) => car,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };
    if car.client_id() != client_id {
        return render_screen(
            bot,
            dialogue,
            chat_id,
            session,
            Screen::new(
                messages::errors::app_error(&AppError::CarDoesNotBelongToClient {
                    car_id,
                    client_id,
                }),
                keyboards::bookings::back_to_menu(),
            ),
        )
        .await;
    }

    session.booking_draft.car_id = Some(car_id);
    session.dialog = DialogState::AddBooking(AddBookingStep::AwaitingDateTime);

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::bookings::ask_datetime(),
            keyboards::bookings::back_to_menu(),
        ),
    )
    .await
}

/// Подтверждает создание записи и сохраняет ее через `BookingService`.
pub async fn confirm(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let result = schedule_booking(&container, &session).await;
    let booking = match result {
        Ok(booking) => booking,
        Err(error) => {
            return render_screen(
                bot,
                dialogue,
                chat_id,
                session,
                Screen::new(error, keyboards::bookings::back_to_menu()),
            )
            .await;
        }
    };
    let details = match details_for_booking(&container, booking.id()).await {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();
    render_booking_card(bot, dialogue, chat_id, session, container, details, true).await
}

/// Показывает карточку записи.
pub async fn show_card(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
    booking_id: BookingId,
) -> HandlerResult {
    let details = match details_for_booking(&container, booking_id).await {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    render_booking_card(bot, dialogue, chat_id, session, container, details, false).await
}

/// Отмечает запись выполненной.
pub async fn complete(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
    booking_id: BookingId,
) -> HandlerResult {
    let booking = match container
        .booking_service()
        .complete_booking(booking_id, Utc::now())
        .await
    {
        Ok(booking) => booking,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };
    let details = match details_for_booking(&container, booking.id()).await {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    render_booking_card(bot, dialogue, chat_id, session, container, details, false).await
}

/// Отменяет запись.
pub async fn cancel(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
    booking_id: BookingId,
) -> HandlerResult {
    let booking = match container
        .booking_service()
        .cancel_booking(booking_id, Utc::now())
        .await
    {
        Ok(booking) => booking,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };
    let details = match details_for_booking(&container, booking.id()).await {
        Ok(details) => details,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    render_booking_card(bot, dialogue, chat_id, session, container, details, false).await
}

/// Создает запись из текущего черновика session state.
async fn schedule_booking(
    container: &AppContainer,
    session: &SessionData,
) -> Result<Booking, String> {
    let draft = &session.booking_draft;
    let client_id = draft
        .client_id
        .ok_or_else(|| "Выберите клиента для записи.".to_string())?;
    let car_id = draft
        .car_id
        .ok_or_else(|| "Выберите автомобиль для записи.".to_string())?;
    let scheduled_at = parse_local_datetime_to_utc(
        draft
            .scheduled_at
            .as_deref()
            .ok_or_else(|| messages::bookings::invalid_datetime().to_string())?,
        container.timezone_offset_hours(),
    )?;
    let reason = BookingReason::parse(
        draft
            .reason
            .as_deref()
            .ok_or_else(|| "Введите причину обращения.".to_string())?,
    )
    .map_err(|error| crate::handlers::errors::app_error_message(&AppError::Booking(error)))?;
    let notes = match draft.notes.as_deref() {
        Some(notes) => BookingNotes::parse(notes).map_err(|error| {
            crate::handlers::errors::app_error_message(&AppError::Booking(error))
        })?,
        None => None,
    };

    container
        .booking_service()
        .schedule_booking(client_id, car_id, scheduled_at, reason, notes, Utc::now())
        .await
        .map_err(|error| crate::handlers::errors::app_error_message(&error))
}

/// Загружает детальную карточку созданной или измененной записи.
async fn details_for_booking(
    container: &AppContainer,
    booking_id: BookingId,
) -> Result<BookingDetails, AppError> {
    container
        .booking_service()
        .get_booking_details(booking_id)
        .await
}

/// Загружает клиента и автомобиль, выбранные в черновике записи.
async fn load_draft_client_car(
    container: &AppContainer,
    session: &SessionData,
) -> Result<Option<(Client, Car)>, AppError> {
    let Some(client_id) = session.booking_draft.client_id else {
        return Ok(None);
    };
    let Some(car_id) = session.booking_draft.car_id else {
        return Ok(None);
    };

    let client = container.client_service().get_client(client_id).await?;
    let car = container.car_service().get_car(car_id).await?;

    Ok(Some((client, car)))
}

/// Отрисовывает карточку записи с актуальными деталями.
async fn render_booking_card(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    session: SessionData,
    container: AppContainer,
    details: BookingDetails,
    created: bool,
) -> HandlerResult {
    let text = if created {
        messages::bookings::created_card(
            &details.booking,
            &details.client,
            &details.car,
            container.timezone_offset_hours(),
        )
    } else {
        messages::bookings::booking_card(
            &details.booking,
            &details.client,
            &details.car,
            container.timezone_offset_hours(),
        )
    };

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(text, keyboards::bookings::booking_card(&details.booking)),
    )
    .await
}

/// Показывает прикладную ошибку на экране записи.
async fn render_app_error(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    session: SessionData,
    error: &AppError,
) -> HandlerResult {
    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            crate::handlers::errors::app_error_message(error),
            keyboards::bookings::back_to_menu(),
        ),
    )
    .await
}

/// Показывает ошибку устаревшего или поврежденного черновика записи.
async fn render_broken_draft(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    session: SessionData,
) -> HandlerResult {
    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            "Данные записи устарели. Начните создание записи заново.",
            keyboards::bookings::back_to_menu(),
        ),
    )
    .await
}

/// Разбирает локальную дату-время пользователя и переводит ее в UTC.
fn parse_local_datetime_to_utc(input: &str, offset_hours: i32) -> Result<DateTime<Utc>, String> {
    let local = NaiveDateTime::parse_from_str(input.trim(), "%d.%m.%Y %H:%M")
        .map_err(|_| messages::bookings::invalid_datetime().to_string())?;
    let utc_naive = local - Duration::hours(i64::from(offset_hours));
    let utc = DateTime::from_naive_utc_and_offset(utc_naive, Utc);

    if utc <= Utc::now() {
        return Err(messages::bookings::past_datetime().to_string());
    }

    Ok(utc)
}

/// Возвращает UTC-границы локального дня автосервиса.
fn local_day_bounds(
    now: DateTime<Utc>,
    offset_hours: i32,
    days_from_today: i64,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let local_now = now + Duration::hours(i64::from(offset_hours));
    let local_start = local_now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("valid local midnight")
        + Duration::days(days_from_today);
    let utc_start = local_start - Duration::hours(i64::from(offset_hours));
    let utc_end = utc_start + Duration::days(1);

    (
        DateTime::from_naive_utc_and_offset(utc_start, Utc),
        DateTime::from_naive_utc_and_offset(utc_end, Utc),
    )
}

/// Превращает пользовательский `-` или пустую строку в `None`.
fn optional_string(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && value != "-").then(|| value.to_string())
}

#[derive(Debug, Clone, Copy)]
pub enum PeriodKind {
    Today,
    Tomorrow,
}

#[derive(Debug, Clone, Copy)]
pub struct PeriodWindow {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
    kind: PeriodKind,
}

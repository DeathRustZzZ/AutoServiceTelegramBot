use chrono::Utc;
use garage_app::AppError;
use garage_domain::{Car, CarId, CarMake, CarModel, CarYear, Client, ClientId, LicensePlate, Vin};
use teloxide::prelude::*;

use crate::container::AppContainer;
use crate::keyboards;
use crate::messages;
use crate::state::{AddCarStep, DialogState, HandlerResult, SessionData, UserDialogue};
use crate::ui::render::{render_screen, Screen};
use crate::ui::reply_preset::set_reply_keyboard_silent;

pub async fn show_client_cars(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    client_id: ClientId,
) -> HandlerResult {
    let client = match load_client(&container, client_id).await {
        Ok(client) => client,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };
    let cars = match container.car_service().list_client_cars(client_id).await {
        Ok(cars) => cars,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();

    let screen = if cars.is_empty() {
        Screen::new(
            messages::cars::empty_client_cars(&client),
            keyboards::cars::empty_client_cars(client_id),
        )
    } else {
        Screen::new(
            messages::cars::list_client_cars(&client, &cars),
            keyboards::cars::client_cars(client_id, &cars),
        )
    };

    render_screen(bot, dialogue, chat_id, session, screen).await
}

pub async fn begin_add(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
    client_id: ClientId,
) -> HandlerResult {
    if let Err(error) = load_client(&container, client_id).await {
        return render_app_error(bot, dialogue, chat_id, session, &error).await;
    }

    session.car_draft.reset();
    session.car_draft.client_id = Some(client_id);
    session.dialog = DialogState::AddCar(AddCarStep::AwaitingMake);
    set_reply_keyboard_silent(bot, chat_id, keyboards::reply::dialog_navigation()).await;

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::cars::ask_make(),
            keyboards::cars::add_car_back_to_client(client_id),
        ),
    )
    .await
}

pub async fn handle_add_text(
    bot: Bot,
    dialogue: UserDialogue,
    msg: Message,
    container: AppContainer,
    mut session: SessionData,
    step: AddCarStep,
    text: String,
) -> HandlerResult {
    let Some(client_id) = session.car_draft.client_id else {
        return render_broken_draft(&bot, &dialogue, msg.chat.id, session).await;
    };

    let screen = match step {
        AddCarStep::AwaitingMake => {
            session.car_draft.make = Some(text);
            session.dialog = DialogState::AddCar(AddCarStep::AwaitingModel);

            Screen::new(
                messages::cars::ask_model(),
                keyboards::cars::add_car_back_to_client(client_id),
            )
        }
        AddCarStep::AwaitingModel => {
            session.car_draft.model = Some(text);
            session.dialog = DialogState::AddCar(AddCarStep::AwaitingYear);

            Screen::new(
                messages::cars::ask_year(),
                keyboards::cars::add_car_back_to_client(client_id),
            )
        }
        AddCarStep::AwaitingYear => {
            session.car_draft.year = optional_string(text);
            session.dialog = DialogState::AddCar(AddCarStep::AwaitingLicensePlate);

            Screen::new(
                messages::cars::ask_license_plate(),
                keyboards::cars::add_car_back_to_client(client_id),
            )
        }
        AddCarStep::AwaitingLicensePlate => {
            session.car_draft.license_plate = optional_string(text);
            session.dialog = DialogState::AddCar(AddCarStep::AwaitingVin);

            Screen::new(
                messages::cars::ask_vin(),
                keyboards::cars::add_car_back_to_client(client_id),
            )
        }
        AddCarStep::AwaitingVin => {
            session.car_draft.vin = optional_string(text);
            session.dialog = DialogState::AddCar(AddCarStep::Confirm);

            let client = match load_client(&container, client_id).await {
                Ok(client) => client,
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            Screen::new(
                messages::cars::confirm(&client, &session.car_draft),
                keyboards::cars::add_car_confirm(client_id),
            )
        }
        AddCarStep::Confirm => {
            let client = match load_client(&container, client_id).await {
                Ok(client) => client,
                Err(error) => {
                    return render_app_error(&bot, &dialogue, msg.chat.id, session, &error).await;
                }
            };

            Screen::new(
                messages::cars::confirm(&client, &session.car_draft),
                keyboards::cars::add_car_confirm(client_id),
            )
        }
    };

    render_screen(&bot, &dialogue, msg.chat.id, session, screen).await
}

pub async fn confirm(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    mut session: SessionData,
) -> HandlerResult {
    let Some(client_id) = session.car_draft.client_id else {
        return render_broken_draft(bot, dialogue, chat_id, session).await;
    };

    let result = create_car(container.clone(), &session).await;
    let car = match result {
        Ok(car) => car,
        Err(error) => {
            return render_screen(
                bot,
                dialogue,
                chat_id,
                session,
                Screen::new(error, keyboards::cars::add_car_back_to_client(client_id)),
            )
            .await;
        }
    };

    let client = match load_client(&container, car.client_id()).await {
        Ok(client) => client,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    session.reset_dialog();
    set_reply_keyboard_silent(bot, chat_id, keyboards::reply::clients_navigation()).await;

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::cars::created_card(&car, &client),
            keyboards::cars::car_card(client.id()),
        ),
    )
    .await
}

pub async fn show_card(
    bot: &Bot,
    dialogue: &UserDialogue,
    chat_id: ChatId,
    container: AppContainer,
    session: SessionData,
    car_id: CarId,
) -> HandlerResult {
    let car = match container.car_service().get_car(car_id).await {
        Ok(car) => car,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };
    let client = match load_client(&container, car.client_id()).await {
        Ok(client) => client,
        Err(error) => return render_app_error(bot, dialogue, chat_id, session, &error).await,
    };

    render_screen(
        bot,
        dialogue,
        chat_id,
        session,
        Screen::new(
            messages::cars::car_card(&car, &client),
            keyboards::cars::car_card(client.id()),
        ),
    )
    .await
}

async fn create_car(container: AppContainer, session: &SessionData) -> Result<Car, String> {
    let draft = &session.car_draft;
    let client_id = draft
        .client_id
        .ok_or_else(|| messages::cars::missing_draft().to_string())?;
    let make = CarMake::parse(required(
        draft.make.as_deref(),
        "Введите марку автомобиля.",
    )?)
    .map_err(|error| crate::handlers::errors::app_error_message(&AppError::Car(error)))?;
    let model = CarModel::parse(required(
        draft.model.as_deref(),
        "Введите модель автомобиля.",
    )?)
    .map_err(|error| crate::handlers::errors::app_error_message(&AppError::Car(error)))?;
    let year = parse_year(draft.year.as_deref())?;
    let license_plate = parse_license_plate(draft.license_plate.as_deref())?;
    let vin = parse_vin(draft.vin.as_deref())?;

    container
        .car_service()
        .create_car(
            client_id,
            make,
            model,
            year,
            license_plate,
            vin,
            None,
            Utc::now(),
        )
        .await
        .map_err(|error| crate::handlers::errors::app_error_message(&error))
}

async fn load_client(container: &AppContainer, client_id: ClientId) -> Result<Client, AppError> {
    container.client_service().get_client(client_id).await
}

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
            crate::keyboards::clients::clients_menu(),
        ),
    )
    .await
}

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
            messages::cars::missing_draft(),
            crate::keyboards::clients::clients_menu(),
        ),
    )
    .await
}

fn required<'a>(value: Option<&'a str>, message: &'static str) -> Result<&'a str, String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| message.to_string())
}

fn parse_year(value: Option<&str>) -> Result<Option<CarYear>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let year = value
        .parse::<u16>()
        .map_err(|_| messages::cars::invalid_year().to_string())?;

    CarYear::new(year)
        .map(Some)
        .map_err(|error| crate::handlers::errors::app_error_message(&AppError::Car(error)))
}

fn parse_license_plate(value: Option<&str>) -> Result<Option<LicensePlate>, String> {
    match value {
        Some(value) => LicensePlate::parse(value)
            .map_err(|error| crate::handlers::errors::app_error_message(&AppError::Car(error))),
        None => Ok(None),
    }
}

fn parse_vin(value: Option<&str>) -> Result<Option<Vin>, String> {
    match value {
        Some(value) => Vin::parse(value)
            .map_err(|error| crate::handlers::errors::app_error_message(&AppError::Car(error))),
        None => Ok(None),
    }
}

fn optional_string(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty() && value != "-").then(|| value.to_string())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DialogState {
    #[default]
    Idle,
    AddClient(AddClientStep),
    AddCar(AddCarStep),
    SearchClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddClientStep {
    AwaitingName,
    AwaitingPhone,
    AwaitingNotes,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddCarStep {
    AwaitingMake,
    AwaitingModel,
    AwaitingYear,
    AwaitingLicensePlate,
    AwaitingVin,
    Confirm,
}

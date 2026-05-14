#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DialogState {
    #[default]
    Idle,
    AddClient(AddClientStep),
    AddCar(AddCarStep),
    AddBooking(AddBookingStep),
    AddPart(AddPartStep),
    StartRepair(StartRepairStep),
    SearchClient,
    SearchPart,
    SetPartStock(SetPartStockStep),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddBookingStep {
    AwaitingClientSearch,
    AwaitingCarSelection,
    AwaitingDateTime,
    AwaitingReason,
    AwaitingNotes,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddPartStep {
    AwaitingName,
    AwaitingSku,
    AwaitingQuantity,
    AwaitingMinQuantity,
    AwaitingUnitPrice,
    AwaitingNotes,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPartStockStep {
    AwaitingQuantity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartRepairStep {
    AwaitingDescription,
    AwaitingNotes,
    Confirm,
}

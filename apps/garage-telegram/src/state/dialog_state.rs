#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DialogState {
    #[default]
    Idle,
    AddClient(AddClientStep),
    SearchClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddClientStep {
    AwaitingName,
    AwaitingPhone,
    AwaitingNotes,
    Confirm,
}

use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("instruction contained invalid value: {0}")]
    InstructionError(String),
    #[error("journal entry does not balance")]
    JournalBalanceError,
}

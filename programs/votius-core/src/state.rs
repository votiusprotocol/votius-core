use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ExperimentStatus {
    Active,
    Completed,
}

#[account]
#[derive(InitSpace)]
pub struct Experiment {
    pub authority: Pubkey,
    pub experiment_id: u64,
    pub event_count: u64,
    pub status: ExperimentStatus,
    pub created_at: i64,
}
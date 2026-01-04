use anchor_lang::prelude::*;

declare_id!("Cv9dh2aKWosf9nh7Qks2hkTurnLwTi4tX9XMqYY4d9oL");

#[program]
pub mod votius_core {
    use super::*;

    pub fn initilize_experiment(
        ctx: Context<InitializeExperiment>,
        experiment_id: u64,
    ) -> Result<()> {
        let experiment = &mut ctx.accounts.experiment;
        experiment.experiment_id = experiment_id;
        experiment.authority = ctx.accounts.authority.key();
        experiment.status = ExperimentStatus::Active;
        experiment.created_at = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn record_event(ctx: Context<RecordEvent>, event_hash: [u8; 32]) -> Result<()> {
        let experiment = &mut ctx.accounts.experiment;

        require!(
            ctx.accounts.authority.key() == experiment.authority,
            VotiusError::Unauthorized
        );
        experiment.event_count += 1;

        emit!(ExperimentEvent {
            experiment: experiment.key(),
            index: experiment.event_count,
            hash: event_hash,
            timestamp: Clock::get()?.unix_timestamp,
            post_completion: experiment.status == ExperimentStatus::Completed,
        });

        Ok(())
    }

    pub fn complete_experiment(ctx: Context<CompleteExperiment>) -> Result<()> {
        let experiment = &mut ctx.accounts.experiment;

        require!(
            ctx.accounts.authority.key() == experiment.authority,
            VotiusError::Unauthorized
        );
        require!(
            experiment.status == ExperimentStatus::Active,
            VotiusError::AlreadyCompleted
        );
        
        experiment.status = ExperimentStatus::Completed;
        emit!(ExperimentCompletedEvent {
            experiment: experiment.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(experiment_id: u64)]
pub struct InitializeExperiment<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + Experiment::INIT_SPACE,
        seeds = [
            authority.key().as_ref(),
            experiment_id.to_le_bytes().as_ref()],
        bump
    )]
    pub experiment: Account<'info, Experiment>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
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

#[derive(Accounts)]
pub struct RecordEvent<'info> {
    pub authority: Signer<'info>,

    #[account(
        mut,
        has_one = authority @ VotiusError::Unauthorized,
    )]
    pub experiment: Account<'info, Experiment>,
}

#[event]
pub struct ExperimentEvent {
    pub experiment: Pubkey,
    pub index: u64,
    pub hash: [u8; 32],
    pub timestamp: i64,
    pub post_completion: bool,
}

#[derive(Accounts)]
pub struct CompleteExperiment<'info> {
    #[account(mut)]
    pub experiment: Account<'info, Experiment>,

    pub authority: Signer<'info>,
}

#[event]
pub struct ExperimentCompletedEvent {
    pub experiment: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum VotiusError {
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Experiment already completed")]
    AlreadyCompleted,
}

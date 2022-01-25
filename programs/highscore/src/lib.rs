use anchor_lang::prelude::*;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

const MAX_SCORES: usize = 5;

#[program]
pub mod highscore {
use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }

    pub fn submit_score(ctx: Context<SubmitScore>, submission: u64) -> ProgramResult {
        
        let highscore = &mut ctx.accounts.highscore.load_mut()?;

        // Check if submission is viable
        let lowest_entry = &highscore.scores[highscore.scores.len() - 1];
        if lowest_entry.amount < submission
        {
            // Check if the submission owner is already on scoreboard
            let mut updated_existing_score = false;

            for entry_index in 0..highscore.scores.len() {

                // Simply replace score when owner is already on scoreboard and score is higher
                let entry = &highscore.scores[entry_index];
                if *ctx.accounts.user.owner == entry.owner
                {
                    highscore.scores[entry_index].amount = if submission > entry.amount { submission } else {entry.amount};
                    updated_existing_score = true;
                    break;
                }
            }
            
            // New player appears on scoreboard overwrite last/lowest entry
            if !updated_existing_score
            {
                highscore.scores[0] = Score{amount: submission, owner: *ctx.accounts.user.owner};
            }

            // Sort values by score on the board, allows for easy subsequent overwrites.
            highscore.scores.sort_by_key(|k| k.amount);
        }
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(zero)]
    highscore: AccountLoader<'info, Highscore>,
}

#[derive(Accounts)]
pub struct SubmitScore<'info> {
    #[account(zero)]
    pub highscore: AccountLoader<'info, Highscore>,
    #[account(mut)]
    pub user: Signer<'info>,
}

#[account(zero_copy)]
pub struct Highscore {
    // Anchor is treating the Vec as a composite field, can't use it yet: https://github.com/project-serum/anchor/issues/69
    // pub scores: Vec<(u64, Pubkey)>,
    pub scores: [Score; MAX_SCORES],
    
}

#[zero_copy]
pub struct Score {
    pub owner: Pubkey,
    pub amount: u64,
}
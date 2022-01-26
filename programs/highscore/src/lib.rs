use anchor_lang::prelude::*;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");


#[program]
pub mod highscore {
use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        let scoreboard = &mut ctx.accounts.scoreboard;
        scoreboard.first_place_score = 0;
        scoreboard.second_place_score = 0;
        scoreboard.third_place_score = 0;
        scoreboard.first_place_user = *ctx.accounts.user.owner;
        scoreboard.second_place_user = *ctx.accounts.user.owner;
        scoreboard.third_place_user = *ctx.accounts.user.owner;
        Ok(())
    }

    pub fn submit_score(ctx: Context<SubmitScore>, submission: u64) -> ProgramResult {
        
        let highscore = &mut ctx.accounts.highscore;

        // Check if submission is viable
        let lowest_entry = highscore.third_place_score;
        let mut updated_existing_score = false;
        
        if lowest_entry < submission
        {
            // Simply replace score when owner is already on scoreboard and score is higher
            if *ctx.accounts.user.owner == highscore.third_place_user
            {
                highscore.third_place_score = if submission > highscore.third_place_score { submission } else {highscore.third_place_score};
                updated_existing_score = true;
                if submission > highscore.second_place_score
                {
                    if submission > highscore.first_place_score
                    {
                        highscore.third_place_score = highscore.second_place_score;
                        highscore.second_place_score = highscore.first_place_score;
                        highscore.first_place_score = submission;

                        highscore.third_place_user = highscore.second_place_user;
                        highscore.second_place_user = highscore.first_place_user;
                        highscore.first_place_user = *ctx.accounts.user.owner;
                    } else {
                        highscore.third_place_score = highscore.second_place_score;
                        highscore.second_place_score = submission;

                        highscore.third_place_user = highscore.second_place_user;
                        highscore.second_place_user = *ctx.accounts.user.owner;
                    }
                }
            }
            else if *ctx.accounts.user.owner == highscore.second_place_user
            {
                highscore.second_place_score = if submission > highscore.second_place_score { submission } else {highscore.second_place_score};
                updated_existing_score = true;

                if submission > highscore.first_place_score
                {
                    highscore.third_place_score = highscore.second_place_score;
                    highscore.second_place_score = highscore.first_place_score;
                    highscore.first_place_score = submission;

                    highscore.third_place_user = highscore.second_place_user;
                    highscore.second_place_user = highscore.first_place_user;
                    highscore.first_place_user = *ctx.accounts.user.owner;
                }
            }
            else if *ctx.accounts.user.owner == highscore.first_place_user
            {
                highscore.first_place_score = if submission > highscore.first_place_score { submission } else {highscore.first_place_score};
                updated_existing_score = true;
            }
            
            // Check each placement and reshuffle order on a hit.
            if !updated_existing_score
            {
                if submission > highscore.first_place_score
                {
                    highscore.third_place_score = highscore.second_place_score;
                    highscore.second_place_score = highscore.first_place_score;
                    highscore.first_place_score = submission;

                    highscore.third_place_user = highscore.second_place_user;
                    highscore.second_place_user = highscore.first_place_user;
                    highscore.first_place_user = *ctx.accounts.user.owner;
                } 
                else if submission > highscore.second_place_score
                {
                    highscore.third_place_score = highscore.second_place_score;
                    highscore.second_place_score = submission;

                    highscore.third_place_user = highscore.second_place_user;
                    highscore.second_place_user = *ctx.accounts.user.owner;
                }
                else if submission > highscore.third_place_score
                {
                    highscore.third_place_score = submission;
                    highscore.third_place_user = *ctx.accounts.user.owner;
                }
            }
        }
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + (3 * 8) + (40 * 3))]
    pub scoreboard: Account<'info, ScoreBoard>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct ScoreBoard {
    pub first_place_user : Pubkey,
    pub first_place_score : u64,
    pub second_place_user : Pubkey,
    pub second_place_score : u64,
    pub third_place_user : Pubkey,
    pub third_place_score : u64,
}

#[derive(Accounts)]
pub struct SubmitScore<'info> {
    #[account(mut)]
    pub highscore: Account<'info, ScoreBoard>,
    #[account(mut)]
    pub user: Signer<'info>,
}
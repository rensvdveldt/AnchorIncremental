use anchor_lang::prelude::*;
use highscore::cpi::accounts::SubmitScore;
use highscore::program::Highscore;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod game {
    use super::*;
    
    const START_CREDITS: u64 = 2;
    const START_ASSETS: u64 = 0;

    // Assets are able to be harvested every 10 seconds.
    const ASSET_REVENUE_INTERVAL : i64 = 10;

    // Highscores submitted every 24h per account
    const HIGHSCORE_SUBMISSION_INTERVAL : i64 = 86400;

    // You can increment manually every 1 second
    const INCREMENTOR_COOLDOWN: i64 = 1;
    const INCREMENTOR_UPGRADE_COST_INITIAL: u64 = 50;
    const INCREMENTOR_VALUE_INITIAL: u64 = 1;

    pub fn initialize(ctx: Context<Initialize>, authority: Pubkey) -> ProgramResult {
        
        let my_assets = &mut ctx.accounts.my_assets;
        my_assets.credits = START_CREDITS;
        my_assets.asset_1 = START_ASSETS;
        my_assets.asset_2 = START_ASSETS;
        my_assets.last_update_time = Clock::get().unwrap().unix_timestamp;
        my_assets.last_submission_time = Clock::get().unwrap().unix_timestamp;
        my_assets.authority = authority;

        let my_incrementor = &mut ctx.accounts.my_incrementor;
        my_incrementor.value = INCREMENTOR_VALUE_INITIAL;
        my_incrementor.last_used_time = Clock::get().unwrap().unix_timestamp;
        my_incrementor.upgrade_cost = INCREMENTOR_UPGRADE_COST_INITIAL;
        my_incrementor.authority = authority;

        Ok(())
    }

    pub fn increment_manual(ctx: Context<IncrementManual>) -> Result<()> {

        let my_incrementor = &mut ctx.accounts.my_incrementor;
        let my_assets = &mut ctx.accounts.my_assets;
        let time_passed = Clock::get().unwrap().unix_timestamp - my_incrementor.last_used_time;

        if time_passed >= INCREMENTOR_COOLDOWN
        {
            my_assets.credits += my_incrementor.value;
            my_incrementor.last_used_time = Clock::get().unwrap().unix_timestamp;   
        }
        else
        {
            return Err(ErrorCode::IncrementTimeInsufficient.into());
        }
        Ok(())
    }

    pub fn acquire_asset(ctx: Context<AcquireAsset>, asset_type: u64) -> Result<()> {

        let my_assets = &mut ctx.accounts.my_assets;
        let mut price = 2;
        let price_scale = 1.25;

        match asset_type {
            0 => {
                if my_assets.credits > price * ((1.0 + my_assets.asset_1 as f32).powf(price_scale)) as u64
                {
                    my_assets.credits -= price * ((1.0 + my_assets.asset_1 as f32).powf(price_scale)) as u64;
                    my_assets.asset_1 += 1;
                }
            },
            1 => {
                price = 100;
                if my_assets.credits > price * ((1.0 + my_assets.asset_2 as f32).powf(price_scale)) as u64
                {
                    my_assets.credits -= price * ((1.0 + my_assets.asset_2 as f32).powf(price_scale)) as u64;
                    my_assets.asset_2 += 1;
                }
                else 
                {
                    return Err(ErrorCode::InsiffiecientCredits.into());
                }
            },
            _ => {
                return Err(ErrorCode::UnkownAssetType.into());
            }
        }

        Ok(())
    }

    pub fn harvest_assets(ctx: Context<HarvestAssets>) -> ProgramResult {
        let my_assets = &mut ctx.accounts.my_assets;

        if Clock::get().unwrap().unix_timestamp - my_assets.last_update_time > ASSET_REVENUE_INTERVAL 
        {
            my_assets.last_update_time = Clock::get().unwrap().unix_timestamp;
            let total_increase = (my_assets.asset_1 * 3) + (my_assets.asset_2 * 15);
            my_assets.credits += total_increase;

            if Clock::get().unwrap().unix_timestamp - my_assets.last_submission_time > HIGHSCORE_SUBMISSION_INTERVAL
            {
                let cpi_program = ctx.accounts.highscore_program.to_account_info();
                let cpi_accounts = SubmitScore {
                    highscore: ctx.accounts.highscore_program.to_account_info(),
                    user: ctx.accounts.authority.to_account_info()
                };
                let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
                return highscore::cpi::submit_score(cpi_ctx, my_assets.credits);
            } 
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {

    #[account(init, payer = user, space = 8 + (5 * 8) + 40)]
    pub my_assets: Account<'info, MyAssets>,

    #[account(init, payer = user, space = 8 + (3 * 8) + 40)]
    pub my_incrementor: Account<'info, MyIncrementor>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncrementManual<'info> {
    #[account(mut, has_one = authority)]
    pub my_incrementor: Account<'info, MyIncrementor>,
    #[account(mut, has_one = authority)]
    pub my_assets: Account<'info, MyAssets>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AcquireAsset<'info> {
    #[account(mut, has_one = authority)]
    pub my_assets: Account<'info, MyAssets>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct HarvestAssets<'info> {
    #[account(mut, has_one = authority)]
    pub my_assets: Account<'info, MyAssets>,
    pub highscore_program: Program<'info, Highscore>,
    pub authority: Signer<'info>,
}

#[account]
pub struct MyAssets {
    pub authority: Pubkey,
    pub credits: u64,
    pub asset_1: u64,
    pub asset_2: u64,
    pub last_update_time: i64,
    pub last_submission_time: i64,
}

#[account]
pub struct MyIncrementor {
    pub authority: Pubkey,
    pub value: u64,
    pub last_used_time: i64,
    pub upgrade_cost: u64
}


#[error]
pub enum ErrorCode {
    #[msg("Too little time between manual increment requests.")]
    IncrementTimeInsufficient,
    #[msg("Not enough credits in account for purchase.")]
    InsiffiecientCredits,
    #[msg("Unknown asset type defined.")]
    UnkownAssetType,
}
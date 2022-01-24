use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
mod game {
    use super::*;

    const START_CREDITS: u64 = 2;
    const START_ASSETS: u64 = 0;
    const INCREMENTOR_VALUE_INITIAL: u64 = 1;
    const INCREMENTOR_UPGRADE_COST_INITIAL: u64 = 50;
    const INCREMENTOR_COOLDOWN: i64 = 1;


    pub fn initialize(ctx: Context<Initialize>) -> ProgramResult {
        
        let my_assets = &mut ctx.accounts.my_assets;
        my_assets.credits = START_CREDITS;
        my_assets.asset_1 = START_ASSETS;
        my_assets.asset_2 = START_ASSETS;

        let my_incrementor = &mut ctx.accounts.my_incrementor;
        my_incrementor.value = INCREMENTOR_VALUE_INITIAL;
        my_incrementor.last_used_time = Clock::get().unwrap().unix_timestamp;
        my_incrementor.upgrade_cost = INCREMENTOR_UPGRADE_COST_INITIAL;

        Ok(())
    }

    // pub fn update(ctx: Context<Update>, data: u64) -> ProgramResult {
    //     let my_assets = &mut ctx.accounts.my_assets;
    //     my_assets.credits = data;
    //     Ok(())
    // }

    pub fn increment_manual(ctx: Context<IncrementManual>) -> ProgramResult {

        let my_incrementor = &mut ctx.accounts.my_incrementor;
        let my_assets = &mut ctx.accounts.my_assets;
        let time_passed = Clock::get().unwrap().unix_timestamp - my_incrementor.last_used_time;

        if time_passed >= INCREMENTOR_COOLDOWN
        {
            my_assets.credits += my_incrementor.value;
            my_incrementor.last_used_time = Clock::get().unwrap().unix_timestamp;
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {

    #[account(init, payer = user, space = 8 + (3 * 8))]
    pub my_assets: Account<'info, MyAssets>,

    #[account(init, payer = user, space = 8 + (3 * 8))]
    pub my_incrementor: Account<'info, MyIncrementor>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncrementManual<'info> {
    #[account(mut)]
    pub my_incrementor: Account<'info, MyIncrementor>,
    #[account(mut)]
    pub my_assets: Account<'info, MyAssets>,
}

#[account]
pub struct MyAssets {
    pub credits: u64,
    pub asset_1: u64,
    pub asset_2: u64,
}

#[account]
pub struct MyIncrementor {
    pub value: u64,
    pub last_used_time: i64,
    pub upgrade_cost: u64
}

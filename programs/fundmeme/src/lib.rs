use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;
declare_id!("BbyJ7ARrBFaU4xYs8yZvBeBuXCTYDXfrG4jFAsPfveFg");

#[program]
pub mod fundmeme {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::initialize(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit(ctx, amount)
    }

    pub fn cashout(ctx: Context<CashOut>) -> Result<()> {
        instructions::cashout::cashout(ctx)
    }

    pub fn compound(ctx: Context<Compound>) -> Result<()> {
        instructions::compound::compound(ctx)
    }
}

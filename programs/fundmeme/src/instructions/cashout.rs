use crate::state::stake::*;
use crate::state::user::*;
use crate::utils::{
  calculate_rewards, 
  calculate_bigger_holder_rewards,
  PORK_MINT_ADDRESS,
};
use crate::errors::PorkStakeError;
use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{ self, burn, Burn as BurnSPL, Mint, Token, TokenAccount, Transfer as SplTransfer }
};


pub fn cashout(ctx: Context<CashOut>) -> Result<()> {

  require_keys_eq!(ctx.accounts.pork_mint.key(), PORK_MINT_ADDRESS, PorkStakeError::PorkMintError);
  
  let pork_mint = &ctx.accounts.pork_mint;
  let destination = &ctx.accounts.to_ata;
  let source = &ctx.accounts.stake_ata;
  let token_program = &ctx.accounts.token_program;
  let stake = &mut ctx.accounts.pork_stake;
  let user = &mut ctx.accounts.pork_user;

  let mut amount: u64 = user.claimable_amount;

  let current_timestamp = Clock::get()?.unix_timestamp;

  amount += calculate_rewards(user.deposted_amount, user.last_deposit_timestamp, current_timestamp);

  if user.times_of_bigger_holder > 0 {
    amount += calculate_bigger_holder_rewards(stake.total_amount, user.times_of_bigger_holder, user.bigger_holder_timestamp, current_timestamp);
    user.bigger_holder_timestamp = current_timestamp;
  }

  user.claimable_amount = 0;
  user.last_deposit_timestamp = current_timestamp;

  if stake.total_amount >= amount {
    stake.total_amount -= amount;
  } else {
    user.claimable_amount = amount - stake.total_amount;
    amount = stake.total_amount;
    stake.total_amount = 0;
  }

  let cashout_amount: u64 = (amount / 100 * 95).try_into().unwrap();
  let burn_amount: u64 = (amount / 100 * 5).try_into().unwrap();

  user.claimed_amount += cashout_amount;
  
  token::transfer(
    CpiContext::new_with_signer(
        token_program.to_account_info(),
        SplTransfer {
          from: source.to_account_info().clone(),
          to: destination.to_account_info().clone(),
          authority: stake.to_account_info().clone(),
        },
        &[&["pork".as_bytes(), &[stake.bump]]],
    ),
    cashout_amount
  )?;

  let signer_seeds: &[&[&[u8]]] = &[&[b"pork", &[stake.bump]]];

  let cpi_program = token_program.to_account_info();
  let cpi_accounts = BurnSPL {
      mint: pork_mint.to_account_info(),
      from: source.to_account_info(),
      authority: stake.to_account_info(),
  };

  let burn_cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
  burn(burn_cpi_ctx, burn_amount)?;
  Ok(())
}

#[derive(Accounts)]
pub struct CashOut<'info> {

  #[account(mut)]
  pub pork_mint: Account<'info, Mint>,

  #[account(mut)]
  pub to: Signer<'info>,

  #[account(
      init_if_needed, 
      payer = to, 
      associated_token::mint = pork_mint,
      associated_token::authority = to,
  )]
  pub to_ata: Account<'info, TokenAccount>,

  #[account(
      mut,  
      seeds = ["pork".as_bytes()],
      bump,
  )]
  pub pork_stake: Account<'info, PorkStake>,

  #[account(
    mut,
    associated_token::mint = pork_mint,
    associated_token::authority = pork_stake,
  )]
  pub stake_ata: Account<'info, TokenAccount>,

  #[account(
    mut,
    seeds = ["porkuser".as_bytes(), to.key().as_ref()],
    bump,
  )]
  pub pork_user: Account<'info, PorkUser>,

  pub token_program: Program<'info, Token>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>
}

use crate::state::stake::*;
use crate::state::user::*;
use crate::utils::{
  calculate_rewards, 
  calculate_bigger_holder_rewards, 
  BIGGER_HOLDER,
  MINIMUM_DEPOSIT,
  PORK_MINT_ADDRESS,
};
use crate::errors::PorkStakeError;
use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{ self, burn, Burn as BurnSPL, Mint, Token, TokenAccount, Transfer as SplTransfer }
};


pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {

  let pork_mint = &ctx.accounts.pork_mint;
  require_gte!(amount, MINIMUM_DEPOSIT, PorkStakeError::MinimumDepositError);
  require_keys_eq!(ctx.accounts.pork_mint.key(), PORK_MINT_ADDRESS, PorkStakeError::PorkMintError);
  require_keys_neq!(ctx.accounts.from.key(), ctx.accounts.referral.key(), PorkStakeError::ReferralError);

  let destination = &ctx.accounts.stake_ata;
  let source = &ctx.accounts.from_ata;
  let token_program = &ctx.accounts.token_program;
  let authority = &ctx.accounts.from;
  let stake = &mut ctx.accounts.pork_stake;
  let user = &mut ctx.accounts.pork_user;

  let deposit_amount: u64 = (amount / 100 * 95).try_into().unwrap();
  let burn_amount: u64 = (amount / 100 * 5).try_into().unwrap();

  if let Some(referral_user) = &mut ctx.accounts.referral_user {

    if user.deposted_amount == 0 {
      let referral_amount: u64 = (deposit_amount / 5).try_into().unwrap();
      referral_user.claimable_amount += referral_amount;
    }
  } 

  stake.total_amount += deposit_amount;

  let current_timestamp = Clock::get()?.unix_timestamp;

  if user.deposted_amount == 0 {
    user.deposted_amount = deposit_amount;
  } else {
    user.claimable_amount += calculate_rewards(user.deposted_amount, user.last_deposit_timestamp, current_timestamp);
    user.deposted_amount += deposit_amount;

    if user.times_of_bigger_holder > 0 {
      user.claimable_amount += calculate_bigger_holder_rewards(stake.total_amount, user.times_of_bigger_holder, user.bigger_holder_timestamp, current_timestamp);
    }
  }
  
  if user.deposted_amount >= BIGGER_HOLDER {
    
    let times_of_bigger_holder: u64 = user.deposted_amount / BIGGER_HOLDER;
    user.times_of_bigger_holder = times_of_bigger_holder;
    user.bigger_holder_timestamp = current_timestamp;
  }
  
  user.last_deposit_timestamp = current_timestamp;


  token::transfer(
    CpiContext::new(
        token_program.to_account_info(),
        SplTransfer {
          from: source.to_account_info().clone(),
          to: destination.to_account_info().clone(),
          authority: authority.to_account_info().clone(),
        },
    ),
    amount,
  )?;

  let signer_seeds: &[&[&[u8]]] = &[&[b"pork", &[stake.bump]]];

  let cpi_program = token_program.to_account_info();
  let cpi_accounts = BurnSPL {
      mint: pork_mint.to_account_info(),
      from: destination.to_account_info(),
      authority: stake.to_account_info(),
  };

  let burn_cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
  burn(burn_cpi_ctx, burn_amount)?;
  
  Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
  
  #[account(mut)]
  pub pork_mint: Account<'info, Mint>,

  #[account(mut)]
  pub from: Signer<'info>,

  #[account(
      mut, 
      associated_token::mint = pork_mint,
      associated_token::authority = from,
  )]
  pub from_ata: Account<'info, TokenAccount>,

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
    init_if_needed,
    payer = from,
    space=PorkUser::LEN,
    seeds = ["porkuser".as_bytes(), from.key().as_ref()],
    bump,
  )]
  pub pork_user: Account<'info, PorkUser>,

  pub referral: SystemAccount<'info>,

  #[account(
    init_if_needed,
    payer = from,
    space=PorkUser::LEN,
    seeds = ["porkuser".as_bytes(), referral.key().as_ref()],
    bump,
  )]
  pub referral_user: Option<Account<'info, PorkUser>>,
  
  token_program: Program<'info, Token>,
  associated_token_program: Program<'info, AssociatedToken>,
  system_program: Program<'info, System>
}



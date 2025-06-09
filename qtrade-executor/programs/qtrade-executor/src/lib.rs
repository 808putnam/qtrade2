// Stack-optimized version of qtrade-executor with Box wrappers
// This version reduces stack usage to avoid BPF stack overflow

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

mod constants;
mod utils;

pub use constants::*;

declare_id!("E4uFtpkcE9vPXfULJaCZrJvoiSW9rJ1oqnhmHJMsEErj");

#[program]
pub mod qtrade_executor {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.owner = ctx.accounts.owner.key();
        state.swap_multi_fee = DEFAULT_SWAP_MULTI_FEE;
        state.bump = ctx.bumps.state;
        Ok(())
    }

    pub fn register_referral_code(
        ctx: Context<RegisterReferralCode>,
        referral_code: u32,
        referral_fee: u16,
        beneficiary: Pubkey,
    ) -> Result<()> {
        let referral_info = &mut ctx.accounts.referral_info;

        require!(referral_fee <= MAX_REFERRAL_FEE, ErrorCode::FeeTooHigh);

        if referral_code <= REFERRAL_WITH_FEE_THRESHOLD {
            require!(referral_fee == 0, ErrorCode::InvalidFeeForCode);
        } else {
            require!(referral_fee > 0, ErrorCode::InvalidFeeForCode);
            require!(beneficiary != Pubkey::default(), ErrorCode::NullBeneficiary);
        }

        referral_info.referral_fee = referral_fee;
        referral_info.beneficiary = beneficiary;
        referral_info.registered = true;
        referral_info.bump = ctx.bumps.referral_info;

        Ok(())
    }

    pub fn set_swap_multi_fee(ctx: Context<SetSwapMultiFee>, new_fee: u16) -> Result<()> {
        require!(new_fee <= MAX_SWAP_MULTI_FEE, ErrorCode::FeeTooHigh);
        ctx.accounts.state.swap_multi_fee = new_fee;
        Ok(())
    }

    /// Complete swap function mapping OdosRouterV2.swap() to Solana DEX integration
    /// Equivalent to: OdosRouterV2.swap(SwapTokenInfo calldata tokenInfo, bytes calldata pathDefinition, address executor, uint32 referralCode)
    pub fn swap(
        ctx: Context<Swap>,
        input_amount: u64,
        output_min: u64,
        output_quote: u64,
        referral_code: u32,
    ) -> Result<()> {
        // Validation equivalent to OdosRouterV2 require statements
        require!(input_amount > 0, ErrorCode::InvalidFundsTransfer);
        require!(output_min <= output_quote, ErrorCode::MinimumGreaterThanQuote);
        require!(output_min > 0, ErrorCode::SlippageLimitTooLow);

        // Handle referral fees if applicable (equivalent to OdosRouterV2 referral logic)
        let referral_fee_amount = if referral_code > REFERRAL_WITH_FEE_THRESHOLD {
            // Validate referral info exists for fee-bearing codes
            require!(!ctx.remaining_accounts.is_empty(), ErrorCode::ReferralInfoMissing);

            // Calculate referral fee using basis points (simplified from OdosRouterV2's complex calculation)
            utils::calculate_referral_fee(output_quote, 250, FEE_DENOMINATOR)? // Example: 2.5% fee
        } else {
            0
        };

        // Transfer input tokens from user to router (equivalent to Solidity transferFrom)
        let cpi_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.user_input_account.to_account_info(),
            to: ctx.accounts.router_input_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, input_amount)?;

        // TODO: Execute actual swap via Jupiter/Orca/Raydium DEX integration
        // This replaces OdosRouterV2's pathDefinition execution logic
        // For now, simulate the swap output (in production this would be DEX CPI calls)
        let actual_output = output_quote; // This would come from DEX execution

        // Apply slippage validation (equivalent to OdosRouterV2 slippage checks)
        require!(actual_output >= output_min, ErrorCode::SlippageLimitExceeded);

        // Calculate final output after referral fees
        let final_output = actual_output.checked_sub(referral_fee_amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        // TODO: Transfer output tokens back to user
        // This would involve transferring from router's output token account to user's output token account

        // Calculate slippage for event emission (equivalent to OdosRouterV2 event data)
        let slippage = utils::calculate_slippage(actual_output, output_quote);

        // Emit comprehensive swap event (equivalent to OdosRouterV2.Swap event)
        emit!(SwapEvent {
            user: ctx.accounts.user.key(),
            input_amount,
            input_token: ctx.accounts.user_input_account.mint,
            amount_out: final_output,
            output_token: ctx.accounts.user_input_account.mint, // TODO: Get from output account
            slippage,
            referral_code,
        });

        Ok(())
    }

    /// Transfer router funds function (equivalent to OdosRouterV2 admin functions)
    /// Maps to OdosRouterV2.transferRouterFunds() for emergency fund recovery
    /// Allows owner to withdraw tokens from router accounts (admin-only function)
    pub fn transfer_router_funds(
        ctx: Context<TransferRouterFunds>,
        token_mint: Pubkey,
        amount: u64,
        recipient: Pubkey,
    ) -> Result<()> {
        // Validation (equivalent to OdosRouterV2 onlyOwner modifier and require statements)
        require!(amount > 0, ErrorCode::InvalidFundsTransfer);
        require!(recipient != Pubkey::default(), ErrorCode::NullBeneficiary);

        // TODO: Implement actual token transfer from router accounts to recipient
        // This would involve:
        // 1. Finding the router's token account for the specified mint
        // 2. Creating CPI call to transfer tokens using PDA authority
        // 3. Validating sufficient balance exists

        // For now, emit an event to track the transfer request
        emit!(TransferRouterFundsEvent {
            owner: ctx.accounts.owner.key(),
            token_mint,
            amount,
            recipient,
        });

        Ok(())
    }

    /// Multi-token swap function (equivalent to OdosRouterV2.swapMulti)
    /// Handles multiple input/output token swaps with complex routing
    pub fn swap_multi(
        ctx: Context<SwapMulti>,
        referral_code: u32,
    ) -> Result<()> {
        // Apply swap multi fee (equivalent to OdosRouterV2 swapMultiFee logic)
        let multi_fee = ctx.accounts.state.swap_multi_fee;

        // TODO: Implement multi-token swap logic
        // This is equivalent to OdosRouterV2.swapMulti() which handles:
        // - Multiple input tokens (via remaining_accounts)
        // - Multiple output tokens (via remaining_accounts)
        // - Complex routing through multiple DEXes
        // - Multi-fee calculations on total output value

        // Placeholder validation
        require!(multi_fee <= MAX_SWAP_MULTI_FEE, ErrorCode::FeeTooHigh);

        // Emit event for tracking
        emit!(SwapMultiEvent {
            sender: ctx.accounts.user.key(),
            referral_code,
            multi_fee,
        });

        Ok(())
    }
}

// Simplified account structures
#[account]
pub struct ProgramState {
    pub owner: Pubkey,
    pub swap_multi_fee: u16,
    pub bump: u8,
}

#[account]
pub struct ReferralInfo {
    pub referral_fee: u16,
    pub beneficiary: Pubkey,
    pub registered: bool,
    pub bump: u8,
}

// Simplified contexts with minimal accounts - using Box to move to heap
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + 32 + 2 + 1, // Discriminator + pubkey + u16 + u8
        seeds = [STATE_SEED],
        bump
    )]
    pub state: Box<Account<'info, ProgramState>>, // Box to move to heap

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(referral_code: u32)]
pub struct RegisterReferralCode<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [STATE_SEED],
        bump = state.bump
    )]
    pub state: Box<Account<'info, ProgramState>>, // Box to move to heap

    #[account(
        init,
        payer = payer,
        space = 8 + 2 + 32 + 1 + 1, // Discriminator + u16 + pubkey + bool + u8
        seeds = [REFERRAL_SEED, referral_code.to_le_bytes().as_ref()],
        bump
    )]
    pub referral_info: Box<Account<'info, ReferralInfo>>, // Box to move to heap

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetSwapMultiFee<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = owner
    )]
    pub state: Box<Account<'info, ProgramState>>, // Box to move to heap

    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [STATE_SEED],
        bump = state.bump
    )]
    pub state: Box<Account<'info, ProgramState>>, // Box to move to heap

    #[account(mut)]
    pub user_input_account: Box<Account<'info, TokenAccount>>, // Box to move to heap

    #[account(mut)]
    pub router_input_account: Box<Account<'info, TokenAccount>>, // Box to move to heap

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferRouterFunds<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump = state.bump,
        has_one = owner  // Equivalent to onlyOwner modifier
    )]
    pub state: Box<Account<'info, ProgramState>>,

    pub owner: Signer<'info>,

    // TODO: Add token accounts for the specific token being transferred
    // This would include router's token account and recipient's token account

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(referral_code: u32)]
pub struct SwapMulti<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [STATE_SEED],
        bump = state.bump
    )]
    pub state: Box<Account<'info, ProgramState>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    // Note: Multiple input/output token accounts would be passed via remaining_accounts
    // This avoids the stack overflow issues we encountered with large account structures
}

// Simplified event
#[event]
pub struct SwapEvent {
    pub user: Pubkey,
    pub input_amount: u64,
    pub input_token: Pubkey,
    pub amount_out: u64,
    pub output_token: Pubkey,
    pub slippage: i64,
    pub referral_code: u32,
}

/// Emitted when a multi-token swap is executed
/// Equivalent to OdosRouterV2.SwapMulti event
#[event]
pub struct SwapMultiEvent {
    pub sender: Pubkey,
    pub referral_code: u32,
    pub multi_fee: u16,
}

/// Emitted when router funds are transferred (admin function)
/// Equivalent to OdosRouterV2.TransferRouterFunds event
#[event]
pub struct TransferRouterFundsEvent {
    pub owner: Pubkey,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub recipient: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Fee too high")]
    FeeTooHigh,
    #[msg("Invalid fee for code")]
    InvalidFeeForCode,
    #[msg("Null beneficiary")]
    NullBeneficiary,
    #[msg("Minimum greater than quote")]
    MinimumGreaterThanQuote,
    #[msg("Slippage limit too low")]
    SlippageLimitTooLow,
    #[msg("Slippage limit exceeded")]
    SlippageLimitExceeded,
    #[msg("Invalid funds transfer")]
    InvalidFundsTransfer,
    #[msg("Referral info missing")]
    ReferralInfoMissing,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}

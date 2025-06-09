use anchor_lang::prelude::*;
use anchor_spl::token::{self, TokenAccount, Transfer};
use crate::constants::NATIVE_SOL;

/// Utility functions module - equivalent to OdosRouterV2 internal helper functions
/// Provides cross-platform token handling for both native SOL and SPL tokens

/// Get the balance of either native SOL or SPL token for an account
/// Equivalent to OdosRouterV2._universalBalance()
#[allow(dead_code)]
pub fn get_universal_balance(
    account_info: &AccountInfo,
    token_address: &Pubkey,
) -> Result<u64> {
    if *token_address == NATIVE_SOL {
        // For native SOL, return lamports balance
        Ok(account_info.lamports())
    } else {
        // For SPL tokens, parse and return token account balance
        let token_account = TokenAccount::try_deserialize(&mut account_info.data.borrow().as_ref())?;
        Ok(token_account.amount)
    }
}

/// Transfer either native SOL or SPL tokens
/// Equivalent to OdosRouterV2._universalTransfer()
#[allow(dead_code)]
pub fn universal_transfer<'info>(
    token_address: &Pubkey,
    from_account: &AccountInfo<'info>,
    to_account: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    if *token_address == NATIVE_SOL {
        // Transfer native SOL
        let from_lamports = from_account.lamports();
        require!(from_lamports >= amount, ErrorCode::InsufficientBalance);

        **from_account.try_borrow_mut_lamports()? = from_lamports.checked_sub(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        **to_account.try_borrow_mut_lamports()? = to_account.lamports().checked_add(amount)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
    } else {
        // Transfer SPL token using CPI
        let transfer_accounts = Transfer {
            from: from_account.clone(),
            to: to_account.clone(),
            authority: authority.clone(),
        };
        let transfer_ctx = CpiContext::new(token_program.clone(), transfer_accounts);
        token::transfer(transfer_ctx, amount)?;
    }

    Ok(())
}

/// Calculate referral fee amount (simplified for basis points)
/// Equivalent to the referral fee calculation in OdosRouterV2._swap() and _swapMulti()
/// Simplified from complex multiplication/division to basis points calculation
#[allow(dead_code)]
pub fn calculate_referral_fee(
    amount: u64,
    referral_fee_rate: u16,
    fee_denominator: u16,
) -> Result<u64> {
    amount
        .checked_mul(referral_fee_rate as u64)
        .and_then(|x| x.checked_div(fee_denominator as u64))
        .ok_or(ErrorCode::ArithmeticOverflow.into())
}

/// Apply fee to an amount using basis points
/// Used for both swap multi fee and referral fee calculations
#[allow(dead_code)]
pub fn apply_fee(
    amount: u64,
    fee_rate: u16,
    fee_denominator: u16,
) -> Result<u64> {
    amount
        .checked_mul((fee_denominator.saturating_sub(fee_rate)) as u64)
        .and_then(|x| x.checked_div(fee_denominator as u64))
        .ok_or(ErrorCode::ArithmeticOverflow.into())
}

/// Validate token arrays for duplicates and arbitrage
/// Equivalent to validation logic in OdosRouterV2._swapMulti()
#[allow(dead_code)]
pub fn validate_token_arrays(
    input_tokens: &[Pubkey],
    output_tokens: &[Pubkey],
) -> Result<()> {
    // Check for duplicate input tokens
    for (i, input) in input_tokens.iter().enumerate() {
        for (j, other_input) in input_tokens.iter().enumerate() {
            if i != j && input == other_input {
                return Err(ErrorCode::DuplicateSourceTokens.into());
            }
        }
    }

    // Check for duplicate output tokens
    for (i, output) in output_tokens.iter().enumerate() {
        for (j, other_output) in output_tokens.iter().enumerate() {
            if i != j && output == other_output {
                return Err(ErrorCode::DuplicateDestinationTokens.into());
            }
        }
    }

    // Check for arbitrage (input and output tokens cannot be the same)
    for input in input_tokens.iter() {
        for output in output_tokens.iter() {
            if input == output {
                return Err(ErrorCode::ArbitrageNotSupported.into());
            }
        }
    }

    Ok(())
}

/// Validate single swap parameters
/// Equivalent to validation logic in OdosRouterV2._swap()
#[allow(dead_code)]
pub fn validate_swap_params(
    input_token: &Pubkey,
    output_token: &Pubkey,
    output_min: u64,
    output_quote: u64,
) -> Result<()> {
    require!(
        output_min <= output_quote,
        ErrorCode::MinimumGreaterThanQuote
    );
    require!(output_min > 0, ErrorCode::SlippageLimitTooLow);
    require!(
        input_token != output_token,
        ErrorCode::ArbitrageNotSupported
    );

    Ok(())
}

/// Helper to check if referral code requires fee handling
/// Equivalent to referralCode > REFERRAL_WITH_FEE_THRESHOLD check in OdosRouterV2
#[allow(dead_code)]
pub fn requires_referral_fee_handling(referral_code: u32, threshold: u32) -> bool {
    referral_code > threshold
}

/// Calculate slippage value
/// Equivalent to slippage calculation in OdosRouterV2._swap()
#[allow(dead_code)]
pub fn calculate_slippage(actual_output: u64, quoted_output: u64) -> i64 {
    (actual_output as i64).saturating_sub(quoted_output as i64)
}

/// Custom error codes for utility functions
#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient balance")]
    InsufficientBalance,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("Minimum greater than quote")]
    MinimumGreaterThanQuote,

    #[msg("Slippage limit too low")]
    SlippageLimitTooLow,

    #[msg("Arbitrage not supported")]
    ArbitrageNotSupported,

    #[msg("Duplicate source tokens")]
    DuplicateSourceTokens,

    #[msg("Duplicate destination tokens")]
    DuplicateDestinationTokens,
}

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount,};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

// Replace for Devnet Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr
// Replace for Localnet 8fFnX9WSPjJEADtG5jQvQQptzfFmmjd6hrW7HjuUT8ur
pub const USDC_MINT_ADDRESS: &str = "8fFnX9WSPjJEADtG5jQvQQptzfFmmjd6hrW7HjuUT8ur";

pub const AUTHORITY: &str = "DfLZV18rD7wCQwjYvhTFwuvLh49WSbXFeJFPQb5czifH";


#[program]
pub mod token3 {
    use super::*;

     pub fn new_token(ctx: Context<NewToken>, name: String, transaction_fee: u64, sale_fee: u64, discount: u64, reward_generic_token: u64, reward_merchant_token: u64, reward_usdc_token: u64) -> Result<()> {
        //TODO: check pdas match accounts passed
        let (token_pda, token_bump) =
            Pubkey::find_program_address(&["MINT".as_bytes(), ctx.accounts.token_data.key().as_ref()], ctx.program_id);

        let (earned_pda, earned_bump) =
            Pubkey::find_program_address(&["EARNED".as_bytes(), ctx.accounts.token_data.key().as_ref(), ctx.accounts.mint.key().as_ref()], ctx.program_id);
        
        let (reserve_pda, reserve_bump) =
            Pubkey::find_program_address(&["RESERVE".as_bytes(), ctx.accounts.token_data.key().as_ref(), ctx.accounts.mint.key().as_ref()], ctx.program_id);

        if token_pda != ctx.accounts.token_mint.key() {
            return err!(ErrorCode::PDA);
        }

        if earned_pda != ctx.accounts.earned_usdc_account.key() {
            return err!(ErrorCode::PDA);
        }

        if reserve_pda != ctx.accounts.reserve_usdc_account.key() {
            return err!(ErrorCode::PDA);
        }

        let token_data = &mut ctx.accounts.token_data;
        token_data.name = name;
        token_data.user = ctx.accounts.user.key();
        token_data.mint = token_pda;
        token_data.earned = earned_pda;
        token_data.reserve = reserve_pda;
        token_data.mint_bump = token_bump;
        token_data.earned_bump = earned_bump;
        token_data.reserve_bump = reserve_bump;
        token_data.transaction_fee = transaction_fee;
        token_data.sale_fee = sale_fee;
        token_data.discount = discount;
        token_data.reward_generic_token = reward_generic_token;
        token_data.reward_merchant_token = reward_merchant_token;
        token_data.reward_usdc_token = reward_usdc_token;
        
        Ok(())
    }

    pub fn mint_token(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        let token_data = ctx.accounts.token_data.key();

        // mint tokens to user
        let seeds = &["MINT".as_bytes(), token_data.as_ref(), &[ctx.accounts.token_data.mint_bump]];
        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.token_mint.to_account_info(),
            },
            &signer,
        );
        token::mint_to(cpi_ctx, amount)?;

        let discount = &ctx.accounts.token_data.discount;
        let sale_fee = &ctx.accounts.token_data.sale_fee;
        
        // TODO: Account for Decimals in Mint
        let usdc_amount = amount * (10000-discount) / 10000;
        let fee_amount = usdc_amount * (sale_fee) / 10000;
        let reserve_amount = usdc_amount - fee_amount ;
        
        // transfer USDC from the User to Treasury
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_usdc_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.treasury_account.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, fee_amount)?;

        // transfer USDC from the User to Reserve
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_usdc_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.reserve_usdc_account.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, reserve_amount)?;


        Ok(())
    }

    pub fn redeem_usdc(ctx: Context<RedeemUsdc>, amount: u64,) -> Result<()> {
        let token_data = ctx.accounts.token_data.key();
        let fee_amount = ctx.accounts.token_data.transaction_fee; 
        let reward_amount = amount * ctx.accounts.token_data.reward_usdc_token / 10000;
        let earned_amount = amount - reward_amount - fee_amount;
        
        // mint reward token to user
        let seeds = &["MINT".as_bytes(), token_data.as_ref(), &[ctx.accounts.token_data.mint_bump]];
        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.token_mint.to_account_info(),
            },
            &signer,
        );

        token::mint_to(cpi_ctx, reward_amount)?;
    
        // transfer USDC fee from User to treasury
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_usdc_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.treasury_account.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, fee_amount)?;

        // transfer USDC reserve
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_usdc_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.reserve_usdc_account.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, reward_amount)?;

        // transfer USDC earned
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_usdc_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.earned_usdc_account.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, earned_amount)?;
        
        Ok(())
    }


    pub fn redeem_one_token(ctx: Context<RedeemOneToken>, amount: u64,) -> Result<()> {
        let token_data = ctx.accounts.token_data.key();
        let reward_amount = amount * ctx.accounts.token_data.reward_merchant_token / 10000;
        let fee_amount = ctx.accounts.token_data.transaction_fee; 
        let usdc_value = (amount - reward_amount) * (ctx.accounts.reserve_usdc_account.amount) / (ctx.accounts.token_mint.supply);
        let earned_amount = usdc_value - fee_amount;
        
        // burn tokens        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::burn(cpi_ctx, amount)?;
        
        // mint is usdc mint
        let mint = ctx.accounts.mint.key();
        let seeds = &["RESERVE".as_bytes(), token_data.as_ref(), mint.as_ref(), &[ctx.accounts.token_data.reserve_bump]];
        let signer = [&seeds[..]];

        // transfer USDC fee from reserve to treasury
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.reserve_usdc_account.to_account_info(),
                authority: ctx.accounts.reserve_usdc_account.to_account_info(),
                to: ctx.accounts.treasury_account.to_account_info(),
            },
            &signer,
        );

        token::transfer(cpi_ctx, fee_amount)?;

        // transfer USDC earned
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.reserve_usdc_account.to_account_info(),
                authority: ctx.accounts.reserve_usdc_account.to_account_info(),
                to: ctx.accounts.earned_usdc_account.to_account_info(),
            },
            &signer,
        );
        
        token::transfer(cpi_ctx, earned_amount)?;

        // mint reward token to user
        let seeds = &["MINT".as_bytes(), token_data.as_ref(), &[ctx.accounts.token_data.mint_bump]];
        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.token_mint.to_account_info(),
            },
            &signer,
        );

        token::mint_to(cpi_ctx, reward_amount)?;
        
        
        Ok(())
    }

    pub fn redeem_two_token(ctx: Context<PartialRedeem>, token_amount: u64, usdc_amount:u64) -> Result<()> {
        let token_data = ctx.accounts.token_data.key();
        let token_reward_amount = token_amount * ctx.accounts.token_data.reward_merchant_token / 10000;
        let usdc_reward_amount = usdc_amount * ctx.accounts.token_data.reward_usdc_token / 10000;
        let total_reward_amount = token_reward_amount + usdc_reward_amount;
        let usdc_value = (token_amount - token_reward_amount) * (ctx.accounts.reserve_usdc_account.amount) / (ctx.accounts.token_mint.supply);
        let fee_amount = ctx.accounts.token_data.transaction_fee; 
        let earned_amount = usdc_value - fee_amount;
        let usdc_earned_amount = usdc_amount - usdc_reward_amount; 

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Burn {
                mint: ctx.accounts.token_mint.to_account_info(),
                from: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::burn(cpi_ctx, token_amount)?;
        
        let mint = ctx.accounts.mint.key();
        let seeds = &["RESERVE".as_bytes(), token_data.as_ref(), mint.as_ref(), &[ctx.accounts.token_data.reserve_bump]];
        let signer = [&seeds[..]];

        // transfer USDC fee.
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.reserve_usdc_account.to_account_info(),
                authority: ctx.accounts.reserve_usdc_account.to_account_info(),
                to: ctx.accounts.treasury_account.to_account_info(),
            },
            &signer,
        );

        token::transfer(cpi_ctx, fee_amount)?;

        // transfer USDC from reserve to earned
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.reserve_usdc_account.to_account_info(),
                authority: ctx.accounts.reserve_usdc_account.to_account_info(),
                to: ctx.accounts.earned_usdc_account.to_account_info(),
            },
            &signer,
        );
        
        token::transfer(cpi_ctx, earned_amount)?;

        // transfer USDC from the User to earned
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_usdc_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.earned_usdc_account.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, usdc_earned_amount)?;

        // transfer USDC from the User to reserve
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.user_usdc_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.reserve_usdc_account.to_account_info(),
            },
        );
        token::transfer(cpi_ctx, usdc_reward_amount)?;

        // mint reward token to user
        let seeds = &["MINT".as_bytes(), token_data.as_ref(), &[ctx.accounts.token_data.mint_bump]];
        let signer = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.user_token.to_account_info(),
                authority: ctx.accounts.token_mint.to_account_info(),
            },
            &signer,
        );

        token::mint_to(cpi_ctx, total_reward_amount)?;
        
        
        Ok(())
    }

   pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        
        let token_data = ctx.accounts.token_data.key();
        let mint = ctx.accounts.mint.key();
        let seeds = &["EARNED".as_bytes(), token_data.as_ref(), mint.as_ref(), &[ctx.accounts.token_data.earned_bump]];
        let signer = [&seeds[..]];

        // transfer USDC earned
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.earned_usdc_account.to_account_info(),
                authority: ctx.accounts.earned_usdc_account.to_account_info(),
                to: ctx.accounts.withdraw_usdc_account.to_account_info(),
            },
            &signer,
        );
        
        let amount = ctx.accounts.earned_usdc_account.amount;
        token::transfer(cpi_ctx, amount)?;
        
        
        Ok(())
    }

    //TODO: does each field need own function to update? can inputs be conditional?
    pub fn update_token_data(ctx: Context<UpdateTokenData>, name: String, discount: u64, reward_usdc_token: u64) -> Result<()> {
        
        let token_data = &mut ctx.accounts.token_data;
        
        token_data.name = name;
        token_data.discount = discount;
        token_data.reward_usdc_token = reward_usdc_token;
        
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct NewToken<'info> {
    #[account(
        init,
        payer = user,
        space = 10000 // TODO: calculate space
    )]
    pub token_data: Account<'info, TokenData>,

    #[account(
        init,
        seeds = ["MINT".as_bytes().as_ref(), token_data.key().as_ref()],
        bump,
        payer = user,
        mint::decimals = 6,
        mint::authority = token_mint, 
        
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        seeds = ["EARNED".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump,
        token::mint = mint,
        token::authority = earned_usdc_account,
    )]
    pub earned_usdc_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        seeds = ["RESERVE".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump,
        token::mint = mint,
        token::authority = reserve_usdc_account,
    )]
    pub reserve_usdc_account: Account<'info, TokenAccount>,

    // "USDC" Mint
    #[account(
        address = USDC_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account()]
    pub token_data: Box<Account<'info, TokenData>>,
    
    #[account(mut,
        seeds = ["MINT".as_bytes().as_ref(), token_data.key().as_ref()],
        bump = token_data.mint_bump
    )]
    pub token_mint: Box<Account<'info, Mint>>,

    // USDC to here
    #[account(
        mut,
        constraint = reserve_usdc_account.mint == mint.key(),
        seeds = ["RESERVE".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.reserve_bump,
    )]
    pub reserve_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        constraint = treasury_account.mint == mint.key(),
    )]
    pub treasury_account: Box<Account<'info, TokenAccount>>,

    // Mint Tokens here
    #[account(mut,
        constraint = user_token.mint == token_mint.key(),
        constraint = user_token.owner == user.key() 
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    // USDC from here
    #[account(mut,
        constraint = user_usdc_token.mint == mint.key(),
        constraint = user_usdc_token.owner == user.key()
    )]
    pub user_usdc_token: Box<Account<'info, TokenAccount>>,
    
    pub user: Signer<'info>,

    // "USDC" Mint
    #[account(
        address = USDC_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct RedeemUsdc<'info> {
    #[account()]
    pub token_data: Box<Account<'info, TokenData>>,

    #[account(mut,
        seeds = ["MINT".as_bytes().as_ref(), token_data.key().as_ref()],
        bump = token_data.mint_bump
    )]
    pub token_mint: Box<Account<'info, Mint>>,
    
    // Mint Tokens here
    #[account(mut,
        constraint = user_token.mint == token_mint.key(),
        constraint = user_token.owner == user.key() 
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    // USDC from here
    #[account(mut,
        constraint = user_usdc_token.mint == mint.key(),
        constraint = user_usdc_token.owner == user.key()
    )]
    pub user_usdc_token: Box<Account<'info, TokenAccount>>,
    
    // The authority allowed to mutate the above ⬆
    pub user: Signer<'info>,

    // USDC to here
    #[account(
        mut,
        constraint = reserve_usdc_account.mint == mint.key(),
        seeds = ["RESERVE".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.reserve_bump,
    )]
    pub reserve_usdc_account: Box<Account<'info, TokenAccount>>,

    // USDC to here
    #[account(
        mut,
        constraint = reserve_usdc_account.mint == mint.key(),
        seeds = ["EARNED".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.earned_bump,
    )]
    pub earned_usdc_account: Box<Account<'info, TokenAccount>>,
    
    
    // Transaction fee to here
    #[account(mut,
        constraint = treasury_account.mint == mint.key(),
    )]
    pub treasury_account: Box<Account<'info, TokenAccount>>,
    
    // "USDC" Mint
    #[account(
        address = USDC_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub mint: Account<'info, Mint>,
    
    // SPL Token Program
    pub token_program: Program<'info, Token>,
}


#[derive(Accounts)]
pub struct RedeemOneToken<'info> {
    #[account()]
    pub token_data: Box<Account<'info, TokenData>>,

    #[account(mut,
        seeds = ["MINT".as_bytes().as_ref(), token_data.key().as_ref()],
        bump = token_data.mint_bump
    )]
    pub token_mint: Box<Account<'info, Mint>>,
    

    // Mint Tokens here
    #[account(mut,
        constraint = user_token.mint == token_mint.key(),
        constraint = user_token.owner == user.key() 
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    // The authority allowed to mutate the above ⬆
    pub user: Signer<'info>,

    // USDC to here
    #[account(
        mut,
        constraint = reserve_usdc_account.mint == mint.key(),
        seeds = ["RESERVE".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.reserve_bump,
    )]
    pub reserve_usdc_account: Box<Account<'info, TokenAccount>>,

    // USDC to here
    #[account(
        mut,
        constraint = reserve_usdc_account.mint == mint.key(),
        seeds = ["EARNED".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.earned_bump,
    )]
    pub earned_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        constraint = treasury_account.mint == mint.key(),
    )]
    pub treasury_account: Box<Account<'info, TokenAccount>>,

    // "USDC" Mint
    #[account(
        address = USDC_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub mint: Account<'info, Mint>,

    // SPL Token Program
    pub token_program: Program<'info, Token>,

}


#[derive(Accounts)]
pub struct PartialRedeem<'info> {
    #[account()]
    pub token_data: Box<Account<'info, TokenData>>,

    #[account(mut,
        seeds = ["MINT".as_bytes().as_ref(), token_data.key().as_ref()],
        bump = token_data.mint_bump
    )]
    pub token_mint: Box<Account<'info, Mint>>,
    

    // Mint Tokens here
    #[account(mut,
        constraint = user_token.mint == token_mint.key(),
        constraint = user_token.owner == user.key() 
    )]
    pub user_token: Box<Account<'info, TokenAccount>>,

    // USDC from here
    #[account(mut,
        constraint = user_usdc_token.mint == mint.key(),
        constraint = user_usdc_token.owner == user.key()
    )]
    pub user_usdc_token: Box<Account<'info, TokenAccount>>,

    // The authority allowed to mutate the above ⬆
    pub user: Signer<'info>,

    // USDC to here
    #[account(
        mut,
        constraint = reserve_usdc_account.mint == mint.key(),
        seeds = ["RESERVE".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.reserve_bump,
    )]
    pub reserve_usdc_account: Box<Account<'info, TokenAccount>>,

    // USDC to here
    #[account(
        mut,
        constraint = reserve_usdc_account.mint == mint.key(),
        seeds = ["EARNED".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.earned_bump,
    )]
    pub earned_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        constraint = treasury_account.mint == mint.key(),
    )]
    pub treasury_account: Box<Account<'info, TokenAccount>>,

    // "USDC" Mint
    #[account(
        address = USDC_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub mint: Account<'info, Mint>,

    // SPL Token Program
    pub token_program: Program<'info, Token>,

}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account()]
    pub token_data: Box<Account<'info, TokenData>>,

    // USDC to here
    #[account(
        mut,
        constraint = earned_usdc_account.mint == mint.key(),
        seeds = ["EARNED".as_bytes().as_ref(), token_data.key().as_ref(), mint.key().as_ref() ],
        bump = token_data.earned_bump,
    )]
    pub earned_usdc_account: Box<Account<'info, TokenAccount>>,

    #[account(mut,
        constraint =withdraw_usdc_account.mint == mint.key(),
    )]
    pub withdraw_usdc_account: Box<Account<'info, TokenAccount>>,

    // "USDC" Mint
    #[account(
        address = USDC_MINT_ADDRESS.parse::<Pubkey>().unwrap(),
    )]
    pub mint: Account<'info, Mint>,

    // SPL Token Program
    pub token_program: Program<'info, Token>,

    // Require Authority Signiture to Withdraw
    #[account(
        address = AUTHORITY.parse::<Pubkey>().unwrap(),
    )]
    pub authority: Signer<'info>,

}

#[derive(Accounts)]
pub struct UpdateTokenData<'info> {
    #[account(mut)]
    pub token_data: Box<Account<'info, TokenData>>,

    // Require Authority Signiture to Withdraw
    #[account(
        constraint = token_data.user == user.key()
    )]
    pub user: Signer<'info>,
}

#[account]
pub struct TokenData {
    pub name: String,
    pub user: Pubkey,
    pub mint: Pubkey,
    pub earned: Pubkey,
    pub reserve: Pubkey,
    pub mint_bump: u8,
    pub earned_bump: u8,
    pub reserve_bump: u8,
    pub transaction_fee: u64, // fee per transaction
    pub sale_fee: u64, // usdc -> diam fee
    pub discount: u64, // usdc -> merchant token discount
    pub reward_generic_token: u64, // token -> mint on redemption
    pub reward_merchant_token: u64, // token -> mint on redemption
    pub reward_usdc_token: u64, // token -> mint on redemption
}

#[error_code]
pub enum ErrorCode {
    #[msg("PDA not match")]
    PDA
}
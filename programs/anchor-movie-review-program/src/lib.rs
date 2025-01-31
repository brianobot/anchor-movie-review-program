use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, MintTo, Mint, Token, TokenAccount};

mod constants;
use constants::*;

declare_id!("8zLkkwXZ6wjdBHKqwzYHjjZu8VLnpcRSyxfZKV6wJhUA");

#[program]
pub mod anchor_movie_review_program {
    use super::*;

    pub fn add_movie_review(
        ctx: Context<AddMovieReview>,
        title: String,
        description: String,
        rating: u8,
    ) -> Result<()> {
        ctx.accounts.add_review(title, description, rating, &ctx.bumps)?;
        Ok(())
    }

    pub fn update_movie_review(
        ctx: Context<UpdateMovieReview>,
        _title: String,
        description: String,
        rating: u8,
    ) -> Result<()> {
        ctx.accounts.update_movie_review(description, rating)?;

        Ok(())
    }

    pub fn delete_movie_review(_ctx: Context<DeleteMovieReview>, title: String) -> Result<()> {
        msg!("Movie review for {} deleted", title);
        Ok(())
    }

    pub fn initialize_token_mint(ctx: Context<InitializeMint>) -> Result<()> {
        ctx.accounts.init()?;
        Ok(())
    }

}

#[derive(Accounts)]
#[instruction(title: String, description: String)]
pub struct AddMovieReview<'info> {
    #[account(
        init,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = MovieAccountState::INIT_SPACE + title.len() + description.len(), // We add the length of the title and description to the init space
    )]
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = mint,
        associated_token::authority = initializer,
    )]
    pub token_account: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> AddMovieReview<'info> {
    pub fn add_review(&mut self, title: String, description: String, rating: u8, bumps: &AddMovieReviewBumps) -> Result<()> {
        // check title length is valid
        require!(title.len() <= MAX_TITLE_LENGTH, MovieReviewError::TitleTooLong);

        // check decsription length is valid
        require!(description.len() <= MAX_DESCRIPTION_LENGTH, MovieReviewError::DescriptionTooLong);

        // check rating is valid
        require!(rating > MIN_RATING && rating <= MAX_RATING, MovieReviewError::InvalidRating);

        // initialize the movie account state account
        self.movie_review.set_inner( MovieAccountState {
            reviewer: *self.initializer.key,
            rating,
            title,
            description,
        });

        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = MintTo {
            mint: self.mint.to_account_info(),
            to: self.token_account.to_account_info(),
            authority: self.initializer.to_account_info(),
        };

        let seeds = ["mint".as_bytes(), &[bumps.mint]];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        mint_to(cpi_ctx, 10 * 10u64.pow(6))?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(title: String)]
pub struct UpdateMovieReview<'info> {
    #[account(
        mut,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,
    )]
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateMovieReview<'info> {
    pub fn update_movie_review(&mut self, description: String, rating: u8) -> Result<()> {
        // check decsription length is valid
        require!(description.len() <= MAX_DESCRIPTION_LENGTH, MovieReviewError::DescriptionTooLong);

        // check rating is valid
        require!(rating > MIN_RATING && rating <= MAX_RATING, MovieReviewError::InvalidRating);

        self.movie_review.description = description;
        self.movie_review.rating = rating;

        Ok(())
    }
}


#[derive(Accounts)]
#[instruction(title: String)]
pub struct DeleteMovieReview<'info> {
    #[account(
        mut,
        seeds=[title.as_bytes(), initializer.key().as_ref()],
        bump,
        close=initializer
    )]
    pub movie_review: Account<'info, MovieAccountState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
}


#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(
        init,
        payer = user,
        seeds = [b"mint"],
        bump,
        mint::decimals = 6,
        mint::authority = user,
    )]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeMint<'info> {
    pub fn init(&mut self) -> Result<()> {
        msg!("Token Mint Initialized");
        Ok(())
    }
}

/*
    For the MovieAccountState account, since it is dynamic, we implement the Space trait to calculate the space required for the account.
    We add the STRING_LENGTH_PREFIX twice to the space to account for the title and description string prefix.
    We need to add the length of the title and description to the space upon initialization.
 */
#[account]
pub struct MovieAccountState {
    pub reviewer: Pubkey,
    pub rating: u8,
    pub title: String,
    pub description: String,
}

impl Space for MovieAccountState {
    const INIT_SPACE: usize = ANCHOR_DISCRIMINATOR + PUBKEY_SIZE + U8_SIZE + STRING_LENGTH_PREFIX + STRING_LENGTH_PREFIX;
}

#[error_code]
enum MovieReviewError {
    #[msg("Rating must be between 1 and 5")]
    InvalidRating,
    #[msg("Movie Title too long")]
    TitleTooLong,
    #[msg("Movie Description too long")]
    DescriptionTooLong,
}
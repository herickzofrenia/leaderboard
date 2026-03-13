use anchor_lang::prelude::*;

declare_id!("GDyukzRzBaZTh9EHuTMmJQwsR6HWuQKQN9BVk6v8xEfn");

#[program]
pub mod leaderboard {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, name: String, max_entries: u8) -> Result<()> {
        require!(name.len() <= 32, LeaderboardError::NameTooLong);
        require!(max_entries > 0 && max_entries <= 100, LeaderboardError::InvalidMaxEntries);

        let lb = &mut ctx.accounts.leaderboard;
        lb.admin = ctx.accounts.admin.key();
        lb.name = name;
        lb.max_entries = max_entries;
        lb.bump = ctx.bumps.leaderboard;
        lb.entries = Vec::new();

        msg!("Leaderboard '{}' created!", lb.name);
        Ok(())
    }

    pub fn submit_score(ctx: Context<SubmitScore>, player_name: String, score: u64) -> Result<()> {
        require!(player_name.len() <= 32, LeaderboardError::NameTooLong);

        let lb = &mut ctx.accounts.leaderboard;
        let player = ctx.accounts.player.key();
        let timestamp = Clock::get()?.unix_timestamp;

        if let Some(entry) = lb.entries.iter_mut().find(|e| e.player == player) {
            if score > entry.score {
                entry.score = score;
                entry.player_name = player_name;
                entry.timestamp = timestamp;
                msg!("Score updated to {}", score);
            } else {
                msg!("Score not updated, current score is higher");
            }
        } else {
            require!(
                lb.entries.len() < lb.max_entries as usize,
                LeaderboardError::LeaderboardFull
            );
            lb.entries.push(ScoreEntry {
                player,
                player_name,
                score,
                timestamp,
            });
            msg!("New score {} submitted!", score);
        }

        lb.entries.sort_by(|a, b| b.score.cmp(&a.score));
        Ok(())
    }

    pub fn remove_player(ctx: Context<AdminAction>, player: Pubkey) -> Result<()> {
        let lb = &mut ctx.accounts.leaderboard;
        lb.entries.retain(|e| e.player != player);
        msg!("Player removed from leaderboard");
        Ok(())
    }

    pub fn reset(ctx: Context<AdminAction>) -> Result<()> {
        let lb = &mut ctx.accounts.leaderboard;
        lb.entries.clear();
        msg!("Leaderboard reset!");
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = Leaderboard::space(),
        seeds = [b"leaderboard", admin.key().as_ref(), name.as_bytes()],
        bump
    )]
    pub leaderboard: Account<'info, Leaderboard>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SubmitScore<'info> {
    #[account(mut, has_one = admin @ LeaderboardError::Unauthorized)]
    pub leaderboard: Account<'info, Leaderboard>,
    pub admin: Signer<'info>,
    /// CHECK: just storing the pubkey
    pub player: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct AdminAction<'info> {
    #[account(mut, has_one = admin @ LeaderboardError::Unauthorized)]
    pub leaderboard: Account<'info, Leaderboard>,
    pub admin: Signer<'info>,
}

#[account]
pub struct Leaderboard {
    pub admin: Pubkey,
    pub name: String,
    pub max_entries: u8,
    pub bump: u8,
    pub entries: Vec<ScoreEntry>,
}

impl Leaderboard {
    pub fn space() -> usize {
        8 +     // discriminator
        32 +    // admin
        4 + 32 + // name
        1 +     // max_entries
        1 +     // bump
        4 + (100 * ScoreEntry::size()) // entries vec
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ScoreEntry {
    pub player: Pubkey,
    pub player_name: String,
    pub score: u64,
    pub timestamp: i64,
}

impl ScoreEntry {
    pub fn size() -> usize {
        32 +        // player pubkey
        4 + 32 +    // player_name
        8 +         // score
        8           // timestamp
    }
}

#[error_code]
pub enum LeaderboardError {
    #[msg("Name too long (max 32 chars)")]
    NameTooLong,
    #[msg("Invalid max entries (1-100)")]
    InvalidMaxEntries,
    #[msg("Leaderboard is full")]
    LeaderboardFull,
    #[msg("Unauthorized")]
    Unauthorized,
}

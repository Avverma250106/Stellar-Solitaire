#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short, Address};

// Structure to track game statistics
#[contracttype]
#[derive(Clone)]
pub struct GameStats {
    pub total_games: u64,      // Total games played
    pub games_won: u64,         // Total games won
    pub games_in_progress: u64, // Active games
    pub total_moves: u64,       // Total moves across all games
}

// Symbol for referencing GameStats
const GAME_STATS: Symbol = symbol_short!("G_STATS");

// Mapping game_id to Game struct
#[contracttype]
pub enum GameBook {
    Game(u64)
}

// Counter for creating unique game IDs
const GAME_COUNT: Symbol = symbol_short!("G_COUNT");

// Structure for individual game data
#[contracttype]
#[derive(Clone)]
pub struct Game {
    pub game_id: u64,
    pub player: Address,
    pub moves: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub is_won: bool,
    pub is_active: bool,
}

#[contract]
pub struct SolitaireContract;

#[contractimpl]
impl SolitaireContract {

    // Function to start a new solitaire game
    pub fn start_game(env: Env, player: Address) -> u64 {
        player.require_auth();
        
        let mut game_count: u64 = env.storage().instance().get(&GAME_COUNT).unwrap_or(0);
        game_count += 1;
        
        let time = env.ledger().timestamp();
        let mut stats = Self::get_game_stats(env.clone());
        
        // Create new game record
        let new_game = Game {
            game_id: game_count,
            player: player.clone(),
            moves: 0,
            start_time: time,
            end_time: 0,
            is_won: false,
            is_active: true,
        };
        
        // Update statistics
        stats.total_games += 1;
        stats.games_in_progress += 1;
        
        // Store the game
        env.storage().instance().set(&GameBook::Game(game_count), &new_game);
        env.storage().instance().set(&GAME_STATS, &stats);
        env.storage().instance().set(&GAME_COUNT, &game_count);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "New Solitaire Game Started! Game ID: {}", game_count);
        
        game_count
    }

    // Function to record a move in the game
    pub fn record_move(env: Env, game_id: u64) {
        let mut game = Self::get_game_by_id(env.clone(), game_id);
        
        if !game.is_active {
            log!(&env, "Game is not active!");
            panic!("Game is not active!");
        }
        
        game.player.require_auth();
        
        // Increment move counter
        game.moves += 1;
        
        let mut stats = Self::get_game_stats(env.clone());
        stats.total_moves += 1;
        
        // Store updated game and stats
        env.storage().instance().set(&GameBook::Game(game_id), &game);
        env.storage().instance().set(&GAME_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Move recorded for Game ID: {}. Total moves: {}", game_id, game.moves);
    }

    // Function to complete a game (win or quit)
    pub fn complete_game(env: Env, game_id: u64, won: bool) {
        let mut game = Self::get_game_by_id(env.clone(), game_id);
        
        if !game.is_active {
            log!(&env, "Game is already completed!");
            panic!("Game is already completed!");
        }
        
        game.player.require_auth();
        
        let time = env.ledger().timestamp();
        
        // Update game record
        game.is_active = false;
        game.is_won = won;
        game.end_time = time;
        
        let mut stats = Self::get_game_stats(env.clone());
        stats.games_in_progress -= 1;
        
        if won {
            stats.games_won += 1;
        }
        
        // Store updated data
        env.storage().instance().set(&GameBook::Game(game_id), &game);
        env.storage().instance().set(&GAME_STATS, &stats);
        env.storage().instance().extend_ttl(5000, 5000);
        
        if won {
            log!(&env, "Congratulations! Game ID: {} won in {} moves!", game_id, game.moves);
        } else {
            log!(&env, "Game ID: {} ended with {} moves", game_id, game.moves);
        }
    }

    // Function to get overall game statistics
    pub fn get_game_stats(env: Env) -> GameStats {
        env.storage().instance().get(&GAME_STATS).unwrap_or(GameStats {
            total_games: 0,
            games_won: 0,
            games_in_progress: 0,
            total_moves: 0,
        })
    }

    // Function to get a specific game by ID
    pub fn get_game_by_id(env: Env, game_id: u64) -> Game {
        let key = GameBook::Game(game_id);
        
        env.storage().instance().get(&key).unwrap_or(Game {
            game_id: 0,
            player: Address::from_string(&String::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")),
            moves: 0,
            start_time: 0,
            end_time: 0,
            is_won: false,
            is_active: false,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Address, Env};

    #[test]
    fn test_start_and_complete_game() {
        let env = Env::default();
        let contract_id = env.register_contract(None, SolitaireContract);
        let client = SolitaireContractClient::new(&env, &contract_id);
        
        let player = Address::generate(&env);
        
        // Start a new game
        let game_id = client.start_game(&player);
        assert_eq!(game_id, 1);
        
        // Get game stats
        let stats = client.get_game_stats();
        assert_eq!(stats.total_games, 1);
        assert_eq!(stats.games_in_progress, 1);
        
        // Record some moves
        client.record_move(&game_id);
        client.record_move(&game_id);
        
        // Complete the game
        client.complete_game(&game_id, &true);
        
        // Check final stats
        let final_stats = client.get_game_stats();
        assert_eq!(final_stats.games_won, 1);
        assert_eq!(final_stats.games_in_progress, 0);
    }
}
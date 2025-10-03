use crate::hand_evaluator::{self, HandRank};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

// サーバーとクライアント間でやり取りされるメッセージの定義
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "payload")]
pub enum GameMessage {
    PlayerAction(PlayerAction),
    GameStateUpdate(GameState),
    DealHand(DealHandPayload),
    ChatMessage(String),
}

// クライアントから送られてくるアクション
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum PlayerAction {
    StartGame,
    Fold,
    Call,
    Bet { amount: u32 },
}

// サーバーから特定のプレイヤーに手札を送るためのペイロード
#[derive(Serialize, Deserialize, Debug)]
pub struct DealHandPayload {
    pub cards: Vec<String>,
}

// プレイヤーの状態
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub username: String,
    pub stack: u32,
    pub hand: Vec<String>,
    pub is_active: bool,
    pub current_bet: u32,
}

// ゲーム全体の現在の状態
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub community_cards: Vec<String>,
    pub pot: u32,
    pub current_turn_username: Option<String>,
    pub status: String, // e.g., "Waiting", "Pre-flop", "Flop", "Turn", "River", "Showdown"
    pub current_bet: u32,
    pub dealer_index: usize,
    #[serde(skip)] // デッキ情報はクライアントに送らない
    deck: Vec<String>,
}

impl GameState {
    // 新しいゲームを作成
    pub fn new() -> Self {
        GameState {
            players: Vec::new(),
            community_cards: Vec::new(),
            pot: 0,
            current_turn_username: None,
            status: "Waiting".to_string(),
            current_bet: 0,
            dealer_index: 0,
            deck: Vec::new(),
        }
    }

    // プレイヤーをゲームに追加
    pub fn add_player(&mut self, username: String) {
        if !self.players.iter().any(|p| p.username == username) {
            self.players.push(Player {
                username,
                stack: 1000, // 初期スタック
                hand: Vec::new(),
                is_active: false,
                current_bet: 0,
            });
        }
    }

    // ゲームを開始する
    pub fn start_game(&mut self) {
        if self.status != "Waiting" || self.players.len() < 2 {
            return; // 待機中でなければ開始しない
        }

        self.dealer_index = (self.dealer_index + 1) % self.players.len();

        self.deck = create_deck();
        self.deck.shuffle(&mut thread_rng());

        // 手札を配る
        for player in &mut self.players {
            player.hand = vec![self.deck.pop().unwrap(), self.deck.pop().unwrap()];
            player.is_active = true;
            player.current_bet = 0;
        }

        // ★ ブラインドの処理
        let small_blind_index = (self.dealer_index + 1) % self.players.len();
        let big_blind_index = (self.dealer_index + 2) % self.players.len();

        let sb_amount = 10;
        let bb_amount = 20;

        // スモールブラインド
        let sb_player = &mut self.players[small_blind_index];
        let sb_bet = std::cmp::min(sb_amount, sb_player.stack);
        sb_player.stack -= sb_bet;
        sb_player.current_bet = sb_bet;
        self.pot += sb_bet;

        // ビッグブラインド
        let bb_player = &mut self.players[big_blind_index];
        let bb_bet = std::cmp::min(bb_amount, bb_player.stack);
        bb_player.stack -= bb_bet;
        bb_player.current_bet = bb_bet;
        self.pot += bb_bet;

        self.current_bet = bb_amount;
        self.status = "Pre-flop".to_string();

        // BBの次の人からアクション開始
        self.current_turn_username = Some(
            self.players[(big_blind_index + 1) % self.players.len()]
                .username
                .clone(),
        );
    }

    // プレイヤーのアクションを処理する
    pub fn handle_action(&mut self, username: &str, action: PlayerAction) {
        if self.current_turn_username.as_deref() != Some(username) {
            return;
        }

        let player_index = self
            .players
            .iter()
            .position(|p| p.username == username)
            .unwrap();

        match action {
            PlayerAction::Fold => {
                self.players[player_index].is_active = false;
            }
            PlayerAction::Call => {
                let to_call = self.current_bet - self.players[player_index].current_bet;
                if to_call > 0 && self.players[player_index].stack >= to_call {
                    let player = &mut self.players[player_index];
                    player.stack -= to_call;
                    player.current_bet += to_call;
                    self.pot += to_call;
                }
            }
            PlayerAction::Bet { amount } => {
                // ベット額が現在のベット額以上か、かつスタックの範囲内かチェック
                if amount >= self.current_bet
                    && self.players[player_index].stack
                        >= (amount - self.players[player_index].current_bet)
                {
                    let player = &mut self.players[player_index];
                    let bet_increase = amount - player.current_bet;
                    player.stack -= bet_increase;
                    player.current_bet = amount;
                    self.pot += bet_increase;
                    self.current_bet = amount;
                } else {
                    return; // 無効なベット
                }
            }
            _ => {}
        }

        // ハンドが終了したかチェック
        if self.check_hand_over() {
            return;
        }

        // ベッティングラウンド終了チェック
        if self.check_betting_round_over() {
            self.proceed_to_next_stage();
        } else {
            self.advance_turn();
        }
    }

    fn proceed_to_next_stage(&mut self) {
        // 次のラウンドの準備
        self.current_bet = 0;
        for p in &mut self.players {
            if p.is_active {
                p.current_bet = 0;
            }
        }
        // ディーラーの次のアクティブなプレイヤーからターンを再開
        self.current_turn_username = self
            .players
            .iter()
            .cycle()
            .skip(self.dealer_index + 1)
            .find(|p| p.is_active)
            .map(|p| p.username.clone());

        match self.status.as_str() {
            "Pre-flop" => {
                self.status = "Flop".to_string();
                self.community_cards.push(self.deck.pop().unwrap());
                self.community_cards.push(self.deck.pop().unwrap());
                self.community_cards.push(self.deck.pop().unwrap());
            }
            "Flop" => {
                self.status = "Turn".to_string();
                self.community_cards.push(self.deck.pop().unwrap());
            }
            "Turn" => {
                self.status = "River".to_string();
                self.community_cards.push(self.deck.pop().unwrap());
            }
            "River" => {
                self.status = "Showdown".to_string();
                self.determine_winner(); // 勝者判定
            }
            _ => {}
        }
    }

    // 勝者を決定する（簡易的なハイカード判定）
    fn determine_winner(&mut self) {
        let mut best_rank: Option<HandRank> = None;
        let mut winners: Vec<String> = Vec::new();

        for player in self.players.iter().filter(|p| p.is_active) {
            let mut seven_cards_str: Vec<String> = self.community_cards.clone();
            seven_cards_str.extend(player.hand.clone());

            let cards = hand_evaluator::parse_cards(&seven_cards_str);
            if let Some(current_rank) = hand_evaluator::evaluate_hand(&cards) {
                if best_rank.is_none() || current_rank > *best_rank.as_ref().unwrap() {
                    best_rank = Some(current_rank);
                    winners.clear();
                    winners.push(player.username.clone());
                } else if current_rank == *best_rank.as_ref().unwrap() {
                    // 引き分けの場合
                    winners.push(player.username.clone());
                }
            }
        }

        if !winners.is_empty() {
            let pot_share = self.pot / winners.len() as u32;
            for winner_name in winners {
                if let Some(winner_player) =
                    self.players.iter_mut().find(|p| p.username == winner_name)
                {
                    winner_player.stack += pot_share;
                }
            }
        }

        self.status = "Waiting".to_string();
        self.current_turn_username = None;
    }

    // ベッティングラウンドが終了したか判定
    fn check_betting_round_over(&self) -> bool {
        // アクティブなプレイヤー全員が同じ額をベットしていればラウンド終了
        self.players
            .iter()
            .filter(|p| p.is_active)
            .all(|p| p.current_bet == self.current_bet)
    }

    // ハンドが終了したかチェックし、終了していればポットを勝者に渡す
    fn check_hand_over(&mut self) -> bool {
        let active_players: Vec<_> = self.players.iter().filter(|p| p.is_active).collect();
        if active_players.len() == 1 {
            let winner_username = active_players[0].username.clone();
            if let Some(winner) = self
                .players
                .iter_mut()
                .find(|p| p.username == winner_username)
            {
                winner.stack += self.pot;
            }
            // ゲーム状態をリセットして次のゲームを待つ
            self.status = "Waiting".to_string();
            self.current_turn_username = None;
            self.pot = 0;
            self.current_bet = 0;
            for p in &mut self.players {
                p.is_active = false;
                p.hand.clear();
            }
            return true;
        }
        false
    }

    // ターンを次のアクティブなプレイヤーに進める
    fn advance_turn(&mut self) {
        let current_turn_username = match self.current_turn_username.clone() {
            Some(name) => name,
            None => return,
        };

        let current_index = self
            .players
            .iter()
            .position(|p| p.username == current_turn_username);

        if let Some(index) = current_index {
            // 次のアクティブなプレイヤーを探す
            for i in 1..=self.players.len() {
                let next_index = (index + i) % self.players.len();
                if self.players[next_index].is_active {
                    self.current_turn_username = Some(self.players[next_index].username.clone());
                    return;
                }
            }
        }
        // アクティブなプレイヤーが一人しかいない場合など
        self.current_turn_username = None;
    }

    // 他のプレイヤーに手札情報が見えないようにサニタイズ（無害化）したGameStateを返す
    pub fn sanitized(&self) -> Self {
        let mut sanitized_state = self.clone();
        for player in &mut sanitized_state.players {
            player.hand = Vec::new();
        }
        sanitized_state
    }
}

// 52枚のカードデッキを作成するヘルパー関数
fn create_deck() -> Vec<String> {
    let suits = ["H", "D", "C", "S"]; // Hearts, Diamonds, Clubs, Spades
    let ranks = [
        "2", "3", "4", "5", "6", "7", "8", "9", "T", "J", "Q", "K", "A",
    ];
    let mut deck = Vec::new();
    for suit in suits.iter() {
        for rank in ranks.iter() {
            deck.push(format!("{}{}", rank, suit));
        }
    }
    deck
}

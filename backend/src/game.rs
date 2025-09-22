use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

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
    Raise { amount: u32 },
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
}

// ゲーム全体の現在の状態
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
    pub community_cards: Vec<String>,
    pub pot: u32,
    pub current_turn_username: Option<String>,
    pub status: String, // e.g., "Waiting", "Pre-flop", "Flop", "Turn", "River", "Showdown"
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
        }
    }

    // プレイヤーをゲームに追加
    pub fn add_player(&mut self, username: String) {
        if !self.players.iter().any(|p| p.username == username) {
            self.players.push(Player {
                username,
                stack: 1000, // 初期スタック
                hand: Vec::new(),
                is_active: true,
            });
        }
    }

    // ゲームを開始する
    pub fn start_game(&mut self) {
        if self.status != "Waiting" || self.players.len() < 2 {
            return; // 待機中でなければ開始しない
        }

        let mut deck = create_deck();
        deck.shuffle(&mut thread_rng());

        // 手札を配る
        for player in &mut self.players {
            player.hand = vec![deck.pop().unwrap(), deck.pop().unwrap()];
            player.is_active = true;
        }

        self.status = "Pre-flop".to_string();
        self.community_cards.clear();
        self.pot = 0;
        // TODO: ブラインドの処理、最初のターンのプレイヤー決定などを追加
        self.current_turn_username = self.players.get(0).map(|p| p.username.clone());
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

export interface Room {
  id: string;
  name: string;
  status: 'waiting' | 'playing' | 'finished';
  created_by: string;
  created_at: string;
}

// プレイヤーの状態
export interface Player {
  username: string;
  stack: number;
  hand: string[];
  is_active: boolean;
  current_bet: number;
}

// ゲーム全体の状態
export interface GameState {
  players: Player[];
  community_cards: string[];
  pot: number;
  current_turn_username: string | null;
  status: string;
  current_bet: number;
  dealer_index: number;
  winner_message: string | null;
}

// WebSocketで送受信するメッセージの型
export type GameMessage =
  | { type: 'ChatMessage'; payload: string }
  | { type: 'GameStateUpdate'; payload: GameState }
  | { type: 'DealHand'; payload: { cards: string[] } };
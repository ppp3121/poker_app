use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Diamond,
    Club,
    Heart,
    Spade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

// Ordを手動実装してカードの強さを比較できるようにする
impl PartialOrd for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank.cmp(&other.rank)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HandRank {
    HighCard(Rank, Rank, Rank, Rank, Rank),
    OnePair(Rank, Rank, Rank, Rank),
    TwoPair(Rank, Rank, Rank),
    ThreeOfAKind(Rank, Rank, Rank),
    Straight(Rank),
    Flush(Rank, Rank, Rank, Rank, Rank),
    FullHouse(Rank, Rank),
    FourOfAKind(Rank, Rank),
    StraightFlush(Rank),
    RoyalFlush,
}

impl fmt::Display for HandRank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HandRank::RoyalFlush => write!(f, "ロイヤルフラッシュ"),
            HandRank::StraightFlush(_) => write!(f, "ストレートフラッシュ"),
            HandRank::FourOfAKind(_, _) => write!(f, "フォーカード"),
            HandRank::FullHouse(_, _) => write!(f, "フルハウス"),
            HandRank::Flush(_, _, _, _, _) => write!(f, "フラッシュ"),
            HandRank::Straight(_) => write!(f, "ストレート"),
            HandRank::ThreeOfAKind(_, _, _) => write!(f, "スリーカード"),
            HandRank::TwoPair(_, _, _) => write!(f, "ツーペア"),
            HandRank::OnePair(_, _, _, _) => write!(f, "ワンペア"),
            HandRank::HighCard(_, _, _, _, _) => write!(f, "ハイカード"),
        }
    }
}

// カードの文字列("As", "Td", "7c")をCard構造体に変換
pub fn parse_cards(card_strs: &[String]) -> Vec<Card> {
    card_strs.iter().filter_map(|s| parse_card(s)).collect()
}

fn parse_card(s: &str) -> Option<Card> {
    if s.len() != 2 {
        return None;
    }
    let mut chars = s.chars();
    let rank = match chars.next()? {
        '2' => Rank::Two,
        '3' => Rank::Three,
        '4' => Rank::Four,
        '5' => Rank::Five,
        '6' => Rank::Six,
        '7' => Rank::Seven,
        '8' => Rank::Eight,
        '9' => Rank::Nine,
        'T' => Rank::Ten,
        'J' => Rank::Jack,
        'Q' => Rank::Queen,
        'K' => Rank::King,
        'A' => Rank::Ace,
        _ => return None,
    };
    let suit = match chars.next()? {
        'H' => Suit::Heart,
        'D' => Suit::Diamond,
        'C' => Suit::Club,
        'S' => Suit::Spade,
        _ => return None,
    };
    Some(Card { rank, suit })
}

// 7枚のカードから最強の5枚の役を見つける
pub fn evaluate_hand(seven_cards: &[Card]) -> Option<HandRank> {
    if seven_cards.len() != 7 {
        return None;
    }
    let mut best_rank: Option<HandRank> = None;

    // 7枚から5枚を選ぶ全ての組み合わせ (21通り) を試す
    for i in 0..seven_cards.len() {
        for j in (i + 1)..seven_cards.len() {
            let mut hand: Vec<Card> = seven_cards.to_vec();
            // 2枚を除外して5枚にする
            hand.remove(j);
            hand.remove(i);

            let current_rank = find_best_rank_for_5_cards(&mut hand);

            if best_rank.is_none() || current_rank > *best_rank.as_ref().unwrap() {
                best_rank = Some(current_rank);
            }
        }
    }
    best_rank
}

// 5枚のカードの役を判定する
fn find_best_rank_for_5_cards(hand: &mut [Card]) -> HandRank {
    hand.sort_by(|a, b| b.rank.cmp(&a.rank)); // 降順ソート
    let ranks: Vec<Rank> = hand.iter().map(|c| c.rank).collect();
    let suits: Vec<Suit> = hand.iter().map(|c| c.suit).collect();

    let is_flush = suits.windows(2).all(|w| w[0] == w[1]);
    let is_straight = ranks.windows(2).all(|w| w[0] as i8 == w[1] as i8 + 1);

    // エースを1と見なすストレート (A-2-3-4-5) の特殊ケース
    let is_ace_low_straight = ranks[0] == Rank::Ace
        && ranks[1] == Rank::Five
        && ranks[2] == Rank::Four
        && ranks[3] == Rank::Three
        && ranks[4] == Rank::Two;

    if is_straight && is_flush {
        if ranks[0] == Rank::Ace {
            return HandRank::RoyalFlush;
        }
        return HandRank::StraightFlush(ranks[0]);
    }
    if is_ace_low_straight && is_flush {
        return HandRank::StraightFlush(Rank::Five);
    }

    let mut counts: HashMap<Rank, u8> = HashMap::new();
    for rank in &ranks {
        *counts.entry(*rank).or_insert(0) += 1;
    }

    let mut pairs = Vec::new();
    let mut threes = Vec::new();
    let mut fours = Vec::new();
    let mut kickers = Vec::new();

    for (rank, count) in counts.iter() {
        match count {
            4 => fours.push(*rank),
            3 => threes.push(*rank),
            2 => pairs.push(*rank),
            _ => kickers.push(*rank),
        }
    }

    pairs.sort_by(|a, b| b.cmp(a));
    kickers.sort_by(|a, b| b.cmp(a));

    if !fours.is_empty() {
        return HandRank::FourOfAKind(fours[0], kickers[0]);
    }
    if !threes.is_empty() && !pairs.is_empty() {
        return HandRank::FullHouse(threes[0], pairs[0]);
    }
    if is_flush {
        return HandRank::Flush(ranks[0], ranks[1], ranks[2], ranks[3], ranks[4]);
    }
    if is_straight {
        return HandRank::Straight(ranks[0]);
    }
    if is_ace_low_straight {
        return HandRank::Straight(Rank::Five);
    }
    if !threes.is_empty() {
        return HandRank::ThreeOfAKind(threes[0], kickers[0], kickers[1]);
    }
    if pairs.len() >= 2 {
        return HandRank::TwoPair(pairs[0], pairs[1], kickers[0]);
    }
    if !pairs.is_empty() {
        return HandRank::OnePair(pairs[0], kickers[0], kickers[1], kickers[2]);
    }
    HandRank::HighCard(ranks[0], ranks[1], ranks[2], ranks[3], ranks[4])
}

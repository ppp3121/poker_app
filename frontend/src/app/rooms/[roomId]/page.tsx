'use client';

import { useEffect, useState, useRef } from 'react';
import { useParams } from 'next/navigation';
import { useUserStore } from '@/store/userStore';
import { Room, GameState, GameMessage } from '@/types';
import { useRouter } from 'next/navigation';

export default function RoomPage() {
  const { isLoggedIn, isInitialized, username } = useUserStore();
  const router = useRouter();
  const params = useParams();
  const roomId = params.roomId as string;

  const [room, setRoom] = useState<Room | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const [gameState, setGameState] = useState<GameState | null>(null);
  const [myHand, setMyHand] = useState<string[]>([]);

  const [betAmount, setBetAmount] = useState<number>(10);
  const myPlayerState = gameState?.players.find(p => p.username === username);
  const handleNextHand = () => handlePlayerAction({ action: 'NextHand' });

  // WebSocketメッセージを管理するためのState
  const [chatMessages, setChatMessages] = useState<string[]>([]);
  const [chatInput, setChatInput] = useState<string>('');
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [connectionStatus, setConnectionStatus] = useState<string>('接続中...');
  const reconnectAttempt = useRef(0);

  useEffect(() => {
    // 認証状態のチェック
    if (isInitialized && !isLoggedIn) {
      router.push('/login');
      return;
    }

    if (isInitialized && isLoggedIn && roomId) {
      const fetchRoom = async () => {
        try {
          const response = await fetch(`http://localhost:8000/api/rooms/${roomId}`, {
            credentials: 'include',
          });

          if (!response.ok) {
            if (response.status === 404) {
              throw new Error('指定されたルームが見つかりません。');
            }
            throw new Error('ルーム情報の取得に失敗しました。');
          }

          const data: Room = await response.json();
          setRoom(data);

        } catch (err) {
          setError(err instanceof Error ? err.message : '不明なエラーです。');
        } finally {
          setIsLoading(false);
        }
      };

      fetchRoom();
    }
  }, [isInitialized, isLoggedIn, router, roomId]);

  // WebSocket接続用のuseEffect
  useEffect(() => {
    if (!isInitialized || !isLoggedIn || !roomId) return;

    const connectWebSocket = () => {
      setConnectionStatus('接続中...');
      const socket = new WebSocket(`ws://localhost:8000/api/ws/rooms/${roomId}`);

      socket.onopen = () => {
        setConnectionStatus('接続済み');
        setWs(socket);
        reconnectAttempt.current = 0;
      };

      socket.onmessage = (event) => {
        // サーバーからのメッセージを正しく処理する
        try {
          const message: GameMessage = JSON.parse(event.data);
          switch (message.type) {
            case 'ChatMessage':
              setChatMessages((prev) => [...prev, message.payload]);
              break;
            case 'GameStateUpdate':
              setGameState(message.payload);
              break;
            case 'DealHand':
              setMyHand(message.payload.cards);
              break;
          }
        } catch (e) {
          setChatMessages((prev) => [...prev, event.data]);
        }
      };

      socket.onclose = () => {
        setConnectionStatus('切断');
        if (reconnectAttempt.current < 5) {
          reconnectAttempt.current++;
          setTimeout(connectWebSocket, 3000);
        } else {
          setError('サーバーとの接続が切れました。ページを更新してください。');
        }
      };

      socket.onerror = (err) => {
        console.error('WebSocket error:', err);
        setConnectionStatus('エラー');
      };
    };

    connectWebSocket();

    return () => {
      reconnectAttempt.current = 5;
      ws?.close();
    };
  }, [isInitialized, isLoggedIn, roomId]);

  // 「ゲーム開始」ボタンの処理
  const handleStartGame = () => {
    if (ws?.readyState === WebSocket.OPEN) {
      const message = {
        type: 'PlayerAction',
        payload: { action: 'StartGame' },
      };
      ws.send(JSON.stringify(message));
    }
  };

  // チャット送信をJSON形式に修正
  const handleSendMessage = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (ws?.readyState === WebSocket.OPEN && chatInput.trim()) {
      const message = {
        type: 'ChatMessage',
        payload: chatInput,
      };
      ws.send(JSON.stringify(message));
      setChatInput('');
    }
  };

  // プレイヤーのアクションを送信する関数
  const handlePlayerAction = (action: any) => {
    if (ws?.readyState === WebSocket.OPEN) {
      const message = {
        type: 'PlayerAction',
        payload: action,
      };
      ws.send(JSON.stringify(message));
    }
  };



  if (isLoading || !isInitialized) {
    return <div>読み込み中...</div>;
  }

  if (error && !room) {
    return <div style={{ color: 'red' }}>エラー: {error}</div>;
  }

  if (!room) {
    return <div>ルーム情報が見つかりません。</div>;
  }

  return (
    <main style={{ display: 'flex', height: 'calc(100vh - 73px)' }}>
      <div style={{ flex: 1, padding: '2rem', overflowY: 'auto' }}>
        <h1>{room?.name}</h1>
        <p>ルームID: {room?.id}</p>
        <hr style={{ margin: '1rem 0' }} />

        {gameState?.status === 'Waiting' && (
          <button onClick={handleStartGame} style={{ padding: '0.5rem 1rem', marginBottom: '1rem' }}>
            ゲーム開始
          </button>
        )}

        {/* ショーダウン時の表示 */}
        {gameState?.status === 'Showdown' && (
          <div style={{ marginTop: '1rem', padding: '1rem', border: '2px solid yellow', backgroundColor: '#330' }}>
            <h2>ショーダウン</h2>
            <p style={{ color: 'yellow', fontSize: '1.2rem' }}>{gameState.winner_message}</p>
            <button onClick={handleNextHand} style={{ padding: '0.5rem 1rem', marginTop: '1rem' }}>
              次のハンドへ
            </button>
          </div>
        )}

        {/* アクションボタンエリア */}
        {gameState && gameState.current_turn_username === username && myPlayerState && (
          <div style={{ marginTop: '1rem', padding: '1rem', border: '2px solid lightgreen' }}>
            <h2>Your Turn!</h2>
            <div style={{ display: 'flex', gap: '1rem', marginTop: '0.5rem' }}>
              <button onClick={() => handlePlayerAction({ action: 'Fold' })} style={{ padding: '0.5rem 1rem' }}>
                フォールド
              </button>

              {/* 状況に応じてチェック/コールを出し分け */}
              {myPlayerState.current_bet < gameState.current_bet ? (
                <button onClick={() => handlePlayerAction({ action: 'Call' })} style={{ padding: '0.5rem 1rem', backgroundColor: '#2a4' }}>
                  コール ({gameState.current_bet})
                </button>
              ) : (
                <button onClick={() => handlePlayerAction({ action: 'Call' })} style={{ padding: '0.5rem 1rem', backgroundColor: '#44b' }}>
                  チェック
                </button>
              )}

              {/* ベット/レイズボタン */}
              <div>
                <input
                  type="number"
                  value={betAmount}
                  onChange={(e) => setBetAmount(Number(e.target.value))}
                  style={{ width: '80px', color: 'black', padding: '0.5rem' }}
                  min={gameState.current_bet * 2} // 簡単なバリデーション
                />
                <button onClick={() => handlePlayerAction({ action: 'Bet', amount: betAmount })} style={{ padding: '0.5rem 1rem', backgroundColor: '#c33' }}>
                  {gameState.current_bet > 0 ? 'レイズ' : 'ベット'}
                </button>
              </div>
            </div>
          </div>
        )}

        <div style={{ marginTop: '1rem', border: '1px solid #ccc', padding: '1rem' }}>
          <h2>ゲームテーブル (Status: {gameState?.status})</h2>
          <div><strong>Community Cards:</strong> {gameState?.community_cards.join(', ')}</div>
          <div><strong>Pot:</strong> {gameState?.pot}</div>
          <div style={{ fontSize: '1.2rem', fontWeight: 'bold', margin: '1rem 0', color: 'lightblue' }}>
            My Hand: {myHand.join(', ')}
          </div>
          <hr />
          <h3>Players:</h3>
          <ul>
            {gameState?.players.map((p, index) => {
              let color = '#DDD';
              if (p.username === gameState.current_turn_username) {
                color = 'lightgreen';
              } else if (!p.is_active && gameState.status !== 'Waiting') {
                color = 'grey';
              }

              const isDealer = index === gameState.dealer_index;
              const isSB = index === (gameState.dealer_index + 1) % gameState.players.length;
              const isBB = index === (gameState.dealer_index + 2) % gameState.players.length;

              return (
                <li key={p.username} style={{ color, fontWeight: isDealer ? 'bold' : 'normal' }}>
                  {isDealer && 'D '}{isSB && 'SB '}{isBB && 'BB '}
                  {p.username} (Stack: {p.stack}) [Bet: {p.current_bet}]
                  {/* ショーダウン時に手札を表示 */}
                  {gameState.status === 'Showdown' && p.hand.length > 0 && ` [Hand: ${p.hand.join(', ')}]`}
                  {p.username === username && ' (You)'}
                  {!p.is_active && gameState.status !== 'Waiting' && ' (Folded)'}
                  {p.username === gameState.current_turn_username && ' (Turn)'}
                </li>
              );
            })}
          </ul>
        </div>
      </div>

      <div style={{ width: '350px', borderLeft: '1px solid #555', padding: '2rem', display: 'flex', flexDirection: 'column' }}>
        <h2>チャット</h2>
        <p>接続状態: {connectionStatus}</p>
        <div style={{ flex: 1, overflowY: 'auto', border: '1px solid #555', padding: '0.5rem', margin: '0.5rem 0' }}>
          {[...chatMessages].reverse().map((msg, index) => (
            <p key={index}>{msg}</p>
          ))}
        </div>
        <form onSubmit={handleSendMessage}>
          <input
            type="text"
            value={chatInput}
            onChange={(e) => setChatInput(e.target.value)}
            style={{ width: '100%', padding: '0.5rem', color: 'black' }}
            placeholder="メッセージを入力..."
            disabled={ws?.readyState !== WebSocket.OPEN}
          />
          <button type="submit" style={{ width: '100%', marginTop: '0.5rem' }} disabled={ws?.readyState !== WebSocket.OPEN}>
            送信
          </button>
        </form>
      </div>
    </main>
  );
}
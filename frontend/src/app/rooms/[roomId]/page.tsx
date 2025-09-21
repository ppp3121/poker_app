'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import { useUserStore } from '@/store/userStore';
import { Room } from '@/types';
import { useRouter } from 'next/navigation';

export default function RoomPage() {
  const { isLoggedIn, isInitialized, username } = useUserStore();
  const router = useRouter();
  const params = useParams();
  const roomId = params.roomId as string;

  const [room, setRoom] = useState<Room | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // WebSocketメッセージを管理するためのState
  const [messages, setMessages] = useState<string[]>([]);
  const [chatInput, setChatInput] = useState<string>('');
  const [ws, setWs] = useState<WebSocket | null>(null);

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
    if (!isInitialized || !isLoggedIn || !roomId) {
      return;
    }

    // WebSocketはHttpOnly Cookieを直接送信できないため、
    // ここでは認証済みのセッションに紐づくWebSocket接続として扱います。
    // (バックエンドのws_handlerでClaimsを取得しているので認証はできている)
    const socket = new WebSocket(`ws://localhost:8000/api/ws/rooms/${roomId}`);

    socket.onopen = () => {
      console.log('WebSocket connected');
      setWs(socket);
    };

    socket.onmessage = (event) => {
      setMessages((prevMessages) => [...prevMessages, event.data]);
    };

    socket.onclose = () => {
      console.log('WebSocket disconnected');
      setWs(null);
    };

    socket.onerror = (error) => {
      console.error('WebSocket error:', error);
      setError('リアルタイム接続に失敗しました。');
    };

    // コンポーネントがアンマウントされるときに接続を閉じる
    return () => {
      socket.close();
    };
  }, [isInitialized, isLoggedIn, roomId]);

  const handleSendMessage = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (ws && chatInput.trim()) {
      ws.send(chatInput); // 入力されたメッセージを送信
      setChatInput('');
    }
  };

  if (isLoading || !isInitialized) {
    return <div>読み込み中...</div>;
  }

  if (error) {
    return <div style={{ color: 'red' }}>エラー: {error}</div>;
  }

  if (!room) {
    return <div>ルーム情報が見つかりません。</div>;
  }

  return (
    <main style={{ display: 'flex', height: 'calc(100vh - 73px)' }}>
      <div style={{ flex: 1, padding: '2rem' }}>
        <h1>{room.name}</h1>
        <p>ルームID: {room.id}</p>
        <p>ステータス: {room.status}</p>

        {/* ここに将来的にゲームコンポーネントが配置されます */}
        <div style={{ marginTop: '2rem', border: '1px solid #ccc', padding: '1rem' }}>
          <h2>ゲームテーブル</h2>
          <p>（ここにポーカーのテーブルが表示されます）</p>
        </div>
      </div>

      {/* チャットとメッセージ表示エリア */}
      <div style={{ width: '350px', borderLeft: '1px solid #555', padding: '2rem', display: 'flex', flexDirection: 'column' }}>
        <h2>チャット</h2>
        <div style={{ flex: 1, overflowY: 'auto', border: '1px solid #555', padding: '0.5rem', marginBottom: '1rem', display: 'flex', flexDirection: 'column-reverse' }}>
          {/* メッセージを逆順に表示して、常に最新が一番下に来るようにする */}
          <div>
            {[...messages].reverse().map((msg, index) => (
              <p key={index}>{msg}</p>
            ))}
          </div>
        </div>
        <form onSubmit={handleSendMessage}>
          <input
            type="text"
            value={chatInput}
            onChange={(e) => setChatInput(e.target.value)}
            style={{ width: '100%', padding: '0.5rem', color: 'black', boxSizing: 'border-box' }}
            placeholder="メッセージを入力..."
            disabled={!ws}
          />
          <button type="submit" style={{ width: '100%', marginTop: '0.5rem' }} disabled={!ws}>
            送信
          </button>
        </form>
      </div>
    </main>
  );
}
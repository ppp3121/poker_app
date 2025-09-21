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
  const [connectionStatus, setConnectionStatus] = useState<string>('接続中...');

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

  // WebSocket接続用のuseEffect（修正版）
  useEffect(() => {
    if (!isInitialized || !isLoggedIn || !roomId) {
      return;
    }

    const connectWebSocket = () => {
      console.log('WebSocket接続を開始します...');
      setConnectionStatus('接続中...');

      // まずCookieでの接続を試す
      const socket = new WebSocket(`ws://localhost:8000/api/ws/rooms/${roomId}`);

      socket.onopen = () => {
        console.log('WebSocket connected successfully');
        setConnectionStatus('接続済み');
        setWs(socket);
        setError(null); // エラーをクリア
      };

      socket.onmessage = (event) => {
        console.log('Received message:', event.data);
        setMessages((prevMessages) => [...prevMessages, event.data]);
      };

      socket.onclose = (event) => {
        console.log('WebSocket disconnected', event.code, event.reason);
        setConnectionStatus('切断');
        setWs(null);

        // 異常な切断の場合は再接続を試す
        if (event.code !== 1000 && event.code !== 1001) {
          console.log('再接続を試みます...');
          setTimeout(connectWebSocket, 3000);
        }
      };

      socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        setConnectionStatus('エラー');
        setError('リアルタイム接続に失敗しました。ページを再読み込みしてみてください。');
      };

      return socket;
    };

    const socket = connectWebSocket();

    // コンポーネントがアンマウントされるときに接続を閉じる
    return () => {
      if (socket) {
        console.log('WebSocket接続を閉じます');
        socket.close(1000, 'Component unmounting');
      }
    };
  }, [isInitialized, isLoggedIn, roomId]);

  const handleSendMessage = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (ws && ws.readyState === WebSocket.OPEN && chatInput.trim()) {
      console.log('Sending message:', chatInput);
      ws.send(chatInput);
      setChatInput('');
    } else {
      console.log('WebSocket not ready or message empty');
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
      <div style={{ flex: 1, padding: '2rem' }}>
        <h1>{room.name}</h1>
        <p>ルームID: {room.id}</p>
        <p>ステータス: {room.status}</p>

        {/* WebSocket接続状態を表示 */}
        <p style={{
          color: connectionStatus === '接続済み' ? 'green' :
            connectionStatus === 'エラー' ? 'red' : 'orange'
        }}>
          接続状態: {connectionStatus}
        </p>

        {error && (
          <div style={{
            color: 'red',
            backgroundColor: '#ffebee',
            padding: '1rem',
            borderRadius: '4px',
            marginBottom: '1rem'
          }}>
            {error}
          </div>
        )}

        {/* ここに将来的にゲームコンポーネントが配置されます */}
        <div style={{ marginTop: '2rem', border: '1px solid #ccc', padding: '1rem' }}>
          <h2>ゲームテーブル</h2>
          <p>（ここにポーカーのテーブルが表示されます）</p>
        </div>
      </div>

      {/* チャットとメッセージ表示エリア */}
      <div style={{
        width: '350px',
        borderLeft: '1px solid #555',
        padding: '2rem',
        display: 'flex',
        flexDirection: 'column'
      }}>
        <h2>チャット</h2>
        <div style={{
          flex: 1,
          overflowY: 'auto',
          border: '1px solid #555',
          padding: '0.5rem',
          marginBottom: '1rem',
          minHeight: '300px',
          maxHeight: '400px'
        }}>
          {messages.length === 0 ? (
            <p style={{ color: '#666', fontStyle: 'italic' }}>
              まだメッセージがありません
            </p>
          ) : (
            messages.map((msg, index) => (
              <p key={index} style={{ marginBottom: '0.5rem', wordBreak: 'break-word' }}>
                {msg}
              </p>
            ))
          )}
        </div>
        <form onSubmit={handleSendMessage}>
          <input
            type="text"
            value={chatInput}
            onChange={(e) => setChatInput(e.target.value)}
            style={{
              width: '100%',
              padding: '0.5rem',
              color: 'black',
              boxSizing: 'border-box',
              border: '1px solid #ccc',
              borderRadius: '4px'
            }}
            placeholder="メッセージを入力..."
            disabled={!ws || ws.readyState !== WebSocket.OPEN}
          />
          <button
            type="submit"
            style={{
              width: '100%',
              marginTop: '0.5rem',
              padding: '0.5rem',
              backgroundColor: (ws && ws.readyState === WebSocket.OPEN) ? '#007bff' : '#ccc',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: (ws && ws.readyState === WebSocket.OPEN) ? 'pointer' : 'not-allowed'
            }}
            disabled={!ws || ws.readyState !== WebSocket.OPEN}
          >
            送信
          </button>
        </form>
      </div>
    </main>
  );
}
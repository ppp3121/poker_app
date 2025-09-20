'use client';

import { useEffect, useState } from 'react';
import { useParams } from 'next/navigation';
import { useUserStore } from '@/store/userStore';
import { Room } from '@/types';
import { useRouter } from 'next/navigation';

export default function RoomPage() {
  const { isLoggedIn, isInitialized } = useUserStore();
  const router = useRouter();
  const params = useParams();
  const roomId = params.roomId as string;

  const [room, setRoom] = useState<Room | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

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
    <main style={{ padding: '2rem' }}>
      <h1>{room.name}</h1>
      <p>ルームID: {room.id}</p>
      <p>ステータス: {room.status}</p>
      <p>作成者ID: {room.created_by}</p>
      <p>作成日時: {new Date(room.created_at).toLocaleString()}</p>

      {/* ここに将来的にゲームコンポーネントが配置されます */}
      <div style={{ marginTop: '2rem', border: '1px solid #ccc', padding: '1rem' }}>
        <h2>ゲームテーブル</h2>
        <p>（ここにポーカーのテーブルが表示されます）</p>
      </div>
    </main>
  );
}
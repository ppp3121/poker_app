'use client';

import { useUserStore } from '@/store/userStore';
import { useRouter } from 'next/navigation';
import { useEffect, useState } from 'react';
import { Room } from '@/types';

export default function LobbyPage() {
  const { isLoggedIn, isInitialized } = useUserStore();
  const router = useRouter();

  const [rooms, setRooms] = useState<Room[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isInitialized && !isLoggedIn) {
      router.push('/login');
      return; // 未ログインなら以降の処理はしない
    }

    if (isInitialized && isLoggedIn) {
      const fetchRooms = async () => {
        try {
          const response = await fetch('http://localhost:8000/api/rooms', {
            method: 'GET',
            credentials: 'include', // 認証Cookieを忘れずに送信
          });

          if (!response.ok) {
            throw new Error('ルームの取得に失敗しました。');
          }

          const data: Room[] = await response.json();
          setRooms(data);

        } catch (err) {
          setError(err instanceof Error ? err.message : '不明なエラーです。');
        } finally {
          setIsLoading(false);
        }
      };

      fetchRooms();
    }
  }, [isInitialized, isLoggedIn, router]);

  useEffect(() => {
    if (isInitialized && !isLoggedIn) {
      router.push('/login');
    }
  }, [isInitialized, isLoggedIn, router]);

  return (
    isLoggedIn && (
      <main style={{ padding: '2rem', maxWidth: '800px', margin: 'auto' }}>
        <h1>ゲームロビー</h1>
        {error && <p style={{ color: 'red' }}>{error}</p>}

        <div style={{ marginTop: '2rem' }}>
          {rooms.length > 0 ? (
            <ul style={{ listStyle: 'none', padding: 0 }}>
              {rooms.map((room) => (
                <li key={room.id} style={{ border: '1px solid #555', padding: '1rem', marginBottom: '1rem', display: 'flex', justifyContent: 'space-between' }}>
                  <span>{room.name}</span>
                  <button style={{ padding: '0.5rem 1rem' }}>参加する</button>
                </li>
              ))}
            </ul>
          ) : (
            <p>現在参加可能なルームはありません。「ルーム作成」から新しいルームを作成してください。</p>
          )}
        </div>
      </main>
    )
  );
}
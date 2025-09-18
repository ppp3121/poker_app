'use client';

import { useUserStore } from '@/store/userStore';
import { useRouter } from 'next/navigation';
import { FormEvent, useEffect, useState } from 'react';

export default function CreateRoomPage() {
  const { isLoggedIn, isInitialized } = useUserStore();
  const router = useRouter();

  // フォームの状態を管理するためのState
  const [roomName, setRoomName] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // フォームが送信されたときの処理
  const handleCreateRoom = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault(); // ページの再読み込みを防ぐ
    setIsLoading(true);
    setError(null);

    if (!roomName.trim()) {
      setError('ルーム名を入力してください。');
      setIsLoading(false);
      return;
    }

    try {
      // バックエンドのルーム作成APIを呼び出す
      const response = await fetch('http://localhost:8000/api/rooms', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ name: roomName }),
        credentials: 'include', // 認証Cookieを送信するために必須
      });

      if (!response.ok) {
        const errorData = await response.text();
        throw new Error(`ルーム作成に失敗しました: ${errorData}`);
      }

      // ルーム作成に成功したら、ロビーページに移動
      router.push('/lobby');

    } catch (err: unknown) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError('予期しないエラーが発生しました。');
      }
    } finally {
      setIsLoading(false);
    }
  };

  // ログイン状態をチェックして、未ログインならリダイレクトする
  useEffect(() => {
    if (isInitialized && !isLoggedIn) {
      router.push('/login');
    }
  }, [isInitialized, isLoggedIn, router]);

  if (!isInitialized) {
    return <div>読み込み中...</div>;
  }

  // ログインしている場合のみフォームを表示
  return (
    isLoggedIn && (
      <main style={{ padding: '2rem', maxWidth: '500px', margin: 'auto' }}>
        <h1>ルーム作成</h1>
        <p style={{ marginBottom: '2rem' }}>
          新しいポーカーテーブルを作成します。
        </p>
        <form onSubmit={handleCreateRoom}>
          <div style={{ marginBottom: '1rem' }}>
            <label htmlFor="roomName" style={{ display: 'block', marginBottom: '0.5rem' }}>
              ルーム名
            </label>
            <input
              id="roomName"
              type="text"
              value={roomName}
              onChange={(e) => setRoomName(e.target.value)}
              required
              style={{ width: '100%', padding: '0.5rem', color: 'black' }}
              placeholder="例：週末わいわいポーカー"
            />
          </div>
          <button type="submit" disabled={isLoading} style={{ padding: '0.5rem 1rem', width: '100%' }}>
            {isLoading ? '作成中...' : 'ルームを作成する'}
          </button>
          {error && <p style={{ color: 'red', marginTop: '1rem' }}>{error}</p>}
        </form>
      </main>
    )
  );
}
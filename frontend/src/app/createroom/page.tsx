'use client';

import { useUserStore } from '@/store/userStore';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

export default function CreateRoomPage() {
  const { isLoggedIn, isInitialized } = useUserStore();
  const router = useRouter();

  useEffect(() => {
    // 初期化が完了していて、かつ未ログインの場合にリダイレクト
    if (isInitialized && !isLoggedIn) {
      router.push('/login');
    }
  }, [isInitialized, isLoggedIn, router]);

  if (!isInitialized) {
    return <div>読み込み中...</div>;
  }

  return (
    isLoggedIn && (
      <main style={{ padding: '2rem', textAlign: 'center' }}>
        <h1>ルーム作成</h1>
        <p>ここで新しいポーカーテーブルを作成します。</p>
        {/* 将来的にはここにルーム名入力フォームなどが入ります */}
      </main>
    )
  );
}
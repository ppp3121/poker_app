'use client';

import { useUserStore } from '@/store/userStore';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

export default function LobbyPage() {
  const { isLoggedIn, isInitialized } = useUserStore();
  const router = useRouter();

  useEffect(() => {
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
        <h1>ゲームロビー</h1>
        <p>参加可能なルームがここに表示されます。</p>
        {/* 将来的にはここにルーム一覧が表示されます */}
      </main>
    )
  );
}
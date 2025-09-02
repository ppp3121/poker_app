'use client';

import { useUserStore } from '@/store/userStore';
import { useRouter } from 'next/navigation'; // next/routerではなくnext/navigation
import { useEffect } from 'react';

export default function MyPage() {
  const { isLoggedIn, username, isInitialized } = useUserStore();
  const router = useRouter();

  useEffect(() => {
    // 初期化が完了していて、かつ未ログインの場合にリダイレクト
    if (isInitialized && !isLoggedIn) {
      router.push('/login');
    }
  }, [isInitialized, isLoggedIn, router]);

  // 初期化中はローディング表示などを出すのが親切
  if (!isInitialized) {
    return <div>読み込み中...</div>;
  }

  // ログインしている場合のみページ内容を表示
  return (
    isLoggedIn && (
      <main style={{ padding: '2rem', textAlign: 'center' }}>
        <h1>マイページ</h1>
        <p>ようこそ、{username}さん！</p>
        <p>このページはログインしているユーザーだけが見ることができます。</p>
      </main>
    )
  );
}
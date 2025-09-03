'use client';

import { useUserStore } from '@/store/userStore';
import Link from 'next/link';

export default function AuthStatus() {
  const { isLoggedIn, username, logout } = useUserStore();

  const handleLogout = async () => {
    try {
      // バックエンドにログアウトAPIをコール
      await fetch('http://localhost:8000/api/logout', {
        method: 'POST',
        credentials: 'include', // Cookie操作のために必要
      });
    } catch (error) {
      console.error('Logout failed', error);
    } finally {
      // APIの成否にかかわらず、フロントエンドの状態はクリアする
      logout();
      //ログアウト後はトップページに移動
      window.location.href = '/';
    }
  };

  if (isLoggedIn) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
        <Link href="/lobby">ロビー</Link>
        <Link href="/createroom">ルーム作成</Link>
        <Link href="/mypage">マイページ</Link>
        <p>ようこそ、{username}さん</p>
        <button onClick={handleLogout} style={{ padding: '0.5rem 1rem' }}>
          ログアウト
        </button>
      </div>
    );
  }


  return (
    <div style={{ display: 'flex', gap: '1rem' }}>
      <Link href="/login" style={{ padding: '0.5rem 1rem', border: '1px solid white' }}>
        ログイン
      </Link>
      <Link href="/register" style={{ padding: '0.5rem 1rem', border: '1px solid white' }}>
        ユーザー登録
      </Link>
    </div>
  );
}
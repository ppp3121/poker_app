'use client';

import { useUserStore } from '@/store/userStore';
import Link from 'next/link';

export default function AuthStatus() {
  const { isLoggedIn, username, logout } = useUserStore();

  if (isLoggedIn) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
        <p>ようこそ、{username}さん</p>
        <button onClick={logout} style={{ padding: '0.5rem 1rem' }}>
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
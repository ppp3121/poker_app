'use client';

import { useUserStore } from '@/store/userStore';
import { useEffect } from 'react';

export default function AuthInitializer() {
  // ★ setInitializedアクションを取得
  const { login, logout, setInitialized } = useUserStore();

  useEffect(() => {
    // このuseEffectはマウント時に一度だけ実行される
    const checkAuthStatus = async () => {
      try {
        const response = await fetch('http://localhost:8000/api/me', {
          credentials: 'include',
        });

        if (response.ok) {
          const data = await response.json();
          if (data.sub) {
            login(data.sub);
          }
        } else {
          logout();
        }
      } catch (error) {
        console.error('認証状態の確認に失敗しました:', error);
        logout();
      } finally {
        // ★ APIへの問い合わせが成功しても失敗しても、初期化は完了したとマークする
        setInitialized(true);
      }
    };

    checkAuthStatus();
    // ★ 依存配列から不要なものを削除し、初回マウント時のみ実行されるようにする
  }, [login, logout, setInitialized]);

  return null;
}
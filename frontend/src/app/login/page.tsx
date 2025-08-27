'use client';

import { useState, FormEvent } from 'react';

export default function LoginPage() {
  const [username, setUsername] = useState<string>('');
  const [password, setPassword] = useState<string>('');
  const [message, setMessage] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const handleLogin = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setIsLoading(true);
    setError(null);
    setMessage('');

    try {
      // RustサーバーのログインAPIエンドポイントを叩く
      const response = await fetch('http://localhost:8000/api/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
      });

      if (!response.ok) {
        const errorData = await response.text();
        throw new Error(
          `ログインに失敗しました。ステータス: ${response.status}, メッセージ: ${errorData}`
        );
      }

      // ★ 成功メッセージを変更
      setMessage('ログインに成功しました！');
      setUsername('');
      setPassword('');

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

  return (
    <main style={{ padding: '2rem', maxWidth: '400px', margin: 'auto' }}>
      <h1>ログイン</h1>
      <form onSubmit={handleLogin}>
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="username" style={{ display: 'block', marginBottom: '0.5rem' }}>
            ユーザー名
          </label>
          <input
            id="username"
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
            style={{ width: '100%', padding: '0.5rem', color: 'black' }}
          />
        </div>
        <div style={{ marginBottom: '1rem' }}>
          <label htmlFor="password" style={{ display: 'block', marginBottom: '0.5rem' }}>
            パスワード
          </label>
          <input
            id="password"
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            style={{ width: '100%', padding: '0.5rem', color: 'black' }}
          />
        </div>
        <button type="submit" disabled={isLoading} style={{ padding: '0.5rem 1rem' }}>
          {isLoading ? 'ログイン中...' : 'ログイン'}
        </button>
      </form>

      <div style={{ marginTop: '1rem', fontFamily: 'monospace' }}>
        {message && <p style={{ color: 'green' }}>{message}</p>}
        {error && <p style={{ color: 'red' }}>エラー: {error}</p>}
      </div>
    </main>
  );
}
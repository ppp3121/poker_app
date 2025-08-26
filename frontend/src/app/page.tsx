'use client'; // ブラウザ側で動くインタラクティブなコンポーネントであることを示す

import { useState, FormEvent } from 'react';

export default function Home() {
  // フォームの入力値を管理するための状態変数
  const [username, setUsername] = useState<string>('');
  const [password, setPassword] = useState<string>('');

  // サーバーからの応答、ローディング状態、エラーを管理するための状態変数
  const [message, setMessage] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  // フォームが送信されたときに実行される関数
  const handleRegister = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault(); // フォームのデフォルトの送信動作を防ぐ
    setIsLoading(true);
    setError(null);
    setMessage('');

    try {
      // Rustサーバーの登録APIエンドポイントを叩く
      const response = await fetch('http://localhost:8000/api/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ username, password }),
      });

      // レスポンスが成功でない場合は、エラーメッセージを組み立てて投げる
      if (!response.ok) {
        const errorData = await response.text(); // プレーンテキストでエラーメッセージを取得
        throw new Error(
          `登録に失敗しました。ステータス: ${response.status}, メッセージ: ${errorData}`
        );
      }

      // 成功メッセージをstateにセット
      setMessage('ユーザー登録が成功しました！');
      setUsername(''); // フォームをクリア
      setPassword(''); // フォームをクリア

    } catch (err: unknown) { // errをunknown型で受け取る
      if (err instanceof Error) {
        // errがErrorオブジェクトであることを確認してからプロパティにアクセス
        setError(err.message);
      } else {
        // 予期せぬエラー（文字列がthrowされた場合など）
        setError('予期しないエラーが発生しました。');
      }
    } finally {
      // ローディング状態を解除
      setIsLoading(false);
    }
  };

  return (
    <main style={{ padding: '2rem', maxWidth: '400px', margin: 'auto' }}>
      <h1>ユーザー登録</h1>
      <form onSubmit={handleRegister}>
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
          {isLoading ? '登録中...' : '登録'}
        </button>
      </form>

      {/* サーバーからの応答を表示するエリア */}
      <div style={{ marginTop: '1rem', fontFamily: 'monospace' }}>
        {message && <p style={{ color: 'green' }}>{message}</p>}
        {error && <p style={{ color: 'red' }}>エラー: {error}</p>}
      </div>
    </main>
  );
}
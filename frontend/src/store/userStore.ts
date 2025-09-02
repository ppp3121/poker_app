import { create } from 'zustand';

// ストアで管理する状態の型を定義
interface UserState {
  username: string | null;
  isLoggedIn: boolean;
  isInitialized: boolean; // ★ 認証チェックが完了したかどうかのフラグ
  login: (username: string) => void;
  logout: () => void;
  setInitialized: (isInitialized: boolean) => void; // ★ isInitializedを更新するアクション
}

// ストアを作成
export const useUserStore = create<UserState>((set) => ({
  // 初期状態
  username: null,
  isLoggedIn: false,
  isInitialized: false, // ★ 初期値はfalse

  // 状態を更新するためのアクション
  login: (username) => set({ username, isLoggedIn: true }),
  logout: () => set({ username: null, isLoggedIn: false }),
  setInitialized: (isInitialized) => set({ isInitialized }), // ★ アクションを定義
}));
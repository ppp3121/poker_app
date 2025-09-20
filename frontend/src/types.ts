export interface Room {
  id: string;
  name: string;
  status: 'waiting' | 'playing' | 'finished';
  created_by: string;
  created_at: string;
}
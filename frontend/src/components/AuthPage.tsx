import { useState } from 'react';
import { Wand2, Loader2 } from 'lucide-react';
import { useAuthStore } from '../hooks/useAuth';

export default function AuthPage() {
  const [mode, setMode] = useState<'login' | 'register'>('login');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const { login, register } = useAuthStore();

  const submit = async () => {
    setError('');
    setLoading(true);
    try {
      if (mode === 'login') await login(email, password);
      else await register(email, password);
    } catch (e: any) {
      setError(e?.response?.data?.error?.message ?? 'Something went wrong');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-violet-50 to-indigo-100 flex items-center justify-center p-4">
      <div className="w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center gap-2 mb-2">
            <Wand2 className="w-8 h-8 text-violet-600" />
            <span className="text-2xl font-bold text-gray-900">RewriteAI</span>
          </div>
          <p className="text-gray-500 text-sm">AI-powered text rewriting with structured insights</p>
        </div>

        <div className="bg-white rounded-2xl shadow-xl p-8 flex flex-col gap-5">
          <div className="flex bg-gray-100 rounded-xl p-1">
            {(['login', 'register'] as const).map((m) => (
              <button
                key={m}
                onClick={() => setMode(m)}
                className={`flex-1 py-2 rounded-lg text-sm font-medium transition-all capitalize ${
                  mode === m ? 'bg-white shadow text-gray-900' : 'text-gray-500'
                }`}
              >
                {m === 'login' ? 'Sign in' : 'Sign up'}
              </button>
            ))}
          </div>

          <input
            type="email"
            placeholder="Email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            className="border border-gray-200 rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-violet-500 w-full"
          />
          <input
            type="password"
            placeholder="Password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && submit()}
            className="border border-gray-200 rounded-xl px-4 py-3 text-sm outline-none focus:ring-2 focus:ring-violet-500 w-full"
          />

          {error && <p className="text-sm text-red-500">{error}</p>}

          <button
            onClick={submit}
            disabled={loading || !email || !password}
            className="bg-violet-600 hover:bg-violet-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-semibold py-3 rounded-xl transition-colors flex items-center justify-center gap-2"
          >
            {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
            {mode === 'login' ? 'Sign in' : 'Create account'}
          </button>

          {mode === 'register' && (
            <p className="text-xs text-center text-gray-400">
              Free plan: 10 rewrites/month. Upgrade anytime.
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

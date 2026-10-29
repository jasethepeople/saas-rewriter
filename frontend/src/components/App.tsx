import { useState } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import {
  Wand2, Zap, FileText, BarChart2, ChevronDown,
  CheckCircle, AlertTriangle, Loader2, LogOut, History
} from 'lucide-react';
import { rewriteApi, userApi, RewriteTone, RewriteStyle, TextAnalysis, RewriteAnalysis } from '../lib/api';
import { useAuthStore } from '../hooks/useAuth';
import clsx from 'clsx';

const TONES: { value: RewriteTone; label: string }[] = [
  { value: 'professional', label: 'Professional' },
  { value: 'casual', label: 'Casual' },
  { value: 'persuasive', label: 'Persuasive' },
  { value: 'concise', label: 'Concise' },
  { value: 'academic', label: 'Academic' },
  { value: 'empathetic', label: 'Empathetic' },
];

const STYLES: { value: RewriteStyle; label: string; desc: string }[] = [
  { value: 'improve', label: 'Improve', desc: 'Overall quality' },
  { value: 'simplify', label: 'Simplify', desc: 'Easier to read' },
  { value: 'expand', label: 'Expand', desc: 'Add more detail' },
  { value: 'condense', label: 'Condense', desc: 'Make shorter' },
  { value: 'fix', label: 'Fix', desc: 'Grammar & spelling' },
];

const SCORE_COLOR = (n: number) =>
  n >= 8 ? 'text-emerald-600' : n >= 5 ? 'text-amber-500' : 'text-red-500';

export default function App() {
  const { user, logout } = useAuthStore();
  const [text, setText] = useState('');
  const [tone, setTone] = useState<RewriteTone>('professional');
  const [style, setStyle] = useState<RewriteStyle>('improve');
  const [tab, setTab] = useState<'rewrite' | 'auto' | 'analyze'>('rewrite');
  const [result, setResult] = useState<{
    analysis?: TextAnalysis;
    rewrite?: RewriteAnalysis;
    original?: string;
  } | null>(null);

  const { data: usage } = useQuery({
    queryKey: ['usage'],
    queryFn: () => userApi.usage().then((r) => r.data),
    refetchInterval: 30_000,
  });

  const rewriteMutation = useMutation({
    mutationFn: () => rewriteApi.create(text, tone, style),
    onSuccess: async ({ data }) => {
      // Poll until job completes
      let job = data.job;
      while (job.status === 'processing' || job.status === 'pending') {
        await new Promise((r) => setTimeout(r, 1200));
        const resp = await rewriteApi.get(job.id);
        job = resp.data.job;
      }
      setResult({ rewrite: job.analysis, original: job.original_text });
    },
  });

  const autoMutation = useMutation({
    mutationFn: () => rewriteApi.auto(text),
    onSuccess: ({ data }) => {
      setResult({ analysis: data.analysis, rewrite: data.rewrite, original: text });
    },
  });

  const analyzeMutation = useMutation({
    mutationFn: () => rewriteApi.analyze(text),
    onSuccess: ({ data }) => {
      setResult({ analysis: data, original: text });
    },
  });

  const isLoading = rewriteMutation.isPending || autoMutation.isPending || analyzeMutation.isPending;

  const handleSubmit = () => {
    if (!text.trim()) return;
    setResult(null);
    if (tab === 'rewrite') rewriteMutation.mutate();
    else if (tab === 'auto') autoMutation.mutate();
    else analyzeMutation.mutate();
  };

  return (
    <div className="min-h-screen bg-gray-50 flex flex-col">
      {/* Header */}
      <header className="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Wand2 className="w-6 h-6 text-violet-600" />
          <span className="text-xl font-bold text-gray-900">RewriteAI</span>
          <span className="ml-2 text-xs bg-violet-100 text-violet-700 px-2 py-0.5 rounded-full font-medium capitalize">
            {user?.plan}
          </span>
        </div>

        <div className="flex items-center gap-4">
          {usage && (
            <div className="text-sm text-gray-500">
              <span className="font-medium text-gray-900">{usage.rewrites_remaining}</span> rewrites left
            </div>
          )}
          <button onClick={logout} className="flex items-center gap-1.5 text-sm text-gray-500 hover:text-gray-900 transition-colors">
            <LogOut className="w-4 h-4" />
            Sign out
          </button>
        </div>
      </header>

      <main className="flex-1 max-w-6xl mx-auto w-full px-6 py-8 grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Left panel – Input */}
        <div className="flex flex-col gap-5">
          {/* Tab selector */}
          <div className="flex bg-white border border-gray-200 rounded-xl p-1 gap-1">
            {[
              { id: 'rewrite', icon: FileText, label: 'Manual Rewrite' },
              { id: 'auto', icon: Zap, label: 'Auto Rewrite' },
              { id: 'analyze', icon: BarChart2, label: 'Analyze Only' },
            ].map(({ id, icon: Icon, label }) => (
              <button
                key={id}
                onClick={() => setTab(id as typeof tab)}
                className={clsx(
                  'flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg text-sm font-medium transition-all',
                  tab === id
                    ? 'bg-violet-600 text-white shadow-sm'
                    : 'text-gray-500 hover:text-gray-700 hover:bg-gray-50'
                )}
              >
                <Icon className="w-4 h-4" />
                {label}
              </button>
            ))}
          </div>

          {/* Text input */}
          <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-4 flex flex-col gap-3">
            <label className="text-sm font-medium text-gray-700">Your text</label>
            <textarea
              value={text}
              onChange={(e) => setText(e.target.value)}
              placeholder="Paste or type your text here…"
              rows={10}
              className="w-full resize-none text-sm text-gray-800 outline-none placeholder-gray-400"
            />
            <div className="flex items-center justify-between text-xs text-gray-400">
              <span>{text.length} / 10,000 chars</span>
              {text.length > 10_000 && <span className="text-red-500">Too long</span>}
            </div>
          </div>

          {/* Manual controls */}
          {tab === 'rewrite' && (
            <div className="grid grid-cols-2 gap-4">
              <div className="bg-white rounded-xl border border-gray-200 p-4">
                <p className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-3">Tone</p>
                <div className="flex flex-col gap-1">
                  {TONES.map((t) => (
                    <button
                      key={t.value}
                      onClick={() => setTone(t.value)}
                      className={clsx(
                        'text-left px-3 py-1.5 rounded-lg text-sm transition-all',
                        tone === t.value
                          ? 'bg-violet-50 text-violet-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-50'
                      )}
                    >
                      {t.label}
                    </button>
                  ))}
                </div>
              </div>
              <div className="bg-white rounded-xl border border-gray-200 p-4">
                <p className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-3">Style</p>
                <div className="flex flex-col gap-1">
                  {STYLES.map((s) => (
                    <button
                      key={s.value}
                      onClick={() => setStyle(s.value)}
                      className={clsx(
                        'text-left px-3 py-1.5 rounded-lg text-sm transition-all',
                        style === s.value
                          ? 'bg-violet-50 text-violet-700 font-medium'
                          : 'text-gray-600 hover:bg-gray-50'
                      )}
                    >
                      <span className="font-medium">{s.label}</span>
                      <span className="text-gray-400 ml-1">· {s.desc}</span>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}

          {tab === 'auto' && (
            <div className="bg-violet-50 border border-violet-200 rounded-xl px-4 py-3 text-sm text-violet-700">
              <Zap className="inline w-4 h-4 mr-1" />
              Auto Rewrite analyzes your text first, then automatically picks the best tone and style to improve it — no configuration needed.
            </div>
          )}

          <button
            onClick={handleSubmit}
            disabled={isLoading || !text.trim() || text.length > 10_000}
            className="w-full bg-violet-600 hover:bg-violet-700 disabled:opacity-50 disabled:cursor-not-allowed text-white font-semibold py-3 rounded-xl transition-colors flex items-center justify-center gap-2"
          >
            {isLoading ? (
              <><Loader2 className="w-4 h-4 animate-spin" /> Processing…</>
            ) : (
              <><Wand2 className="w-4 h-4" />
                {tab === 'rewrite' ? 'Rewrite' : tab === 'auto' ? 'Auto Rewrite' : 'Analyze'}</>
            )}
          </button>
        </div>

        {/* Right panel – Results */}
        <div className="flex flex-col gap-5">
          {!result && !isLoading && (
            <div className="flex-1 bg-white rounded-xl border border-dashed border-gray-200 flex items-center justify-center text-gray-400 py-24">
              <div className="text-center">
                <Wand2 className="w-10 h-10 mx-auto mb-3 opacity-30" />
                <p className="text-sm">Results will appear here</p>
              </div>
            </div>
          )}

          {isLoading && (
            <div className="flex-1 bg-white rounded-xl border border-gray-200 flex items-center justify-center py-24">
              <div className="text-center">
                <Loader2 className="w-8 h-8 mx-auto mb-3 animate-spin text-violet-500" />
                <p className="text-sm text-gray-500">Talking to OpenAI…</p>
              </div>
            </div>
          )}

          {result?.analysis && <AnalysisCard analysis={result.analysis} />}
          {result?.rewrite && <RewriteCard rewrite={result.rewrite} original={result.original ?? ''} />}
        </div>
      </main>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

function ScoreBadge({ label, value }: { label: string; value: number }) {
  return (
    <div className="flex flex-col items-center bg-gray-50 rounded-lg px-4 py-3">
      <span className={clsx('text-2xl font-bold', SCORE_COLOR(value))}>{value}</span>
      <span className="text-xs text-gray-500 mt-0.5">{label}</span>
    </div>
  );
}

function AnalysisCard({ analysis }: { analysis: TextAnalysis }) {
  return (
    <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-5 flex flex-col gap-4">
      <h2 className="font-semibold text-gray-900 flex items-center gap-2">
        <BarChart2 className="w-4 h-4 text-violet-500" /> Text Analysis
      </h2>

      <div className="grid grid-cols-3 gap-3">
        <ScoreBadge label="Quality" value={analysis.overall_quality} />
        <div className="flex flex-col items-center bg-gray-50 rounded-lg px-4 py-3">
          <span className="text-sm font-semibold text-gray-700 text-center leading-tight">{analysis.readability_level}</span>
          <span className="text-xs text-gray-500 mt-0.5">Reading level</span>
        </div>
        <div className="flex flex-col items-center bg-gray-50 rounded-lg px-4 py-3">
          <span className="text-sm font-semibold text-gray-700 capitalize">{analysis.tone_detected}</span>
          <span className="text-xs text-gray-500 mt-0.5">Tone</span>
        </div>
      </div>

      {analysis.issues.length > 0 && (
        <div>
          <p className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">Issues</p>
          <ul className="flex flex-col gap-2">
            {analysis.issues.map((issue, i) => (
              <li key={i} className="flex gap-2 text-sm">
                <AlertTriangle className={clsx('w-4 h-4 flex-shrink-0 mt-0.5',
                  issue.severity === 'high' ? 'text-red-500' :
                  issue.severity === 'medium' ? 'text-amber-500' : 'text-gray-400'
                )} />
                <span className="text-gray-700">{issue.description}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {analysis.strengths.length > 0 && (
        <div>
          <p className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">Strengths</p>
          <ul className="flex flex-col gap-1.5">
            {analysis.strengths.map((s, i) => (
              <li key={i} className="flex gap-2 text-sm text-gray-700">
                <CheckCircle className="w-4 h-4 flex-shrink-0 mt-0.5 text-emerald-500" />
                {s}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function RewriteCard({ rewrite, original }: { rewrite: RewriteAnalysis; original: string }) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(rewrite.rewritten_text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="bg-white rounded-xl border border-gray-200 shadow-sm p-5 flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h2 className="font-semibold text-gray-900 flex items-center gap-2">
          <Wand2 className="w-4 h-4 text-violet-500" /> Rewritten Text
        </h2>
        <button onClick={copy} className="text-xs text-violet-600 hover:text-violet-800 font-medium transition-colors">
          {copied ? '✓ Copied' : 'Copy'}
        </button>
      </div>

      <div className="bg-gray-50 rounded-lg p-4 text-sm text-gray-800 leading-relaxed whitespace-pre-wrap">
        {rewrite.rewritten_text}
      </div>

      <div className="grid grid-cols-2 gap-3">
        <ScoreBadge label="Readability" value={rewrite.readability_score} />
        <ScoreBadge label="Tone match" value={rewrite.tone_match_score} />
      </div>

      <p className="text-sm text-gray-600 italic">{rewrite.changes_summary}</p>

      <div className="flex gap-4 text-xs text-gray-400">
        <span>{rewrite.word_count_original} words → <strong className="text-gray-600">{rewrite.word_count_rewritten} words</strong></span>
      </div>

      {rewrite.improvements.length > 0 && (
        <div>
          <p className="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">What changed</p>
          <ul className="flex flex-col gap-1.5">
            {rewrite.improvements.map((imp, i) => (
              <li key={i} className="flex gap-2 text-sm text-gray-700">
                <span className="inline-block w-16 flex-shrink-0 text-xs font-medium text-violet-600 capitalize">{imp.category}</span>
                {imp.description}
              </li>
            ))}
          </ul>
        </div>
      )}

      {rewrite.warnings.length > 0 && (
        <div className="bg-amber-50 rounded-lg p-3">
          <p className="text-xs font-semibold text-amber-700 mb-1">Warnings</p>
          {rewrite.warnings.map((w, i) => <p key={i} className="text-xs text-amber-700">{w}</p>)}
        </div>
      )}
    </div>
  );
}

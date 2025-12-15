'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { api } from '@/lib/api';
import { formatDate, formatDuration } from '@/lib/utils';
import Link from 'next/link';
import {
  Search,
  Filter,
  ChevronLeft,
  ChevronRight,
  Mic,
  Clock,
  Target,
  Building2,
  Play,
} from 'lucide-react';

const MODES = ['sales', 'interview', 'negotiation', 'technical', 'discovery'];
const OUTCOMES = ['achieved', 'partial', 'not_achieved', 'pending'];

export default function RecordingsPage() {
  const router = useRouter();
  const { isAuthenticated, isLoading: authLoading, checkAuth } = useAuth();

  const [page, setPage] = useState(1);
  const [modeFilter, setModeFilter] = useState<string>('');
  const [outcomeFilter, setOutcomeFilter] = useState<string>('');

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth/login');
    }
  }, [isAuthenticated, authLoading, router]);

  const { data, isLoading } = useQuery({
    queryKey: ['recordings', page, modeFilter, outcomeFilter],
    queryFn: () => api.getRecordings({
      page,
      mode: modeFilter || undefined,
      outcome: outcomeFilter || undefined
    }),
    enabled: isAuthenticated,
  });

  if (authLoading || !isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16 items-center">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard" className="flex items-center space-x-2">
                <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
                  <span className="text-primary-foreground font-bold">H</span>
                </div>
                <span className="text-xl font-bold">Hekax</span>
              </Link>
            </div>
            <nav className="flex items-center space-x-6">
              <Link href="/dashboard" className="text-muted-foreground hover:text-foreground">
                Dashboard
              </Link>
              <Link href="/leads" className="text-muted-foreground hover:text-foreground">
                Leads
              </Link>
              <Link href="/recordings" className="text-foreground font-medium">
                Recordings
              </Link>
            </nav>
            <div className="w-32"></div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Page Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold">Recordings</h1>
          <p className="text-muted-foreground mt-1">
            Review and analyze your call recordings
          </p>
        </div>

        {/* Filters */}
        <div className="flex flex-wrap gap-4 mb-6">
          <div className="flex items-center space-x-2">
            <Filter className="w-4 h-4 text-muted-foreground" />
            <select
              value={modeFilter}
              onChange={(e) => {
                setModeFilter(e.target.value);
                setPage(1);
              }}
              className="px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
            >
              <option value="">All Modes</option>
              {MODES.map((mode) => (
                <option key={mode} value={mode}>
                  {mode.charAt(0).toUpperCase() + mode.slice(1)}
                </option>
              ))}
            </select>
          </div>
          <select
            value={outcomeFilter}
            onChange={(e) => {
              setOutcomeFilter(e.target.value);
              setPage(1);
            }}
            className="px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
          >
            <option value="">All Outcomes</option>
            {OUTCOMES.map((outcome) => (
              <option key={outcome} value={outcome}>
                {outcome.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')}
              </option>
            ))}
          </select>
        </div>

        {/* Recordings Grid */}
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
          </div>
        ) : data?.recordings && data.recordings.length > 0 ? (
          <>
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
              {data.recordings.map((recording) => (
                <Link
                  key={recording.id}
                  href={`/recordings/${recording.id}`}
                  className="block bg-card rounded-xl border border-border p-6 hover:border-primary/50 transition-colors group"
                >
                  <div className="flex items-start justify-between mb-4">
                    <div className="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
                      <Mic className="w-6 h-6 text-primary" />
                    </div>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                      recording.outcome === 'achieved'
                        ? 'bg-green-100 text-green-800'
                        : recording.outcome === 'partial'
                        ? 'bg-yellow-100 text-yellow-800'
                        : recording.outcome === 'not_achieved'
                        ? 'bg-red-100 text-red-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}>
                      {recording.outcome
                        ? recording.outcome.split('_').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ')
                        : recording.status}
                    </span>
                  </div>

                  <h3 className="font-semibold text-lg capitalize mb-2">
                    {recording.mode} Call
                  </h3>

                  <div className="space-y-2 text-sm text-muted-foreground">
                    <div className="flex items-center space-x-2">
                      <Clock className="w-4 h-4" />
                      <span>{formatDate(recording.start_time)}</span>
                    </div>
                    {recording.duration_seconds && (
                      <div className="flex items-center space-x-2">
                        <Play className="w-4 h-4" />
                        <span>{formatDuration(recording.duration_seconds)}</span>
                      </div>
                    )}
                    {recording.lead_id && (
                      <div className="flex items-center space-x-2">
                        <Building2 className="w-4 h-4" />
                        <span>Linked to lead</span>
                      </div>
                    )}
                  </div>

                  {recording.summary && (
                    <p className="mt-4 text-sm text-muted-foreground line-clamp-2">
                      {recording.summary}
                    </p>
                  )}

                  {recording.sentiment_score !== undefined && recording.sentiment_score !== null && (
                    <div className="mt-4 pt-4 border-t border-border">
                      <div className="flex items-center justify-between text-sm">
                        <span className="text-muted-foreground">Sentiment</span>
                        <div className="flex items-center space-x-2">
                          <div className="w-16 h-2 rounded-full bg-muted overflow-hidden">
                            <div
                              className={`h-full rounded-full ${
                                recording.sentiment_score > 0.6
                                  ? 'bg-green-500'
                                  : recording.sentiment_score > 0.4
                                  ? 'bg-yellow-500'
                                  : 'bg-red-500'
                              }`}
                              style={{ width: `${recording.sentiment_score * 100}%` }}
                            />
                          </div>
                          <span className="font-medium">{Math.round(recording.sentiment_score * 100)}%</span>
                        </div>
                      </div>
                    </div>
                  )}
                </Link>
              ))}
            </div>

            {/* Pagination */}
            {data.total_pages > 1 && (
              <div className="flex items-center justify-between">
                <p className="text-sm text-muted-foreground">
                  Showing {((page - 1) * 20) + 1} to {Math.min(page * 20, data.total)} of {data.total} recordings
                </p>
                <div className="flex items-center space-x-2">
                  <button
                    onClick={() => setPage(p => Math.max(1, p - 1))}
                    disabled={page === 1}
                    className="p-2 rounded-lg border border-border hover:bg-muted disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <ChevronLeft className="w-4 h-4" />
                  </button>
                  <span className="text-sm">
                    Page {page} of {data.total_pages}
                  </span>
                  <button
                    onClick={() => setPage(p => Math.min(data.total_pages, p + 1))}
                    disabled={page === data.total_pages}
                    className="p-2 rounded-lg border border-border hover:bg-muted disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <ChevronRight className="w-4 h-4" />
                  </button>
                </div>
              </div>
            )}
          </>
        ) : (
          <div className="bg-card rounded-xl border border-border p-12 text-center">
            <Mic className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
            <h3 className="text-lg font-medium mb-2">No recordings yet</h3>
            <p className="text-muted-foreground">
              Start using Voice Copilot to record and analyze your calls.
            </p>
          </div>
        )}
      </main>
    </div>
  );
}

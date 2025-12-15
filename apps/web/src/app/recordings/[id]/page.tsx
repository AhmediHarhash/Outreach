'use client';

import { useEffect } from 'react';
import { useRouter, useParams } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { api } from '@/lib/api';
import { formatDate, formatDuration } from '@/lib/utils';
import Link from 'next/link';
import {
  ArrowLeft,
  Mic,
  Clock,
  Target,
  Building2,
  User,
  MessageSquare,
  CheckCircle2,
  AlertCircle,
  TrendingUp,
  BarChart3,
} from 'lucide-react';

export default function RecordingDetailPage() {
  const router = useRouter();
  const params = useParams();
  const { isAuthenticated, isLoading: authLoading, checkAuth } = useAuth();

  const recordingId = params.id as string;

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth/login');
    }
  }, [isAuthenticated, authLoading, router]);

  const { data: recording, isLoading } = useQuery({
    queryKey: ['recording', recordingId],
    queryFn: () => api.getRecording(recordingId),
    enabled: isAuthenticated && !!recordingId,
  });

  if (authLoading || !isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="min-h-screen bg-background">
        <header className="border-b border-border bg-card">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex h-16 items-center">
              <Link href="/recordings" className="flex items-center text-muted-foreground hover:text-foreground">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Recordings
              </Link>
            </div>
          </div>
        </header>
        <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="flex items-center justify-center py-12">
            <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
          </div>
        </main>
      </div>
    );
  }

  if (!recording) {
    return (
      <div className="min-h-screen bg-background">
        <header className="border-b border-border bg-card">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex h-16 items-center">
              <Link href="/recordings" className="flex items-center text-muted-foreground hover:text-foreground">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Recordings
              </Link>
            </div>
          </div>
        </header>
        <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="text-center py-12">
            <h2 className="text-xl font-bold mb-2">Recording not found</h2>
            <p className="text-muted-foreground">This recording may have been deleted.</p>
          </div>
        </main>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex h-16 items-center">
            <Link href="/recordings" className="flex items-center text-muted-foreground hover:text-foreground">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Recordings
            </Link>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid lg:grid-cols-3 gap-8">
          {/* Main Content */}
          <div className="lg:col-span-2 space-y-6">
            {/* Recording Header */}
            <div className="bg-card rounded-xl border border-border p-6">
              <div className="flex items-start justify-between mb-6">
                <div className="flex items-center space-x-4">
                  <div className="w-16 h-16 rounded-xl bg-primary/10 flex items-center justify-center">
                    <Mic className="w-8 h-8 text-primary" />
                  </div>
                  <div>
                    <h1 className="text-2xl font-bold capitalize">{recording.mode} Call</h1>
                    <p className="text-muted-foreground">{formatDate(recording.start_time)}</p>
                  </div>
                </div>
                <span className={`px-3 py-1 rounded-full text-sm font-medium ${
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

              <div className="grid sm:grid-cols-3 gap-4">
                <div className="flex items-center space-x-3">
                  <div className="p-2 rounded-lg bg-muted">
                    <Clock className="w-4 h-4 text-muted-foreground" />
                  </div>
                  <div>
                    <p className="text-sm text-muted-foreground">Duration</p>
                    <p className="font-medium">
                      {recording.duration_seconds
                        ? formatDuration(recording.duration_seconds)
                        : 'In progress'}
                    </p>
                  </div>
                </div>
                {recording.talk_ratio !== undefined && recording.talk_ratio !== null && (
                  <div className="flex items-center space-x-3">
                    <div className="p-2 rounded-lg bg-muted">
                      <BarChart3 className="w-4 h-4 text-muted-foreground" />
                    </div>
                    <div>
                      <p className="text-sm text-muted-foreground">Talk Ratio</p>
                      <p className="font-medium">{Math.round(recording.talk_ratio * 100)}%</p>
                    </div>
                  </div>
                )}
                {recording.sentiment_score !== undefined && recording.sentiment_score !== null && (
                  <div className="flex items-center space-x-3">
                    <div className="p-2 rounded-lg bg-muted">
                      <TrendingUp className="w-4 h-4 text-muted-foreground" />
                    </div>
                    <div>
                      <p className="text-sm text-muted-foreground">Sentiment</p>
                      <p className={`font-medium ${
                        recording.sentiment_score > 0.6 ? 'text-green-600' :
                        recording.sentiment_score > 0.4 ? 'text-yellow-600' : 'text-red-600'
                      }`}>
                        {Math.round(recording.sentiment_score * 100)}%
                      </p>
                    </div>
                  </div>
                )}
              </div>
            </div>

            {/* Summary */}
            {recording.summary && (
              <div className="bg-card rounded-xl border border-border p-6">
                <h2 className="text-lg font-semibold mb-4 flex items-center">
                  <Target className="w-5 h-5 mr-2" />
                  Summary
                </h2>
                <p className="text-muted-foreground">{recording.summary}</p>
              </div>
            )}

            {/* Transcript */}
            {recording.transcript_turns && recording.transcript_turns.length > 0 && (
              <div className="bg-card rounded-xl border border-border p-6">
                <h2 className="text-lg font-semibold mb-4 flex items-center">
                  <MessageSquare className="w-5 h-5 mr-2" />
                  Transcript
                </h2>
                <div className="space-y-4 max-h-[600px] overflow-y-auto">
                  {recording.transcript_turns.map((turn: any, index: number) => (
                    <div
                      key={index}
                      className={`flex ${turn.speaker === 'user' ? 'justify-end' : 'justify-start'}`}
                    >
                      <div className={`max-w-[80%] ${
                        turn.speaker === 'user'
                          ? 'bg-primary text-primary-foreground rounded-tl-xl rounded-tr-xl rounded-bl-xl'
                          : 'bg-muted rounded-tl-xl rounded-tr-xl rounded-br-xl'
                      } p-4`}>
                        <div className="flex items-center space-x-2 mb-1">
                          <span className="text-xs font-medium opacity-70 capitalize">
                            {turn.speaker}
                          </span>
                          {turn.timestamp && (
                            <span className="text-xs opacity-50">
                              {formatDuration(turn.timestamp)}
                            </span>
                          )}
                        </div>
                        <p className="text-sm">{turn.text}</p>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Key Points */}
            {recording.key_points && recording.key_points.length > 0 && (
              <div className="bg-card rounded-xl border border-border p-6">
                <h2 className="text-lg font-semibold mb-4 flex items-center">
                  <CheckCircle2 className="w-5 h-5 mr-2 text-green-500" />
                  Key Points
                </h2>
                <ul className="space-y-3">
                  {recording.key_points.map((point: string, index: number) => (
                    <li key={index} className="flex items-start space-x-2">
                      <div className="w-1.5 h-1.5 rounded-full bg-primary mt-2 flex-shrink-0" />
                      <span className="text-sm text-muted-foreground">{point}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {/* Action Items */}
            {recording.action_items && recording.action_items.length > 0 && (
              <div className="bg-card rounded-xl border border-border p-6">
                <h2 className="text-lg font-semibold mb-4 flex items-center">
                  <AlertCircle className="w-5 h-5 mr-2 text-yellow-500" />
                  Action Items
                </h2>
                <ul className="space-y-3">
                  {recording.action_items.map((item: string, index: number) => (
                    <li key={index} className="flex items-start space-x-2">
                      <input
                        type="checkbox"
                        className="mt-1 rounded border-input"
                        disabled
                      />
                      <span className="text-sm text-muted-foreground">{item}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}

            {/* Linked Lead */}
            {recording.lead_id && (
              <div className="bg-card rounded-xl border border-border p-6">
                <h2 className="text-lg font-semibold mb-4 flex items-center">
                  <Building2 className="w-5 h-5 mr-2" />
                  Linked Lead
                </h2>
                <Link
                  href={`/leads/${recording.lead_id}`}
                  className="block p-4 rounded-lg border border-border hover:border-primary/50 transition-colors"
                >
                  <div className="flex items-center space-x-3">
                    <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
                      <Building2 className="w-5 h-5 text-primary" />
                    </div>
                    <div>
                      <p className="font-medium">View Lead Details</p>
                      <p className="text-sm text-muted-foreground">Click to open lead</p>
                    </div>
                  </div>
                </Link>
              </div>
            )}

            {/* Metadata */}
            <div className="bg-card rounded-xl border border-border p-6">
              <h2 className="text-lg font-semibold mb-4">Details</h2>
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Status</span>
                  <span className="font-medium capitalize">{recording.status}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Mode</span>
                  <span className="font-medium capitalize">{recording.mode}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Started</span>
                  <span className="font-medium">{formatDate(recording.start_time)}</span>
                </div>
                {recording.end_time && (
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Ended</span>
                    <span className="font-medium">{formatDate(recording.end_time)}</span>
                  </div>
                )}
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Recording ID</span>
                  <span className="font-mono text-xs">{recording.id.slice(0, 8)}...</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

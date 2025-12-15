'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useQuery } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { api } from '@/lib/api';
import { formatDate, getStatusColor, formatDuration } from '@/lib/utils';
import Link from 'next/link';
import {
  Users,
  Phone,
  TrendingUp,
  Clock,
  ChevronRight,
  Plus,
  Mail,
  Eye,
  MousePointer,
  Sparkles,
} from 'lucide-react';

export default function DashboardPage() {
  const router = useRouter();
  const { user, isAuthenticated, isLoading: authLoading, checkAuth, logout } = useAuth();

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth/login');
    }
  }, [isAuthenticated, authLoading, router]);

  const { data: leads } = useQuery({
    queryKey: ['leads'],
    queryFn: () => api.getLeads({ perPage: 5 }),
    enabled: isAuthenticated,
  });

  const { data: recordings } = useQuery({
    queryKey: ['recordings'],
    queryFn: () => api.getRecordings({ page: 1 }),
    enabled: isAuthenticated,
  });

  const { data: emailStats } = useQuery({
    queryKey: ['emailStats'],
    queryFn: () => (api as any).getEmailStats(),
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
              <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
                <span className="text-primary-foreground font-bold">H</span>
              </div>
              <span className="text-xl font-bold">Hekax</span>
            </div>
            <nav className="flex items-center space-x-6">
              <Link href="/dashboard" className="text-foreground font-medium">
                Dashboard
              </Link>
              <Link href="/leads" className="text-muted-foreground hover:text-foreground">
                Leads
              </Link>
              <Link href="/emails" className="text-muted-foreground hover:text-foreground">
                Emails
              </Link>
              <Link href="/recordings" className="text-muted-foreground hover:text-foreground">
                Recordings
              </Link>
            </nav>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-muted-foreground">{user?.email}</span>
              <button
                onClick={() => logout()}
                className="text-sm text-muted-foreground hover:text-foreground"
              >
                Logout
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Welcome */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold">
            Welcome back, {user?.full_name?.split(' ')[0] || 'there'}!
          </h1>
          <p className="text-muted-foreground mt-1">
            Here's what's happening with your pipeline today.
          </p>
        </div>

        {/* Stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <StatCard
            icon={<Users className="w-5 h-5" />}
            label="Total Leads"
            value={leads?.total || 0}
          />
          <StatCard
            icon={<Mail className="w-5 h-5" />}
            label="Emails Sent"
            value={emailStats?.total || 0}
            subValue={emailStats?.openRate ? `${emailStats.openRate}% opened` : undefined}
          />
          <StatCard
            icon={<Eye className="w-5 h-5" />}
            label="Emails Opened"
            value={emailStats?.opened || 0}
            highlight
          />
          <StatCard
            icon={<TrendingUp className="w-5 h-5" />}
            label="Reply Rate"
            value={emailStats?.replied || 0}
            subValue={emailStats?.replyRate ? `${emailStats.replyRate}%` : undefined}
          />
        </div>

        {/* Quick Actions */}
        <div className="flex space-x-4 mb-8">
          <Link
            href="/leads?new=true"
            className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
          >
            <Plus className="w-4 h-4" />
            <span>Add Lead</span>
          </Link>
          <Link
            href="/emails"
            className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-gradient-to-r from-indigo-500 to-purple-600 text-white hover:from-indigo-600 hover:to-purple-700 transition-all"
          >
            <Sparkles className="w-4 h-4" />
            <span>Compose Email</span>
          </Link>
        </div>

        {/* Recent Activity */}
        <div className="grid md:grid-cols-2 gap-8">
          {/* Recent Leads */}
          <div className="bg-card rounded-xl border border-border p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">Recent Leads</h2>
              <Link
                href="/leads"
                className="text-sm text-primary hover:underline flex items-center"
              >
                View all <ChevronRight className="w-4 h-4 ml-1" />
              </Link>
            </div>
            <div className="space-y-4">
              {leads?.leads.slice(0, 5).map((lead) => (
                <Link
                  key={lead.id}
                  href={`/leads/${lead.id}`}
                  className="block p-4 rounded-lg border border-border hover:border-primary/50 transition-colors"
                >
                  <div className="flex justify-between items-start">
                    <div>
                      <h3 className="font-medium">{lead.company_name}</h3>
                      <p className="text-sm text-muted-foreground">
                        {lead.contact_name || 'No contact'}
                      </p>
                    </div>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(lead.status)}`}>
                      {lead.status}
                    </span>
                  </div>
                </Link>
              ))}
              {(!leads?.leads || leads.leads.length === 0) && (
                <p className="text-muted-foreground text-center py-8">
                  No leads yet. Add your first lead!
                </p>
              )}
            </div>
          </div>

          {/* Recent Recordings */}
          <div className="bg-card rounded-xl border border-border p-6">
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">Recent Calls</h2>
              <Link
                href="/recordings"
                className="text-sm text-primary hover:underline flex items-center"
              >
                View all <ChevronRight className="w-4 h-4 ml-1" />
              </Link>
            </div>
            <div className="space-y-4">
              {recordings?.recordings.slice(0, 5).map((recording) => (
                <Link
                  key={recording.id}
                  href={`/recordings/${recording.id}`}
                  className="block p-4 rounded-lg border border-border hover:border-primary/50 transition-colors"
                >
                  <div className="flex justify-between items-start">
                    <div>
                      <h3 className="font-medium capitalize">{recording.mode} Call</h3>
                      <p className="text-sm text-muted-foreground">
                        {formatDate(recording.start_time)}
                        {recording.duration_seconds && (
                          <> &middot; {formatDuration(recording.duration_seconds)}</>
                        )}
                      </p>
                    </div>
                    <span className={`px-2 py-1 rounded-full text-xs font-medium ${
                      recording.outcome === 'achieved'
                        ? 'bg-green-100 text-green-800'
                        : recording.outcome === 'partial'
                        ? 'bg-yellow-100 text-yellow-800'
                        : 'bg-gray-100 text-gray-800'
                    }`}>
                      {recording.outcome || recording.status}
                    </span>
                  </div>
                </Link>
              ))}
              {(!recordings?.recordings || recordings.recordings.length === 0) && (
                <p className="text-muted-foreground text-center py-8">
                  No recordings yet. Start a call with Voice Copilot!
                </p>
              )}
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

function StatCard({
  icon,
  label,
  value,
  subValue,
  highlight,
}: {
  icon: React.ReactNode;
  label: string;
  value: number;
  subValue?: string;
  highlight?: boolean;
}) {
  return (
    <div className={`bg-card rounded-xl border p-6 transition-all ${
      highlight ? 'border-emerald-500/30 bg-emerald-500/5' : 'border-border'
    }`}>
      <div className="flex items-center space-x-3">
        <div className={`p-2 rounded-lg ${
          highlight ? 'bg-emerald-500/20 text-emerald-400' : 'bg-primary/10 text-primary'
        }`}>
          {icon}
        </div>
        <div>
          <p className="text-2xl font-bold">{value}</p>
          <p className="text-sm text-muted-foreground">{label}</p>
          {subValue && (
            <p className={`text-xs ${highlight ? 'text-emerald-400' : 'text-primary'}`}>
              {subValue}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

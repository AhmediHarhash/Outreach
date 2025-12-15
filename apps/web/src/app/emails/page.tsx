'use client';

import { useEffect, useState, Suspense } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { api, Email, EmailStats } from '@/lib/api';
import { formatDate } from '@/lib/utils';
import Link from 'next/link';
import {
  Mail,
  Send,
  Eye,
  MousePointer,
  MessageSquare,
  AlertTriangle,
  TrendingUp,
  TrendingDown,
  Sparkles,
  Plus,
  Filter,
  Search,
  ChevronRight,
  Clock,
  CheckCircle,
  XCircle,
  Zap,
  Target,
  Award,
  Flame,
} from 'lucide-react';
import { toast } from 'sonner';
import ComposeEmailModal from './compose-modal';

// Psychology-driven color mapping for status
const statusConfig: Record<string, { color: string; bg: string; icon: React.ReactNode; label: string }> = {
  sent: { color: 'text-blue-400', bg: 'bg-blue-500/10', icon: <Send className="w-3 h-3" />, label: 'Sent' },
  delivered: { color: 'text-cyan-400', bg: 'bg-cyan-500/10', icon: <CheckCircle className="w-3 h-3" />, label: 'Delivered' },
  opened: { color: 'text-emerald-400', bg: 'bg-emerald-500/10', icon: <Eye className="w-3 h-3" />, label: 'Opened' },
  clicked: { color: 'text-amber-400', bg: 'bg-amber-500/10', icon: <MousePointer className="w-3 h-3" />, label: 'Clicked' },
  replied: { color: 'text-green-400', bg: 'bg-green-500/10', icon: <MessageSquare className="w-3 h-3" />, label: 'Replied' },
  bounced: { color: 'text-red-400', bg: 'bg-red-500/10', icon: <XCircle className="w-3 h-3" />, label: 'Bounced' },
};

function EmailsPageContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const queryClient = useQueryClient();
  const { user, isAuthenticated, isLoading: authLoading, checkAuth } = useAuth();

  const [showCompose, setShowCompose] = useState(false);
  const [selectedStatus, setSelectedStatus] = useState<string | undefined>(
    searchParams.get('status') || undefined
  );
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth/login');
    }
  }, [isAuthenticated, authLoading, router]);

  // Fetch emails
  const { data: emailsData, isLoading: emailsLoading } = useQuery({
    queryKey: ['emails', selectedStatus],
    queryFn: () => (api as any).getEmails({ status: selectedStatus, perPage: 50 }),
    enabled: isAuthenticated,
    refetchInterval: 30000, // Variable reward: refresh every 30s for new data
  });

  // Fetch stats - the addiction trigger
  const { data: stats } = useQuery({
    queryKey: ['emailStats'],
    queryFn: () => (api as any).getEmailStats(),
    enabled: isAuthenticated,
    refetchInterval: 15000, // More frequent for dopamine hits
  });

  // Fetch leads for compose modal
  const { data: leads } = useQuery({
    queryKey: ['leads'],
    queryFn: () => api.getLeads({ perPage: 100 }),
    enabled: isAuthenticated,
  });

  if (authLoading || !isAuthenticated) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-indigo-500"></div>
      </div>
    );
  }

  const filteredEmails = emailsData?.emails?.filter((email: Email) =>
    !searchQuery ||
    email.subject.toLowerCase().includes(searchQuery.toLowerCase()) ||
    email.toEmail.toLowerCase().includes(searchQuery.toLowerCase()) ||
    email.contactName?.toLowerCase().includes(searchQuery.toLowerCase())
  ) || [];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* Header */}
      <header className="border-b border-white/5 bg-black/20 backdrop-blur-xl sticky top-0 z-40">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16 items-center">
            <div className="flex items-center space-x-4">
              <Link href="/dashboard" className="flex items-center space-x-3">
                <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/25">
                  <span className="text-white font-bold text-lg">O</span>
                </div>
                <span className="text-xl font-bold bg-gradient-to-r from-white to-white/60 bg-clip-text text-transparent">
                  Outreach
                </span>
              </Link>
            </div>
            <nav className="flex items-center space-x-1">
              {[
                { href: '/dashboard', label: 'Dashboard' },
                { href: '/leads', label: 'Leads' },
                { href: '/emails', label: 'Emails', active: true },
                { href: '/recordings', label: 'Recordings' },
              ].map((item) => (
                <Link
                  key={item.href}
                  href={item.href}
                  className={`px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                    item.active
                      ? 'bg-white/10 text-white'
                      : 'text-white/60 hover:text-white hover:bg-white/5'
                  }`}
                >
                  {item.label}
                </Link>
              ))}
            </nav>
            <div className="flex items-center space-x-4">
              <span className="text-sm text-white/40">{user?.email}</span>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {/* Hero Stats - Dopamine Dashboard */}
        <div className="mb-8">
          <div className="flex items-center justify-between mb-6">
            <div>
              <h1 className="text-3xl font-bold text-white flex items-center gap-3">
                <Mail className="w-8 h-8 text-indigo-400" />
                Email Command Center
              </h1>
              <p className="text-white/50 mt-1">
                Your outreach performance at a glance
              </p>
            </div>
            <button
              onClick={() => setShowCompose(true)}
              className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-indigo-500 to-purple-600 text-white font-semibold shadow-lg shadow-indigo-500/25 hover:shadow-indigo-500/40 transition-all hover:scale-105 active:scale-95"
            >
              <Sparkles className="w-5 h-5" />
              Compose with AI
            </button>
          </div>

          {/* Stats Grid - Psychology: Variable Rewards */}
          <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-7 gap-4">
            <StatCard
              label="Total Sent"
              value={stats?.total || 0}
              icon={<Send className="w-5 h-5" />}
              color="indigo"
            />
            <StatCard
              label="Delivered"
              value={stats?.delivered || 0}
              icon={<CheckCircle className="w-5 h-5" />}
              color="cyan"
            />
            <StatCard
              label="Opened"
              value={stats?.opened || 0}
              subValue={`${stats?.openRate || 0}%`}
              icon={<Eye className="w-5 h-5" />}
              color="emerald"
              highlight
            />
            <StatCard
              label="Clicked"
              value={stats?.clicked || 0}
              subValue={`${stats?.clickRate || 0}%`}
              icon={<MousePointer className="w-5 h-5" />}
              color="amber"
            />
            <StatCard
              label="Replied"
              value={stats?.replied || 0}
              subValue={`${stats?.replyRate || 0}%`}
              icon={<MessageSquare className="w-5 h-5" />}
              color="green"
              highlight
            />
            <StatCard
              label="Bounced"
              value={stats?.bounced || 0}
              icon={<AlertTriangle className="w-5 h-5" />}
              color="red"
              warning={stats?.bounced > 0}
            />
            <PerformanceCard openRate={parseFloat(stats?.openRate || '0')} />
          </div>
        </div>

        {/* Engagement Insights - Psychology: Social Proof + Loss Aversion */}
        <div className="grid md:grid-cols-3 gap-4 mb-8">
          <InsightCard
            icon={<Flame className="w-5 h-5 text-orange-400" />}
            title="Hot Leads"
            description={`${stats?.opened || 0} people opened your emails - follow up now!`}
            action="View Opened"
            actionColor="orange"
            onClick={() => setSelectedStatus('opened')}
          />
          <InsightCard
            icon={<Target className="w-5 h-5 text-emerald-400" />}
            title="Engaged Prospects"
            description={`${stats?.clicked || 0} clicked your links - they're interested!`}
            action="View Clicked"
            actionColor="emerald"
            onClick={() => setSelectedStatus('clicked')}
          />
          <InsightCard
            icon={<Award className="w-5 h-5 text-amber-400" />}
            title="Your Performance"
            description={
              parseFloat(stats?.openRate || '0') > 25
                ? `${stats?.openRate}% open rate - you're crushing it!`
                : `${stats?.openRate}% open rate - let's optimize your subject lines`
            }
            action="View Tips"
            actionColor="amber"
          />
        </div>

        {/* Filters and Search */}
        <div className="flex flex-col sm:flex-row gap-4 mb-6">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-white/30" />
            <input
              type="text"
              placeholder="Search emails..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10 pr-4 py-3 rounded-xl bg-white/5 border border-white/10 text-white placeholder:text-white/30 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-all"
            />
          </div>
          <div className="flex gap-2">
            <StatusFilter
              selected={selectedStatus}
              onSelect={setSelectedStatus}
            />
          </div>
        </div>

        {/* Email List */}
        <div className="bg-white/5 rounded-2xl border border-white/10 overflow-hidden">
          <div className="divide-y divide-white/5">
            {emailsLoading ? (
              // Skeleton loading - smooth UX
              [...Array(5)].map((_, i) => (
                <div key={i} className="p-4 animate-pulse">
                  <div className="flex items-center gap-4">
                    <div className="w-10 h-10 rounded-full bg-white/10"></div>
                    <div className="flex-1">
                      <div className="h-4 w-48 bg-white/10 rounded mb-2"></div>
                      <div className="h-3 w-64 bg-white/5 rounded"></div>
                    </div>
                    <div className="h-6 w-20 bg-white/10 rounded-full"></div>
                  </div>
                </div>
              ))
            ) : filteredEmails.length === 0 ? (
              <div className="p-12 text-center">
                <Mail className="w-12 h-12 text-white/20 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-white/60 mb-2">No emails yet</h3>
                <p className="text-white/40 mb-6">Start your outreach journey by composing your first email</p>
                <button
                  onClick={() => setShowCompose(true)}
                  className="inline-flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-indigo-500 to-purple-600 text-white font-semibold"
                >
                  <Sparkles className="w-5 h-5" />
                  Compose with AI
                </button>
              </div>
            ) : (
              filteredEmails.map((email: Email) => (
                <EmailRow key={email.id} email={email} />
              ))
            )}
          </div>
        </div>
      </main>

      {/* Compose Modal */}
      {showCompose && (
        <ComposeEmailModal
          leads={leads?.leads || []}
          onClose={() => setShowCompose(false)}
          onSuccess={() => {
            setShowCompose(false);
            queryClient.invalidateQueries({ queryKey: ['emails'] });
            queryClient.invalidateQueries({ queryKey: ['emailStats'] });
          }}
        />
      )}
    </div>
  );
}

// Stat Card with psychology-driven colors
function StatCard({
  label,
  value,
  subValue,
  icon,
  color,
  highlight,
  warning,
}: {
  label: string;
  value: number;
  subValue?: string;
  icon: React.ReactNode;
  color: 'indigo' | 'cyan' | 'emerald' | 'amber' | 'green' | 'red';
  highlight?: boolean;
  warning?: boolean;
}) {
  const colorMap = {
    indigo: 'from-indigo-500/20 to-indigo-500/5 border-indigo-500/20 text-indigo-400',
    cyan: 'from-cyan-500/20 to-cyan-500/5 border-cyan-500/20 text-cyan-400',
    emerald: 'from-emerald-500/20 to-emerald-500/5 border-emerald-500/20 text-emerald-400',
    amber: 'from-amber-500/20 to-amber-500/5 border-amber-500/20 text-amber-400',
    green: 'from-green-500/20 to-green-500/5 border-green-500/20 text-green-400',
    red: 'from-red-500/20 to-red-500/5 border-red-500/20 text-red-400',
  };

  return (
    <div
      className={`relative p-4 rounded-xl bg-gradient-to-br ${colorMap[color]} border backdrop-blur-sm transition-all hover:scale-105 ${
        highlight ? 'ring-2 ring-white/10' : ''
      } ${warning ? 'animate-pulse' : ''}`}
    >
      <div className={`${colorMap[color].split(' ').pop()} mb-2`}>{icon}</div>
      <div className="text-2xl font-bold text-white">{value.toLocaleString()}</div>
      {subValue && (
        <div className={`text-sm font-medium ${colorMap[color].split(' ').pop()}`}>
          {subValue}
        </div>
      )}
      <div className="text-xs text-white/40 mt-1">{label}</div>
    </div>
  );
}

// Performance Score Card - Gamification
function PerformanceCard({ openRate }: { openRate: number }) {
  const getScore = () => {
    if (openRate >= 40) return { label: 'Excellent', color: 'emerald', icon: '🔥' };
    if (openRate >= 25) return { label: 'Good', color: 'green', icon: '👍' };
    if (openRate >= 15) return { label: 'Average', color: 'amber', icon: '📈' };
    return { label: 'Needs Work', color: 'orange', icon: '💪' };
  };

  const score = getScore();

  return (
    <div className="p-4 rounded-xl bg-gradient-to-br from-purple-500/20 to-pink-500/10 border border-purple-500/20 backdrop-blur-sm">
      <div className="text-2xl mb-1">{score.icon}</div>
      <div className="text-lg font-bold text-white">{score.label}</div>
      <div className="text-xs text-white/40">Performance</div>
    </div>
  );
}

// Insight Card - Loss Aversion Trigger
function InsightCard({
  icon,
  title,
  description,
  action,
  actionColor,
  onClick,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  action: string;
  actionColor: 'orange' | 'emerald' | 'amber';
  onClick?: () => void;
}) {
  const colorMap = {
    orange: 'text-orange-400 hover:bg-orange-500/10',
    emerald: 'text-emerald-400 hover:bg-emerald-500/10',
    amber: 'text-amber-400 hover:bg-amber-500/10',
  };

  return (
    <div className="p-5 rounded-xl bg-white/5 border border-white/10 hover:border-white/20 transition-all group">
      <div className="flex items-start gap-3">
        <div className="p-2 rounded-lg bg-white/5">{icon}</div>
        <div className="flex-1">
          <h3 className="font-semibold text-white mb-1">{title}</h3>
          <p className="text-sm text-white/50 mb-3">{description}</p>
          {onClick && (
            <button
              onClick={onClick}
              className={`text-sm font-medium ${colorMap[actionColor]} px-3 py-1 rounded-lg transition-all`}
            >
              {action} <ChevronRight className="w-4 h-4 inline" />
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

// Status Filter Pills
function StatusFilter({
  selected,
  onSelect,
}: {
  selected: string | undefined;
  onSelect: (status: string | undefined) => void;
}) {
  const statuses = [
    { value: undefined, label: 'All' },
    { value: 'sent', label: 'Sent' },
    { value: 'opened', label: 'Opened' },
    { value: 'clicked', label: 'Clicked' },
    { value: 'replied', label: 'Replied' },
  ];

  return (
    <div className="flex gap-1 p-1 bg-white/5 rounded-xl">
      {statuses.map((status) => (
        <button
          key={status.value || 'all'}
          onClick={() => onSelect(status.value)}
          className={`px-4 py-2 rounded-lg text-sm font-medium transition-all ${
            selected === status.value
              ? 'bg-white/10 text-white'
              : 'text-white/50 hover:text-white hover:bg-white/5'
          }`}
        >
          {status.label}
        </button>
      ))}
    </div>
  );
}

// Email Row Component
function EmailRow({ email }: { email: Email }) {
  const status = statusConfig[email.status] || statusConfig.sent;

  return (
    <Link
      href={`/emails/${email.id}`}
      className="block p-4 hover:bg-white/5 transition-all group"
    >
      <div className="flex items-center gap-4">
        {/* Status Icon */}
        <div className={`w-10 h-10 rounded-full ${status.bg} flex items-center justify-center ${status.color}`}>
          {status.icon}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="font-medium text-white truncate">
              {email.contactName || email.toEmail}
            </span>
            {email.companyName && (
              <span className="text-white/40 text-sm truncate">
                @ {email.companyName}
              </span>
            )}
          </div>
          <p className="text-sm text-white/60 truncate">{email.subject}</p>
        </div>

        {/* Meta */}
        <div className="flex items-center gap-4">
          {/* Engagement indicators */}
          <div className="flex items-center gap-2">
            {email.openedAt && (
              <div className="flex items-center gap-1 text-emerald-400 text-xs">
                <Eye className="w-3 h-3" />
              </div>
            )}
            {email.clickedAt && (
              <div className="flex items-center gap-1 text-amber-400 text-xs">
                <MousePointer className="w-3 h-3" />
              </div>
            )}
          </div>

          {/* Status Badge */}
          <div className={`px-3 py-1 rounded-full ${status.bg} ${status.color} text-xs font-medium flex items-center gap-1`}>
            {status.icon}
            {status.label}
          </div>

          {/* Time */}
          <div className="text-xs text-white/40 w-24 text-right">
            {formatDate(email.sentAt)}
          </div>

          <ChevronRight className="w-5 h-5 text-white/20 group-hover:text-white/40 transition-colors" />
        </div>
      </div>
    </Link>
  );
}

// Main export with Suspense
export default function EmailsPage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
          <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-indigo-500"></div>
        </div>
      }
    >
      <EmailsPageContent />
    </Suspense>
  );
}

'use client';

import { useEffect } from 'react';
import { useRouter, useParams } from 'next/navigation';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { api } from '@/lib/api';
import { formatDate } from '@/lib/utils';
import Link from 'next/link';
import {
  ArrowLeft,
  Mail,
  Send,
  Eye,
  MousePointer,
  MessageSquare,
  Clock,
  Building,
  User,
  Briefcase,
  Globe,
  Linkedin,
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Target,
  Lightbulb,
  HelpCircle,
  ThumbsUp,
  ThumbsDown,
  Zap,
  Award,
  ExternalLink,
  MoreVertical,
  Trash2,
  Reply,
} from 'lucide-react';
import { toast } from 'sonner';

export default function EmailDetailPage() {
  const router = useRouter();
  const params = useParams();
  const queryClient = useQueryClient();
  const { isAuthenticated, isLoading: authLoading, checkAuth } = useAuth();

  const emailId = params.id as string;

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth/login');
    }
  }, [isAuthenticated, authLoading, router]);

  // Fetch email details
  const { data: email, isLoading } = useQuery({
    queryKey: ['email', emailId],
    queryFn: () => (api as any).getEmail(emailId),
    enabled: isAuthenticated && !!emailId,
    refetchInterval: 10000, // Refresh for real-time tracking updates
  });

  // Mark as replied mutation
  const markRepliedMutation = useMutation({
    mutationFn: () => (api as any).markEmailReplied(emailId),
    onSuccess: () => {
      toast.success('Marked as replied!');
      queryClient.invalidateQueries({ queryKey: ['email', emailId] });
      queryClient.invalidateQueries({ queryKey: ['emails'] });
    },
  });

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: () => (api as any).deleteEmail(emailId),
    onSuccess: () => {
      toast.success('Email deleted');
      router.push('/emails');
    },
  });

  if (authLoading || !isAuthenticated || isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-indigo-500"></div>
      </div>
    );
  }

  if (!email) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
        <div className="text-center">
          <Mail className="w-12 h-12 text-white/20 mx-auto mb-4" />
          <h2 className="text-xl font-medium text-white mb-2">Email not found</h2>
          <Link href="/emails" className="text-indigo-400 hover:text-indigo-300">
            Back to emails
          </Link>
        </div>
      </div>
    );
  }

  const statusTimeline = [
    { status: 'sent', time: email.sentAt, icon: <Send className="w-4 h-4" />, label: 'Sent' },
    { status: 'opened', time: email.openedAt, icon: <Eye className="w-4 h-4" />, label: 'Opened' },
    { status: 'clicked', time: email.clickedAt, icon: <MousePointer className="w-4 h-4" />, label: 'Clicked' },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-950 via-slate-900 to-slate-950">
      {/* Header */}
      <header className="border-b border-white/5 bg-black/20 backdrop-blur-xl sticky top-0 z-40">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16 items-center">
            <div className="flex items-center gap-4">
              <Link
                href="/emails"
                className="p-2 rounded-lg hover:bg-white/10 text-white/60 hover:text-white transition-colors"
              >
                <ArrowLeft className="w-5 h-5" />
              </Link>
              <div>
                <h1 className="text-lg font-semibold text-white truncate max-w-md">
                  {email.subject}
                </h1>
                <p className="text-sm text-white/50">
                  To: {email.toEmail}
                </p>
              </div>
            </div>
            <div className="flex items-center gap-2">
              {email.status !== 'replied' && (
                <button
                  onClick={() => markRepliedMutation.mutate()}
                  className="px-4 py-2 rounded-lg bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30 transition-colors flex items-center gap-2 text-sm font-medium"
                >
                  <Reply className="w-4 h-4" />
                  Mark Replied
                </button>
              )}
              <button
                onClick={() => {
                  if (confirm('Delete this email?')) {
                    deleteMutation.mutate();
                  }
                }}
                className="p-2 rounded-lg hover:bg-red-500/20 text-white/60 hover:text-red-400 transition-colors"
              >
                <Trash2 className="w-5 h-5" />
              </button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid lg:grid-cols-3 gap-8">
          {/* Main Content - Email & Tracking */}
          <div className="lg:col-span-2 space-y-6">
            {/* Engagement Timeline - The dopamine track */}
            <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <Zap className="w-5 h-5 text-amber-400" />
                Engagement Timeline
              </h2>
              <div className="flex items-center gap-4">
                {statusTimeline.map((item, index) => (
                  <div key={item.status} className="flex items-center">
                    <div
                      className={`flex flex-col items-center ${
                        item.time ? 'text-emerald-400' : 'text-white/20'
                      }`}
                    >
                      <div
                        className={`w-12 h-12 rounded-full flex items-center justify-center ${
                          item.time
                            ? 'bg-emerald-500/20 ring-2 ring-emerald-500/50'
                            : 'bg-white/5'
                        }`}
                      >
                        {item.icon}
                      </div>
                      <span className="text-xs mt-2 font-medium">{item.label}</span>
                      {item.time && (
                        <span className="text-xs text-white/40 mt-1">
                          {formatDate(item.time)}
                        </span>
                      )}
                    </div>
                    {index < statusTimeline.length - 1 && (
                      <div
                        className={`w-16 h-0.5 mx-2 ${
                          statusTimeline[index + 1].time
                            ? 'bg-emerald-500/50'
                            : 'bg-white/10'
                        }`}
                      />
                    )}
                  </div>
                ))}
              </div>

              {/* Engagement Stats */}
              {email.openedAt && (
                <div className="mt-6 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                  <div className="flex items-center gap-2 text-emerald-400 font-medium mb-2">
                    <CheckCircle className="w-5 h-5" />
                    This email was opened!
                  </div>
                  <p className="text-sm text-white/60">
                    {email.contactName || 'The recipient'} opened your email on{' '}
                    {new Date(email.openedAt).toLocaleString()}
                    {email.clickedAt && ' and clicked a link inside.'}
                  </p>
                </div>
              )}

              {!email.openedAt && email.status === 'sent' && (
                <div className="mt-6 p-4 rounded-xl bg-amber-500/10 border border-amber-500/20">
                  <div className="flex items-center gap-2 text-amber-400 font-medium mb-2">
                    <Clock className="w-5 h-5" />
                    Waiting for engagement
                  </div>
                  <p className="text-sm text-white/60">
                    Your email was sent successfully. We'll notify you when it's opened.
                  </p>
                </div>
              )}
            </div>

            {/* Email Content */}
            <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
              <h2 className="text-lg font-semibold text-white mb-4">Email Content</h2>
              <div className="p-4 rounded-xl bg-white/5 border border-white/10">
                <div className="mb-4 pb-4 border-b border-white/10">
                  <div className="text-sm text-white/50 mb-1">Subject</div>
                  <div className="font-medium text-white">{email.subject}</div>
                </div>
                <div
                  className="prose prose-invert prose-sm max-w-none"
                  dangerouslySetInnerHTML={{
                    __html: email.metadata?.bodyHtml || '<p class="text-white/40">Email content not available</p>',
                  }}
                />
              </div>
            </div>
          </div>

          {/* Sidebar - Lead Analysis */}
          <div className="space-y-6">
            {/* Lead Card */}
            {email.lead && (
              <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
                <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                  <User className="w-5 h-5 text-indigo-400" />
                  Lead Info
                </h2>
                <div className="space-y-4">
                  <div className="flex items-center gap-3">
                    <div className="w-12 h-12 rounded-full bg-indigo-500/20 flex items-center justify-center">
                      <User className="w-6 h-6 text-indigo-400" />
                    </div>
                    <div>
                      <div className="font-medium text-white">{email.lead.contactName}</div>
                      <div className="text-sm text-white/50">{email.lead.contactTitle}</div>
                    </div>
                  </div>

                  <div className="space-y-2 pt-4 border-t border-white/10">
                    <InfoRow icon={<Building className="w-4 h-4" />} label="Company" value={email.lead.companyName} />
                    <InfoRow icon={<Globe className="w-4 h-4" />} label="Industry" value={email.lead.industry} />
                    <InfoRow icon={<User className="w-4 h-4" />} label="Size" value={email.lead.companySize} />
                    {email.lead.contactLinkedin && (
                      <a
                        href={email.lead.contactLinkedin}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="flex items-center gap-2 text-sm text-indigo-400 hover:text-indigo-300 transition-colors"
                      >
                        <Linkedin className="w-4 h-4" />
                        View LinkedIn
                        <ExternalLink className="w-3 h-3" />
                      </a>
                    )}
                  </div>

                  <Link
                    href={`/leads/${email.lead.id}`}
                    className="block w-full py-3 rounded-xl bg-indigo-500/20 text-indigo-400 font-medium text-center hover:bg-indigo-500/30 transition-colors"
                  >
                    View Full Profile
                  </Link>
                </div>
              </div>
            )}

            {/* AI Analysis - The killer feature */}
            {email.analysis && (
              <>
                {/* Match Score */}
                <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
                  <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <Target className="w-5 h-5 text-emerald-400" />
                    Match Score
                  </h2>
                  <div className="flex items-center gap-4">
                    <div className="relative w-20 h-20">
                      <svg className="w-full h-full -rotate-90">
                        <circle
                          cx="40"
                          cy="40"
                          r="36"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="8"
                          className="text-white/10"
                        />
                        <circle
                          cx="40"
                          cy="40"
                          r="36"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="8"
                          strokeDasharray={`${(email.analysis.matchScore / 100) * 226} 226`}
                          className={
                            email.analysis.matchScore >= 70
                              ? 'text-emerald-400'
                              : email.analysis.matchScore >= 50
                              ? 'text-amber-400'
                              : 'text-red-400'
                          }
                          strokeLinecap="round"
                        />
                      </svg>
                      <div className="absolute inset-0 flex items-center justify-center">
                        <span className="text-2xl font-bold text-white">{email.analysis.matchScore}</span>
                      </div>
                    </div>
                    <div className="flex-1">
                      <div className="text-sm text-white/60 mb-1">
                        {email.analysis.matchScore >= 70
                          ? 'Excellent Match'
                          : email.analysis.matchScore >= 50
                          ? 'Good Potential'
                          : 'Low Match'}
                      </div>
                      <p className="text-xs text-white/40">{email.analysis.summary}</p>
                    </div>
                  </div>
                </div>

                {/* Pros */}
                <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
                  <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <ThumbsUp className="w-5 h-5 text-emerald-400" />
                    Pros
                  </h2>
                  <div className="space-y-3">
                    {email.analysis.pros.map((pro: { point: string; reasoning: string }, i: number) => (
                      <div key={i} className="p-3 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                        <div className="font-medium text-emerald-400 text-sm mb-1">{pro.point}</div>
                        <div className="text-xs text-white/50">{pro.reasoning}</div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Cons */}
                <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
                  <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <ThumbsDown className="w-5 h-5 text-amber-400" />
                    Cons & Mitigations
                  </h2>
                  <div className="space-y-3">
                    {email.analysis.cons.map((con: { point: string; mitigation: string }, i: number) => (
                      <div key={i} className="p-3 rounded-xl bg-amber-500/10 border border-amber-500/20">
                        <div className="font-medium text-amber-400 text-sm mb-1">{con.point}</div>
                        <div className="text-xs text-white/50">
                          <span className="text-amber-400/70">Mitigation:</span> {con.mitigation}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Opportunities */}
                <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
                  <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <Lightbulb className="w-5 h-5 text-purple-400" />
                    Opportunities
                  </h2>
                  <div className="space-y-3">
                    {email.analysis.opportunities.map((opp: { description: string; howToLeverage: string; potentialValue: string }, i: number) => (
                      <div key={i} className="p-3 rounded-xl bg-purple-500/10 border border-purple-500/20">
                        <div className="font-medium text-purple-400 text-sm mb-1">{opp.description}</div>
                        <div className="text-xs text-white/50 mb-1">
                          <span className="text-purple-400/70">How to leverage:</span> {opp.howToLeverage}
                        </div>
                        <div className="text-xs text-emerald-400/70">
                          Potential: {opp.potentialValue}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>

                {/* Next Steps */}
                <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
                  <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <Award className="w-5 h-5 text-cyan-400" />
                    Next Steps
                  </h2>
                  <ul className="space-y-2">
                    {email.analysis.nextSteps.map((step: string, i: number) => (
                      <li key={i} className="flex items-start gap-2 text-sm text-white/70">
                        <CheckCircle className="w-4 h-4 text-cyan-400 shrink-0 mt-0.5" />
                        {step}
                      </li>
                    ))}
                  </ul>
                </div>

                {/* Questions to Ask */}
                <div className="bg-white/5 rounded-2xl border border-white/10 p-6">
                  <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                    <HelpCircle className="w-5 h-5 text-pink-400" />
                    Questions to Ask
                  </h2>
                  <ul className="space-y-2">
                    {email.analysis.questionsToAsk.map((q: string, i: number) => (
                      <li key={i} className="flex items-start gap-2 text-sm text-white/70">
                        <span className="text-pink-400">?</span>
                        {q}
                      </li>
                    ))}
                  </ul>
                </div>
              </>
            )}

            {/* No Analysis Yet */}
            {!email.analysis && email.lead && (
              <div className="bg-white/5 rounded-2xl border border-white/10 p-6 text-center">
                <Target className="w-12 h-12 text-white/20 mx-auto mb-4" />
                <h3 className="font-medium text-white mb-2">No AI Analysis Yet</h3>
                <p className="text-sm text-white/50 mb-4">
                  Run AI analysis on this lead to get insights
                </p>
                <Link
                  href={`/leads/${email.lead.id}?analyze=true`}
                  className="inline-flex items-center gap-2 px-4 py-2 rounded-lg bg-indigo-500/20 text-indigo-400 text-sm font-medium hover:bg-indigo-500/30 transition-colors"
                >
                  <Zap className="w-4 h-4" />
                  Analyze Lead
                </Link>
              </div>
            )}
          </div>
        </div>
      </main>
    </div>
  );
}

function InfoRow({
  icon,
  label,
  value,
}: {
  icon: React.ReactNode;
  label: string;
  value: string | null;
}) {
  if (!value) return null;
  return (
    <div className="flex items-center gap-3 text-sm">
      <span className="text-white/40">{icon}</span>
      <span className="text-white/40">{label}:</span>
      <span className="text-white">{value}</span>
    </div>
  );
}

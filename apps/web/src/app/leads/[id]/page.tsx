'use client';

import { useState, useEffect } from 'react';
import { useRouter, useParams } from 'next/navigation';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { api, Lead } from '@/lib/api';
import { formatDate, getStatusColor, formatDuration } from '@/lib/utils';
import Link from 'next/link';
import {
  ArrowLeft,
  Building2,
  User,
  Mail,
  Phone,
  Globe,
  Linkedin,
  Edit2,
  Trash2,
  Save,
  X,
  Clock,
  Tag,
  FileText,
  Mic,
} from 'lucide-react';
import { toast } from 'sonner';

const STATUSES = ['new', 'contacted', 'qualified', 'proposal', 'negotiation', 'won', 'lost'];

export default function LeadDetailPage() {
  const router = useRouter();
  const params = useParams();
  const queryClient = useQueryClient();
  const { isAuthenticated, isLoading: authLoading, checkAuth } = useAuth();

  const [isEditing, setIsEditing] = useState(false);
  const [editData, setEditData] = useState<Partial<Lead>>({});
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const leadId = params.id as string;

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth/login');
    }
  }, [isAuthenticated, authLoading, router]);

  const { data: lead, isLoading } = useQuery({
    queryKey: ['lead', leadId],
    queryFn: () => api.getLead(leadId),
    enabled: isAuthenticated && !!leadId,
  });

  const { data: recordings } = useQuery({
    queryKey: ['lead-recordings', leadId],
    queryFn: () => api.getRecordings({ leadId }),
    enabled: isAuthenticated && !!leadId,
  });

  const updateMutation = useMutation({
    mutationFn: (data: Partial<Lead>) => api.updateLead(leadId, data),
    onSuccess: () => {
      toast.success('Lead updated');
      queryClient.invalidateQueries({ queryKey: ['lead', leadId] });
      queryClient.invalidateQueries({ queryKey: ['leads'] });
      setIsEditing(false);
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to update lead');
    },
  });

  const deleteMutation = useMutation({
    mutationFn: () => api.deleteLead(leadId),
    onSuccess: () => {
      toast.success('Lead deleted');
      router.push('/leads');
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to delete lead');
    },
  });

  const startEditing = () => {
    if (lead) {
      setEditData({
        company_name: lead.company_name,
        company_domain: lead.company_domain,
        company_linkedin: lead.company_linkedin,
        company_size: lead.company_size,
        industry: lead.industry,
        contact_name: lead.contact_name,
        contact_title: lead.contact_title,
        contact_email: lead.contact_email,
        contact_phone: lead.contact_phone,
        contact_linkedin: lead.contact_linkedin,
        status: lead.status,
        priority: lead.priority,
        notes: lead.notes,
      });
      setIsEditing(true);
    }
  };

  const saveChanges = () => {
    updateMutation.mutate(editData);
  };

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
              <Link href="/leads" className="flex items-center text-muted-foreground hover:text-foreground">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Leads
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

  if (!lead) {
    return (
      <div className="min-h-screen bg-background">
        <header className="border-b border-border bg-card">
          <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div className="flex h-16 items-center">
              <Link href="/leads" className="flex items-center text-muted-foreground hover:text-foreground">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Back to Leads
              </Link>
            </div>
          </div>
        </header>
        <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
          <div className="text-center py-12">
            <h2 className="text-xl font-bold mb-2">Lead not found</h2>
            <p className="text-muted-foreground">This lead may have been deleted.</p>
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
          <div className="flex justify-between h-16 items-center">
            <Link href="/leads" className="flex items-center text-muted-foreground hover:text-foreground">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Leads
            </Link>
            <div className="flex items-center space-x-2">
              {isEditing ? (
                <>
                  <button
                    onClick={() => setIsEditing(false)}
                    className="flex items-center space-x-2 px-4 py-2 rounded-lg border border-border hover:bg-muted"
                  >
                    <X className="w-4 h-4" />
                    <span>Cancel</span>
                  </button>
                  <button
                    onClick={saveChanges}
                    disabled={updateMutation.isPending}
                    className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
                  >
                    <Save className="w-4 h-4" />
                    <span>{updateMutation.isPending ? 'Saving...' : 'Save'}</span>
                  </button>
                </>
              ) : (
                <>
                  <button
                    onClick={startEditing}
                    className="flex items-center space-x-2 px-4 py-2 rounded-lg border border-border hover:bg-muted"
                  >
                    <Edit2 className="w-4 h-4" />
                    <span>Edit</span>
                  </button>
                  <button
                    onClick={() => setShowDeleteConfirm(true)}
                    className="flex items-center space-x-2 px-4 py-2 rounded-lg border border-red-500 text-red-500 hover:bg-red-500/10"
                  >
                    <Trash2 className="w-4 h-4" />
                    <span>Delete</span>
                  </button>
                </>
              )}
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid lg:grid-cols-3 gap-8">
          {/* Main Info */}
          <div className="lg:col-span-2 space-y-6">
            {/* Company Card */}
            <div className="bg-card rounded-xl border border-border p-6">
              <div className="flex items-start justify-between mb-6">
                <div className="flex items-center space-x-4">
                  <div className="w-16 h-16 rounded-xl bg-primary/10 flex items-center justify-center">
                    <Building2 className="w-8 h-8 text-primary" />
                  </div>
                  <div>
                    {isEditing ? (
                      <input
                        type="text"
                        value={editData.company_name || ''}
                        onChange={(e) => setEditData({ ...editData, company_name: e.target.value })}
                        className="text-2xl font-bold bg-background border border-input rounded px-2 py-1"
                      />
                    ) : (
                      <h1 className="text-2xl font-bold">{lead.company_name}</h1>
                    )}
                    {isEditing ? (
                      <input
                        type="text"
                        value={editData.industry || ''}
                        onChange={(e) => setEditData({ ...editData, industry: e.target.value })}
                        placeholder="Industry"
                        className="text-muted-foreground bg-background border border-input rounded px-2 py-1 mt-1"
                      />
                    ) : (
                      lead.industry && <p className="text-muted-foreground">{lead.industry}</p>
                    )}
                  </div>
                </div>
                {isEditing ? (
                  <select
                    value={editData.status || 'new'}
                    onChange={(e) => setEditData({ ...editData, status: e.target.value })}
                    className="px-3 py-1 rounded-full text-sm bg-background border border-input"
                  >
                    {STATUSES.map((s) => (
                      <option key={s} value={s}>{s}</option>
                    ))}
                  </select>
                ) : (
                  <span className={`px-3 py-1 rounded-full text-sm font-medium ${getStatusColor(lead.status)}`}>
                    {lead.status}
                  </span>
                )}
              </div>

              <div className="grid sm:grid-cols-2 gap-4">
                <InfoRow
                  icon={<Globe className="w-4 h-4" />}
                  label="Website"
                  value={isEditing ? (
                    <input
                      type="text"
                      value={editData.company_domain || ''}
                      onChange={(e) => setEditData({ ...editData, company_domain: e.target.value })}
                      placeholder="company.com"
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.company_domain ? (
                    <a href={`https://${lead.company_domain}`} target="_blank" rel="noopener noreferrer" className="text-primary hover:underline">
                      {lead.company_domain}
                    </a>
                  ) : '-'}
                />
                <InfoRow
                  icon={<Linkedin className="w-4 h-4" />}
                  label="LinkedIn"
                  value={isEditing ? (
                    <input
                      type="text"
                      value={editData.company_linkedin || ''}
                      onChange={(e) => setEditData({ ...editData, company_linkedin: e.target.value })}
                      placeholder="linkedin.com/company/..."
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.company_linkedin ? (
                    <a href={lead.company_linkedin} target="_blank" rel="noopener noreferrer" className="text-primary hover:underline">
                      View Profile
                    </a>
                  ) : '-'}
                />
                <InfoRow
                  icon={<User className="w-4 h-4" />}
                  label="Company Size"
                  value={isEditing ? (
                    <input
                      type="text"
                      value={editData.company_size || ''}
                      onChange={(e) => setEditData({ ...editData, company_size: e.target.value })}
                      placeholder="50-200"
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.company_size || '-'}
                />
                <InfoRow
                  icon={<Clock className="w-4 h-4" />}
                  label="Added"
                  value={formatDate(lead.created_at)}
                />
              </div>
            </div>

            {/* Contact Card */}
            <div className="bg-card rounded-xl border border-border p-6">
              <h2 className="text-lg font-semibold mb-4 flex items-center">
                <User className="w-5 h-5 mr-2" />
                Contact Information
              </h2>
              <div className="grid sm:grid-cols-2 gap-4">
                <InfoRow
                  icon={<User className="w-4 h-4" />}
                  label="Name"
                  value={isEditing ? (
                    <input
                      type="text"
                      value={editData.contact_name || ''}
                      onChange={(e) => setEditData({ ...editData, contact_name: e.target.value })}
                      placeholder="John Smith"
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.contact_name || '-'}
                />
                <InfoRow
                  icon={<Tag className="w-4 h-4" />}
                  label="Title"
                  value={isEditing ? (
                    <input
                      type="text"
                      value={editData.contact_title || ''}
                      onChange={(e) => setEditData({ ...editData, contact_title: e.target.value })}
                      placeholder="VP of Sales"
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.contact_title || '-'}
                />
                <InfoRow
                  icon={<Mail className="w-4 h-4" />}
                  label="Email"
                  value={isEditing ? (
                    <input
                      type="email"
                      value={editData.contact_email || ''}
                      onChange={(e) => setEditData({ ...editData, contact_email: e.target.value })}
                      placeholder="john@company.com"
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.contact_email ? (
                    <a href={`mailto:${lead.contact_email}`} className="text-primary hover:underline">
                      {lead.contact_email}
                    </a>
                  ) : '-'}
                />
                <InfoRow
                  icon={<Phone className="w-4 h-4" />}
                  label="Phone"
                  value={isEditing ? (
                    <input
                      type="tel"
                      value={editData.contact_phone || ''}
                      onChange={(e) => setEditData({ ...editData, contact_phone: e.target.value })}
                      placeholder="+1 555 123 4567"
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.contact_phone ? (
                    <a href={`tel:${lead.contact_phone}`} className="text-primary hover:underline">
                      {lead.contact_phone}
                    </a>
                  ) : '-'}
                />
                <InfoRow
                  icon={<Linkedin className="w-4 h-4" />}
                  label="LinkedIn"
                  value={isEditing ? (
                    <input
                      type="text"
                      value={editData.contact_linkedin || ''}
                      onChange={(e) => setEditData({ ...editData, contact_linkedin: e.target.value })}
                      placeholder="linkedin.com/in/..."
                      className="bg-background border border-input rounded px-2 py-1 w-full"
                    />
                  ) : lead.contact_linkedin ? (
                    <a href={lead.contact_linkedin} target="_blank" rel="noopener noreferrer" className="text-primary hover:underline">
                      View Profile
                    </a>
                  ) : '-'}
                />
              </div>
            </div>

            {/* Notes Card */}
            <div className="bg-card rounded-xl border border-border p-6">
              <h2 className="text-lg font-semibold mb-4 flex items-center">
                <FileText className="w-5 h-5 mr-2" />
                Notes
              </h2>
              {isEditing ? (
                <textarea
                  value={editData.notes || ''}
                  onChange={(e) => setEditData({ ...editData, notes: e.target.value })}
                  rows={5}
                  className="w-full bg-background border border-input rounded-lg px-4 py-3 resize-none"
                  placeholder="Add notes about this lead..."
                />
              ) : (
                <p className="text-muted-foreground whitespace-pre-wrap">
                  {lead.notes || 'No notes yet.'}
                </p>
              )}
            </div>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Related Recordings */}
            <div className="bg-card rounded-xl border border-border p-6">
              <h2 className="text-lg font-semibold mb-4 flex items-center">
                <Mic className="w-5 h-5 mr-2" />
                Related Calls
              </h2>
              <div className="space-y-3">
                {recordings?.recordings && recordings.recordings.length > 0 ? (
                  recordings.recordings.map((recording) => (
                    <Link
                      key={recording.id}
                      href={`/recordings/${recording.id}`}
                      className="block p-3 rounded-lg border border-border hover:border-primary/50 transition-colors"
                    >
                      <div className="flex justify-between items-start">
                        <div>
                          <p className="font-medium capitalize">{recording.mode} Call</p>
                          <p className="text-sm text-muted-foreground">
                            {formatDate(recording.start_time)}
                          </p>
                        </div>
                        {recording.duration_seconds && (
                          <span className="text-sm text-muted-foreground">
                            {formatDuration(recording.duration_seconds)}
                          </span>
                        )}
                      </div>
                    </Link>
                  ))
                ) : (
                  <p className="text-muted-foreground text-center py-4">
                    No calls recorded for this lead yet.
                  </p>
                )}
              </div>
            </div>

            {/* Tech Stack (if available) */}
            {lead.tech_stack && lead.tech_stack.length > 0 && (
              <div className="bg-card rounded-xl border border-border p-6">
                <h2 className="text-lg font-semibold mb-4">Tech Stack</h2>
                <div className="flex flex-wrap gap-2">
                  {lead.tech_stack.map((tech, i) => (
                    <span key={i} className="px-3 py-1 rounded-full text-sm bg-primary/10 text-primary">
                      {tech}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Tags (if available) */}
            {lead.tags && lead.tags.length > 0 && (
              <div className="bg-card rounded-xl border border-border p-6">
                <h2 className="text-lg font-semibold mb-4">Tags</h2>
                <div className="flex flex-wrap gap-2">
                  {lead.tags.map((tag, i) => (
                    <span key={i} className="px-3 py-1 rounded-full text-sm bg-muted text-muted-foreground">
                      {tag}
                    </span>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </main>

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="absolute inset-0 bg-black/50" onClick={() => setShowDeleteConfirm(false)} />
          <div className="relative bg-card rounded-2xl border border-border p-6 w-full max-w-md">
            <h2 className="text-xl font-bold mb-2">Delete Lead</h2>
            <p className="text-muted-foreground mb-6">
              Are you sure you want to delete {lead.company_name}? This action cannot be undone.
            </p>
            <div className="flex justify-end space-x-3">
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="px-4 py-2 rounded-lg border border-border hover:bg-muted"
              >
                Cancel
              </button>
              <button
                onClick={() => deleteMutation.mutate()}
                disabled={deleteMutation.isPending}
                className="px-4 py-2 rounded-lg bg-red-500 text-white hover:bg-red-600 disabled:opacity-50"
              >
                {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
              </button>
            </div>
          </div>
        </div>
      )}
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
  value: React.ReactNode;
}) {
  return (
    <div className="flex items-start space-x-3">
      <div className="p-2 rounded-lg bg-muted text-muted-foreground">
        {icon}
      </div>
      <div className="flex-1 min-w-0">
        <p className="text-sm text-muted-foreground">{label}</p>
        <div className="font-medium truncate">{value}</div>
      </div>
    </div>
  );
}

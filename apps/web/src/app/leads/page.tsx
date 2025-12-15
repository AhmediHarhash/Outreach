'use client';

import { useState, useEffect, Suspense } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useAuth } from '@/lib/auth';
import { api, Lead } from '@/lib/api';
import { formatDate, getStatusColor } from '@/lib/utils';
import Link from 'next/link';
import {
  Search,
  Plus,
  Filter,
  ChevronLeft,
  ChevronRight,
  Building2,
  User,
  Mail,
  Phone,
  X,
} from 'lucide-react';
import { toast } from 'sonner';

const STATUSES = ['new', 'contacted', 'qualified', 'proposal', 'negotiation', 'won', 'lost'];

function LeadsPageContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const queryClient = useQueryClient();
  const { isAuthenticated, isLoading: authLoading, checkAuth } = useAuth();

  const [page, setPage] = useState(1);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('');
  const [showNewModal, setShowNewModal] = useState(false);

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth/login');
    }
  }, [isAuthenticated, authLoading, router]);

  useEffect(() => {
    if (searchParams.get('new') === 'true') {
      setShowNewModal(true);
    }
  }, [searchParams]);

  const { data, isLoading } = useQuery({
    queryKey: ['leads', page, search, statusFilter],
    queryFn: () => api.getLeads({ page, search: search || undefined, status: statusFilter || undefined }),
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
              <Link href="/leads" className="text-foreground font-medium">
                Leads
              </Link>
              <Link href="/recordings" className="text-muted-foreground hover:text-foreground">
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
        <div className="flex justify-between items-center mb-8">
          <div>
            <h1 className="text-3xl font-bold">Leads</h1>
            <p className="text-muted-foreground mt-1">
              Manage your sales pipeline
            </p>
          </div>
          <button
            onClick={() => setShowNewModal(true)}
            className="flex items-center space-x-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
          >
            <Plus className="w-4 h-4" />
            <span>Add Lead</span>
          </button>
        </div>

        {/* Filters */}
        <div className="flex flex-col sm:flex-row gap-4 mb-6">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              type="text"
              value={search}
              onChange={(e) => {
                setSearch(e.target.value);
                setPage(1);
              }}
              placeholder="Search leads..."
              className="w-full pl-10 pr-4 py-2 rounded-lg bg-background border border-input focus:border-primary focus:ring-1 focus:ring-primary outline-none"
            />
          </div>
          <div className="flex items-center space-x-2">
            <Filter className="w-4 h-4 text-muted-foreground" />
            <select
              value={statusFilter}
              onChange={(e) => {
                setStatusFilter(e.target.value);
                setPage(1);
              }}
              className="px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
            >
              <option value="">All Statuses</option>
              {STATUSES.map((status) => (
                <option key={status} value={status}>
                  {status.charAt(0).toUpperCase() + status.slice(1)}
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Leads Table */}
        <div className="bg-card rounded-xl border border-border overflow-hidden">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
            </div>
          ) : data?.leads && data.leads.length > 0 ? (
            <>
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead className="bg-muted/50">
                    <tr>
                      <th className="text-left px-6 py-3 text-sm font-medium text-muted-foreground">
                        Company
                      </th>
                      <th className="text-left px-6 py-3 text-sm font-medium text-muted-foreground">
                        Contact
                      </th>
                      <th className="text-left px-6 py-3 text-sm font-medium text-muted-foreground">
                        Status
                      </th>
                      <th className="text-left px-6 py-3 text-sm font-medium text-muted-foreground">
                        Industry
                      </th>
                      <th className="text-left px-6 py-3 text-sm font-medium text-muted-foreground">
                        Added
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-border">
                    {data.leads.map((lead) => (
                      <tr
                        key={lead.id}
                        onClick={() => router.push(`/leads/${lead.id}`)}
                        className="hover:bg-muted/30 cursor-pointer transition-colors"
                      >
                        <td className="px-6 py-4">
                          <div className="flex items-center space-x-3">
                            <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center">
                              <Building2 className="w-5 h-5 text-primary" />
                            </div>
                            <div>
                              <p className="font-medium">{lead.company_name}</p>
                              {lead.company_domain && (
                                <p className="text-sm text-muted-foreground">{lead.company_domain}</p>
                              )}
                            </div>
                          </div>
                        </td>
                        <td className="px-6 py-4">
                          {lead.contact_name ? (
                            <div>
                              <p className="font-medium">{lead.contact_name}</p>
                              {lead.contact_title && (
                                <p className="text-sm text-muted-foreground">{lead.contact_title}</p>
                              )}
                            </div>
                          ) : (
                            <span className="text-muted-foreground">No contact</span>
                          )}
                        </td>
                        <td className="px-6 py-4">
                          <span className={`px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(lead.status)}`}>
                            {lead.status}
                          </span>
                        </td>
                        <td className="px-6 py-4 text-muted-foreground">
                          {lead.industry || '-'}
                        </td>
                        <td className="px-6 py-4 text-muted-foreground">
                          {formatDate(lead.created_at)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>

              {/* Pagination */}
              {data.total_pages > 1 && (
                <div className="flex items-center justify-between px-6 py-4 border-t border-border">
                  <p className="text-sm text-muted-foreground">
                    Showing {((page - 1) * 20) + 1} to {Math.min(page * 20, data.total)} of {data.total} leads
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
            <div className="text-center py-12">
              <Building2 className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="text-lg font-medium mb-2">No leads yet</h3>
              <p className="text-muted-foreground mb-4">
                Add your first lead to get started
              </p>
              <button
                onClick={() => setShowNewModal(true)}
                className="inline-flex items-center space-x-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90"
              >
                <Plus className="w-4 h-4" />
                <span>Add Lead</span>
              </button>
            </div>
          )}
        </div>
      </main>

      {/* New Lead Modal */}
      {showNewModal && (
        <NewLeadModal
          onClose={() => {
            setShowNewModal(false);
            router.replace('/leads', { scroll: false });
          }}
          onSuccess={() => {
            setShowNewModal(false);
            router.replace('/leads', { scroll: false });
            queryClient.invalidateQueries({ queryKey: ['leads'] });
          }}
        />
      )}
    </div>
  );
}

export default function LeadsPage() {
  return (
    <Suspense fallback={
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
      </div>
    }>
      <LeadsPageContent />
    </Suspense>
  );
}

function NewLeadModal({
  onClose,
  onSuccess,
}: {
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [isLoading, setIsLoading] = useState(false);
  const [formData, setFormData] = useState({
    company_name: '',
    company_domain: '',
    contact_name: '',
    contact_title: '',
    contact_email: '',
    contact_phone: '',
    industry: '',
    status: 'new',
    notes: '',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);

    try {
      await api.createLead(formData);
      toast.success('Lead created successfully');
      onSuccess();
    } catch (error: any) {
      toast.error(error.message || 'Failed to create lead');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      <div className="relative bg-card rounded-2xl border border-border p-6 w-full max-w-lg max-h-[90vh] overflow-y-auto">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-bold">Add New Lead</h2>
          <button onClick={onClose} className="p-2 hover:bg-muted rounded-lg">
            <X className="w-5 h-5" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Company Info */}
          <div className="space-y-4">
            <h3 className="text-sm font-medium text-muted-foreground flex items-center">
              <Building2 className="w-4 h-4 mr-2" />
              Company Information
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div className="col-span-2">
                <label className="block text-sm font-medium mb-1">Company Name *</label>
                <input
                  type="text"
                  required
                  value={formData.company_name}
                  onChange={(e) => setFormData({ ...formData, company_name: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
                  placeholder="Acme Inc."
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Domain</label>
                <input
                  type="text"
                  value={formData.company_domain}
                  onChange={(e) => setFormData({ ...formData, company_domain: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
                  placeholder="acme.com"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Industry</label>
                <input
                  type="text"
                  value={formData.industry}
                  onChange={(e) => setFormData({ ...formData, industry: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
                  placeholder="Technology"
                />
              </div>
            </div>
          </div>

          {/* Contact Info */}
          <div className="space-y-4 pt-4 border-t border-border">
            <h3 className="text-sm font-medium text-muted-foreground flex items-center">
              <User className="w-4 h-4 mr-2" />
              Contact Information
            </h3>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-1">Contact Name</label>
                <input
                  type="text"
                  value={formData.contact_name}
                  onChange={(e) => setFormData({ ...formData, contact_name: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
                  placeholder="John Smith"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Title</label>
                <input
                  type="text"
                  value={formData.contact_title}
                  onChange={(e) => setFormData({ ...formData, contact_title: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
                  placeholder="VP of Sales"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Email</label>
                <input
                  type="email"
                  value={formData.contact_email}
                  onChange={(e) => setFormData({ ...formData, contact_email: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
                  placeholder="john@acme.com"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-1">Phone</label>
                <input
                  type="tel"
                  value={formData.contact_phone}
                  onChange={(e) => setFormData({ ...formData, contact_phone: e.target.value })}
                  className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
                  placeholder="+1 555 123 4567"
                />
              </div>
            </div>
          </div>

          {/* Status & Notes */}
          <div className="space-y-4 pt-4 border-t border-border">
            <div>
              <label className="block text-sm font-medium mb-1">Status</label>
              <select
                value={formData.status}
                onChange={(e) => setFormData({ ...formData, status: e.target.value })}
                className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none"
              >
                {STATUSES.map((status) => (
                  <option key={status} value={status}>
                    {status.charAt(0).toUpperCase() + status.slice(1)}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Notes</label>
              <textarea
                value={formData.notes}
                onChange={(e) => setFormData({ ...formData, notes: e.target.value })}
                rows={3}
                className="w-full px-4 py-2 rounded-lg bg-background border border-input focus:border-primary outline-none resize-none"
                placeholder="Add any notes about this lead..."
              />
            </div>
          </div>

          <div className="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 rounded-lg border border-border hover:bg-muted"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isLoading}
              className="px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {isLoading ? 'Creating...' : 'Create Lead'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

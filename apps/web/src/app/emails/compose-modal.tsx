'use client';

import { useState, useEffect } from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { api, Lead, GenerateEmailRequest } from '@/lib/api';
import {
  X,
  Sparkles,
  Send,
  User,
  Building,
  Briefcase,
  Zap,
  Loader2,
  ChevronDown,
  Check,
  FileText,
  Wand2,
  RefreshCw,
} from 'lucide-react';
import { toast } from 'sonner';

interface ComposeEmailModalProps {
  leads: Lead[];
  preselectedLead?: Lead;
  onClose: () => void;
  onSuccess: () => void;
}

const purposes = [
  { value: 'cold_outreach', label: 'Cold Outreach', icon: '🎯', desc: 'First contact with a new lead' },
  { value: 'follow_up', label: 'Follow Up', icon: '🔄', desc: 'Continue the conversation' },
  { value: 'cv_submission', label: 'CV Submission', icon: '📄', desc: 'Apply with your resume' },
  { value: 'meeting_request', label: 'Meeting Request', icon: '📅', desc: 'Schedule a call or meeting' },
  { value: 'thank_you', label: 'Thank You', icon: '🙏', desc: 'Express gratitude' },
];

const tones = [
  { value: 'professional', label: 'Professional', icon: '💼', desc: 'Business-appropriate' },
  { value: 'formal', label: 'Formal', icon: '🎩', desc: 'Very proper and respectful' },
  { value: 'friendly', label: 'Friendly', icon: '😊', desc: 'Warm and personable' },
  { value: 'casual', label: 'Casual', icon: '✌️', desc: 'Relaxed and conversational' },
];

export default function ComposeEmailModal({
  leads,
  preselectedLead,
  onClose,
  onSuccess,
}: ComposeEmailModalProps) {
  // State
  const [selectedLead, setSelectedLead] = useState<Lead | null>(preselectedLead || null);
  const [showLeadSelector, setShowLeadSelector] = useState(false);
  const [leadSearch, setLeadSearch] = useState('');
  const [purpose, setPurpose] = useState<GenerateEmailRequest['purpose']>('cold_outreach');
  const [tone, setTone] = useState<GenerateEmailRequest['tone']>('professional');
  const [includeCV, setIncludeCV] = useState(false);
  const [customInstructions, setCustomInstructions] = useState('');

  // Generated email state
  const [subject, setSubject] = useState('');
  const [bodyHtml, setBodyHtml] = useState('');
  const [isEditing, setIsEditing] = useState(false);

  // Generate email mutation
  const generateMutation = useMutation({
    mutationFn: (data: GenerateEmailRequest) => api.generateEmail(data),
    onSuccess: (data) => {
      setSubject(data.subject);
      setBodyHtml(data.bodyHtml);
      toast.success('Email generated with AI magic!', {
        icon: <Sparkles className="w-4 h-4 text-indigo-400" />,
      });
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to generate email');
    },
  });

  // Send email mutation
  const sendMutation = useMutation({
    mutationFn: (data: { leadId: string; subject: string; bodyHtml: string; purpose: string }) =>
      api.sendEmail(data),
    onSuccess: () => {
      toast.success('Email sent successfully!', {
        icon: <Send className="w-4 h-4 text-emerald-400" />,
      });
      onSuccess();
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to send email');
    },
  });

  // Filter leads based on search
  const filteredLeads = leads.filter(
    (lead) =>
      !leadSearch ||
      lead.company_name.toLowerCase().includes(leadSearch.toLowerCase()) ||
      lead.contact_name?.toLowerCase().includes(leadSearch.toLowerCase()) ||
      lead.contact_email?.toLowerCase().includes(leadSearch.toLowerCase())
  );

  // Generate email
  const handleGenerate = () => {
    if (!selectedLead) {
      toast.error('Please select a lead first');
      return;
    }

    generateMutation.mutate({
      leadId: selectedLead.id,
      purpose,
      tone,
      includeCV,
      customInstructions: customInstructions || undefined,
    });
  };

  // Send email
  const handleSend = () => {
    if (!selectedLead) {
      toast.error('Please select a lead');
      return;
    }
    if (!subject || !bodyHtml) {
      toast.error('Please generate or write an email first');
      return;
    }

    sendMutation.mutate({
      leadId: selectedLead.id,
      subject,
      bodyHtml,
      purpose,
    });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/80 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="relative w-full max-w-4xl max-h-[90vh] bg-gradient-to-br from-slate-900 to-slate-950 rounded-2xl border border-white/10 shadow-2xl overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-white/10">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-xl bg-gradient-to-br from-indigo-500 to-purple-600">
              <Sparkles className="w-5 h-5 text-white" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white">AI Email Composer</h2>
              <p className="text-sm text-white/50">Let AI craft the perfect outreach</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-white/10 text-white/60 hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="grid md:grid-cols-2 gap-6">
            {/* Left Column - Configuration */}
            <div className="space-y-6">
              {/* Lead Selector */}
              <div>
                <label className="block text-sm font-medium text-white/70 mb-2">
                  Select Lead
                </label>
                <div className="relative">
                  <button
                    onClick={() => setShowLeadSelector(!showLeadSelector)}
                    className="w-full p-4 rounded-xl bg-white/5 border border-white/10 hover:border-white/20 text-left transition-all"
                  >
                    {selectedLead ? (
                      <div className="flex items-center gap-3">
                        <div className="w-10 h-10 rounded-full bg-indigo-500/20 flex items-center justify-center">
                          <User className="w-5 h-5 text-indigo-400" />
                        </div>
                        <div className="flex-1">
                          <div className="font-medium text-white">
                            {selectedLead.contact_name || 'No contact'}
                          </div>
                          <div className="text-sm text-white/50 flex items-center gap-2">
                            <Building className="w-3 h-3" />
                            {selectedLead.company_name}
                            {selectedLead.contact_title && (
                              <>
                                <span className="text-white/20">•</span>
                                <Briefcase className="w-3 h-3" />
                                {selectedLead.contact_title}
                              </>
                            )}
                          </div>
                        </div>
                        <ChevronDown className="w-5 h-5 text-white/40" />
                      </div>
                    ) : (
                      <div className="flex items-center justify-between text-white/40">
                        <span>Choose a lead to email...</span>
                        <ChevronDown className="w-5 h-5" />
                      </div>
                    )}
                  </button>

                  {/* Lead Dropdown */}
                  {showLeadSelector && (
                    <div className="absolute top-full left-0 right-0 mt-2 p-2 rounded-xl bg-slate-800 border border-white/10 shadow-xl z-10 max-h-64 overflow-y-auto">
                      <input
                        type="text"
                        placeholder="Search leads..."
                        value={leadSearch}
                        onChange={(e) => setLeadSearch(e.target.value)}
                        className="w-full p-3 mb-2 rounded-lg bg-white/5 border border-white/10 text-white placeholder:text-white/30 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                        autoFocus
                      />
                      {filteredLeads.map((lead) => (
                        <button
                          key={lead.id}
                          onClick={() => {
                            setSelectedLead(lead);
                            setShowLeadSelector(false);
                            setLeadSearch('');
                          }}
                          className="w-full p-3 rounded-lg hover:bg-white/10 text-left transition-colors flex items-center gap-3"
                        >
                          <div className="w-8 h-8 rounded-full bg-indigo-500/20 flex items-center justify-center">
                            <User className="w-4 h-4 text-indigo-400" />
                          </div>
                          <div className="flex-1">
                            <div className="text-sm font-medium text-white">
                              {lead.contact_name || 'No contact'}
                            </div>
                            <div className="text-xs text-white/50">
                              {lead.company_name}
                            </div>
                          </div>
                          {selectedLead?.id === lead.id && (
                            <Check className="w-4 h-4 text-emerald-400" />
                          )}
                        </button>
                      ))}
                      {filteredLeads.length === 0 && (
                        <div className="p-4 text-center text-white/40 text-sm">
                          No leads found
                        </div>
                      )}
                    </div>
                  )}
                </div>
              </div>

              {/* Purpose Selector */}
              <div>
                <label className="block text-sm font-medium text-white/70 mb-2">
                  Email Purpose
                </label>
                <div className="grid grid-cols-2 gap-2">
                  {purposes.map((p) => (
                    <button
                      key={p.value}
                      onClick={() => setPurpose(p.value as GenerateEmailRequest['purpose'])}
                      className={`p-3 rounded-xl border text-left transition-all ${
                        purpose === p.value
                          ? 'bg-indigo-500/20 border-indigo-500/50 ring-2 ring-indigo-500/30'
                          : 'bg-white/5 border-white/10 hover:border-white/20'
                      }`}
                    >
                      <div className="flex items-center gap-2 mb-1">
                        <span>{p.icon}</span>
                        <span className="text-sm font-medium text-white">{p.label}</span>
                      </div>
                      <p className="text-xs text-white/40">{p.desc}</p>
                    </button>
                  ))}
                </div>
              </div>

              {/* Tone Selector */}
              <div>
                <label className="block text-sm font-medium text-white/70 mb-2">
                  Tone
                </label>
                <div className="flex flex-wrap gap-2">
                  {tones.map((t) => (
                    <button
                      key={t.value}
                      onClick={() => setTone(t.value as GenerateEmailRequest['tone'])}
                      className={`px-4 py-2 rounded-lg border text-sm transition-all flex items-center gap-2 ${
                        tone === t.value
                          ? 'bg-indigo-500/20 border-indigo-500/50 text-white'
                          : 'bg-white/5 border-white/10 text-white/60 hover:border-white/20 hover:text-white'
                      }`}
                    >
                      <span>{t.icon}</span>
                      {t.label}
                    </button>
                  ))}
                </div>
              </div>

              {/* Custom Instructions */}
              <div>
                <label className="block text-sm font-medium text-white/70 mb-2">
                  Custom Instructions (Optional)
                </label>
                <textarea
                  value={customInstructions}
                  onChange={(e) => setCustomInstructions(e.target.value)}
                  placeholder="e.g., Mention our recent product launch, reference their LinkedIn post about AI..."
                  className="w-full p-4 rounded-xl bg-white/5 border border-white/10 text-white placeholder:text-white/30 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 resize-none h-24"
                />
              </div>

              {/* Include CV Toggle */}
              {purpose === 'cv_submission' && (
                <label className="flex items-center gap-3 p-4 rounded-xl bg-white/5 border border-white/10 cursor-pointer hover:border-white/20 transition-colors">
                  <input
                    type="checkbox"
                    checked={includeCV}
                    onChange={(e) => setIncludeCV(e.target.checked)}
                    className="w-5 h-5 rounded border-white/20 bg-white/5 text-indigo-500 focus:ring-indigo-500/50"
                  />
                  <div className="flex-1">
                    <div className="font-medium text-white flex items-center gap-2">
                      <FileText className="w-4 h-4" />
                      Include CV/Resume
                    </div>
                    <p className="text-xs text-white/50">Attach your default CV to this email</p>
                  </div>
                </label>
              )}

              {/* Generate Button */}
              <button
                onClick={handleGenerate}
                disabled={!selectedLead || generateMutation.isPending}
                className="w-full py-4 rounded-xl bg-gradient-to-r from-indigo-500 to-purple-600 text-white font-semibold flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed hover:shadow-lg hover:shadow-indigo-500/25 transition-all"
              >
                {generateMutation.isPending ? (
                  <>
                    <Loader2 className="w-5 h-5 animate-spin" />
                    Generating Magic...
                  </>
                ) : (
                  <>
                    <Wand2 className="w-5 h-5" />
                    Generate with AI
                  </>
                )}
              </button>
            </div>

            {/* Right Column - Preview */}
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <label className="text-sm font-medium text-white/70">
                  Email Preview
                </label>
                {bodyHtml && (
                  <button
                    onClick={handleGenerate}
                    disabled={generateMutation.isPending}
                    className="text-sm text-indigo-400 hover:text-indigo-300 flex items-center gap-1"
                  >
                    <RefreshCw className={`w-4 h-4 ${generateMutation.isPending ? 'animate-spin' : ''}`} />
                    Regenerate
                  </button>
                )}
              </div>

              {/* Subject Line */}
              <div>
                <input
                  type="text"
                  value={subject}
                  onChange={(e) => setSubject(e.target.value)}
                  placeholder="Subject line..."
                  className="w-full p-4 rounded-xl bg-white/5 border border-white/10 text-white placeholder:text-white/30 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 font-medium"
                />
              </div>

              {/* Email Body */}
              <div className="relative">
                {isEditing ? (
                  <textarea
                    value={bodyHtml.replace(/<[^>]*>/g, '')}
                    onChange={(e) => setBodyHtml(`<p>${e.target.value.replace(/\n/g, '</p><p>')}</p>`)}
                    className="w-full p-4 rounded-xl bg-white/5 border border-white/10 text-white placeholder:text-white/30 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 resize-none h-80 font-mono text-sm"
                    placeholder="Your email content will appear here after generation..."
                  />
                ) : (
                  <div
                    className="w-full p-4 rounded-xl bg-white/5 border border-white/10 min-h-80 overflow-y-auto prose prose-invert prose-sm max-w-none"
                    dangerouslySetInnerHTML={{
                      __html: bodyHtml || '<p class="text-white/30">Your email content will appear here after generation...</p>',
                    }}
                    onClick={() => bodyHtml && setIsEditing(true)}
                  />
                )}
                {bodyHtml && !isEditing && (
                  <button
                    onClick={() => setIsEditing(true)}
                    className="absolute top-2 right-2 px-3 py-1 rounded-lg bg-white/10 text-xs text-white/60 hover:text-white hover:bg-white/20 transition-colors"
                  >
                    Edit
                  </button>
                )}
                {isEditing && (
                  <button
                    onClick={() => setIsEditing(false)}
                    className="absolute top-2 right-2 px-3 py-1 rounded-lg bg-indigo-500/20 text-xs text-indigo-400 hover:bg-indigo-500/30 transition-colors"
                  >
                    Preview
                  </button>
                )}
              </div>

              {/* Recipient Info */}
              {selectedLead && (
                <div className="p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
                  <div className="flex items-center gap-2 text-emerald-400 text-sm font-medium mb-1">
                    <Check className="w-4 h-4" />
                    Ready to send
                  </div>
                  <p className="text-sm text-white/60">
                    To: {selectedLead.contact_email || `${selectedLead.contact_name} at ${selectedLead.company_name}`}
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-6 border-t border-white/10 bg-black/20">
          <button
            onClick={onClose}
            className="px-6 py-3 rounded-xl border border-white/10 text-white/60 hover:text-white hover:bg-white/5 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSend}
            disabled={!selectedLead || !subject || !bodyHtml || sendMutation.isPending}
            className="px-8 py-3 rounded-xl bg-gradient-to-r from-emerald-500 to-green-600 text-white font-semibold flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed hover:shadow-lg hover:shadow-emerald-500/25 transition-all"
          >
            {sendMutation.isPending ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                Sending...
              </>
            ) : (
              <>
                <Send className="w-5 h-5" />
                Send Email
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
}

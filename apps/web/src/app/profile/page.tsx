'use client';

import { useState, useEffect } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api, CVProfile, CVAnalysis, Skill, Experience, Education } from '@/lib/api';

// Skill Level Colors
const skillLevelColors = {
  beginner: 'bg-gray-500',
  intermediate: 'bg-blue-500',
  advanced: 'bg-emerald-500',
  expert: 'bg-purple-500',
};

// ATS Score Color
function getATSScoreColor(score: number) {
  if (score >= 80) return 'text-emerald-400';
  if (score >= 60) return 'text-amber-400';
  return 'text-red-400';
}

export default function ProfilePage() {
  const queryClient = useQueryClient();
  const [activeTab, setActiveTab] = useState<'profile' | 'ats' | 'preview'>('profile');
  const [isEditing, setIsEditing] = useState(false);
  const [editedProfile, setEditedProfile] = useState<Partial<CVProfile>>({});

  // Fetch profile
  const { data: profile, isLoading: profileLoading, error: profileError } = useQuery({
    queryKey: ['cvProfile'],
    queryFn: () => api.getCVProfile(),
    retry: false,
  });

  // Fetch templates
  const { data: templatesData } = useQuery({
    queryKey: ['cvTemplates'],
    queryFn: () => api.getCVTemplates(),
  });

  // Save profile mutation
  const saveMutation = useMutation({
    mutationFn: (data: Partial<CVProfile>) => api.saveCVProfile(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['cvProfile'] });
      setIsEditing(false);
    },
  });

  // Analyze CV mutation
  const analyzeMutation = useMutation({
    mutationFn: (jobDescription?: string) => api.analyzeCV(jobDescription),
  });

  // Generate HTML mutation
  const generateMutation = useMutation({
    mutationFn: (templateId?: string) => api.generateCVHtml(templateId),
  });

  useEffect(() => {
    if (profile) {
      setEditedProfile(profile);
    }
  }, [profile]);

  const handleSaveProfile = () => {
    saveMutation.mutate(editedProfile);
  };

  const handleAnalyze = () => {
    analyzeMutation.mutate(undefined);
  };

  const handleGeneratePreview = (templateId?: string) => {
    generateMutation.mutate(templateId);
  };

  // Add skill
  const addSkill = () => {
    setEditedProfile(prev => ({
      ...prev,
      skills: [...(prev.skills || []), { name: '', level: 'intermediate' as const, category: 'tool' as const }],
    }));
  };

  // Remove skill
  const removeSkill = (index: number) => {
    setEditedProfile(prev => ({
      ...prev,
      skills: prev.skills?.filter((_, i) => i !== index),
    }));
  };

  // Update skill
  const updateSkill = (index: number, field: keyof Skill, value: string) => {
    setEditedProfile(prev => ({
      ...prev,
      skills: prev.skills?.map((skill, i) =>
        i === index ? { ...skill, [field]: value } : skill
      ),
    }));
  };

  // Add experience
  const addExperience = () => {
    setEditedProfile(prev => ({
      ...prev,
      experience: [...(prev.experience || []), {
        company: '',
        title: '',
        startDate: '',
        endDate: '',
        achievements: [''],
      }],
    }));
  };

  // Remove experience
  const removeExperience = (index: number) => {
    setEditedProfile(prev => ({
      ...prev,
      experience: prev.experience?.filter((_, i) => i !== index),
    }));
  };

  // Update experience
  const updateExperience = (index: number, field: keyof Experience, value: any) => {
    setEditedProfile(prev => ({
      ...prev,
      experience: prev.experience?.map((exp, i) =>
        i === index ? { ...exp, [field]: value } : exp
      ),
    }));
  };

  // Add education
  const addEducation = () => {
    setEditedProfile(prev => ({
      ...prev,
      education: [...(prev.education || []), {
        institution: '',
        degree: '',
        field: '',
        endDate: '',
      }],
    }));
  };

  // Remove education
  const removeEducation = (index: number) => {
    setEditedProfile(prev => ({
      ...prev,
      education: prev.education?.filter((_, i) => i !== index),
    }));
  };

  // Update education
  const updateEducation = (index: number, field: keyof Education, value: any) => {
    setEditedProfile(prev => ({
      ...prev,
      education: prev.education?.map((edu, i) =>
        i === index ? { ...edu, [field]: value } : edu
      ),
    }));
  };

  if (profileLoading) {
    return (
      <div className="min-h-screen bg-gray-950 flex items-center justify-center">
        <div className="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-indigo-500"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-950 p-6">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-3xl font-bold text-white">CV Profile</h1>
            <p className="text-gray-400 mt-1">Manage your professional profile and generate tailored CVs</p>
          </div>
          <div className="flex gap-3">
            {isEditing ? (
              <>
                <button
                  onClick={() => setIsEditing(false)}
                  className="px-4 py-2 rounded-lg bg-gray-800 text-gray-300 hover:bg-gray-700"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSaveProfile}
                  disabled={saveMutation.isPending}
                  className="px-4 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
                >
                  {saveMutation.isPending ? 'Saving...' : 'Save Profile'}
                </button>
              </>
            ) : (
              <button
                onClick={() => setIsEditing(true)}
                className="px-4 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700"
              >
                Edit Profile
              </button>
            )}
          </div>
        </div>

        {/* Tabs */}
        <div className="flex gap-4 mb-6 border-b border-gray-800">
          {(['profile', 'ats', 'preview'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-3 font-medium capitalize transition ${
                activeTab === tab
                  ? 'text-indigo-400 border-b-2 border-indigo-400'
                  : 'text-gray-400 hover:text-gray-300'
              }`}
            >
              {tab === 'ats' ? 'ATS Analysis' : tab}
            </button>
          ))}
        </div>

        {/* Profile Tab */}
        {activeTab === 'profile' && (
          <div className="space-y-6">
            {/* Basic Info */}
            <div className="bg-gray-900 rounded-xl p-6">
              <h2 className="text-xl font-semibold text-white mb-4">Basic Information</h2>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Full Name *</label>
                  <input
                    type="text"
                    value={editedProfile.fullName || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, fullName: e.target.value }))}
                    disabled={!isEditing}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60"
                    placeholder="John Doe"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Email *</label>
                  <input
                    type="email"
                    value={editedProfile.email || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, email: e.target.value }))}
                    disabled={!isEditing}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60"
                    placeholder="john@example.com"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Phone</label>
                  <input
                    type="tel"
                    value={editedProfile.phone || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, phone: e.target.value }))}
                    disabled={!isEditing}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60"
                    placeholder="+1 (555) 123-4567"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Location</label>
                  <input
                    type="text"
                    value={editedProfile.location || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, location: e.target.value }))}
                    disabled={!isEditing}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60"
                    placeholder="New York, NY"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">LinkedIn</label>
                  <input
                    type="url"
                    value={editedProfile.linkedin || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, linkedin: e.target.value }))}
                    disabled={!isEditing}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60"
                    placeholder="https://linkedin.com/in/johndoe"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">GitHub</label>
                  <input
                    type="url"
                    value={editedProfile.github || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, github: e.target.value }))}
                    disabled={!isEditing}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60"
                    placeholder="https://github.com/johndoe"
                  />
                </div>
              </div>
            </div>

            {/* Professional Summary */}
            <div className="bg-gray-900 rounded-xl p-6">
              <h2 className="text-xl font-semibold text-white mb-4">Professional Summary</h2>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Headline</label>
                  <input
                    type="text"
                    value={editedProfile.headline || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, headline: e.target.value }))}
                    disabled={!isEditing}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60"
                    placeholder="Senior Full-Stack Developer"
                  />
                </div>
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Summary</label>
                  <textarea
                    value={editedProfile.summary || ''}
                    onChange={(e) => setEditedProfile(prev => ({ ...prev, summary: e.target.value }))}
                    disabled={!isEditing}
                    rows={4}
                    className="w-full px-4 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 disabled:opacity-60 resize-none"
                    placeholder="Results-driven professional with 8+ years of experience..."
                  />
                </div>
              </div>
            </div>

            {/* Skills */}
            <div className="bg-gray-900 rounded-xl p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold text-white">Skills</h2>
                {isEditing && (
                  <button onClick={addSkill} className="text-indigo-400 hover:text-indigo-300">
                    + Add Skill
                  </button>
                )}
              </div>
              <div className="space-y-3">
                {editedProfile.skills?.map((skill, index) => (
                  <div key={index} className="flex items-center gap-3">
                    <input
                      type="text"
                      value={skill.name}
                      onChange={(e) => updateSkill(index, 'name', e.target.value)}
                      disabled={!isEditing}
                      className="flex-1 px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                      placeholder="Skill name"
                    />
                    <select
                      value={skill.level || 'intermediate'}
                      onChange={(e) => updateSkill(index, 'level', e.target.value)}
                      disabled={!isEditing}
                      className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                    >
                      <option value="beginner">Beginner</option>
                      <option value="intermediate">Intermediate</option>
                      <option value="advanced">Advanced</option>
                      <option value="expert">Expert</option>
                    </select>
                    <select
                      value={skill.category || 'tool'}
                      onChange={(e) => updateSkill(index, 'category', e.target.value)}
                      disabled={!isEditing}
                      className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                    >
                      <option value="language">Language</option>
                      <option value="framework">Framework</option>
                      <option value="tool">Tool</option>
                      <option value="soft_skill">Soft Skill</option>
                      <option value="other">Other</option>
                    </select>
                    {isEditing && (
                      <button
                        onClick={() => removeSkill(index)}
                        className="text-red-400 hover:text-red-300"
                      >
                        Remove
                      </button>
                    )}
                  </div>
                ))}
                {(!editedProfile.skills || editedProfile.skills.length === 0) && (
                  <p className="text-gray-500 text-center py-4">No skills added yet</p>
                )}
              </div>
            </div>

            {/* Experience */}
            <div className="bg-gray-900 rounded-xl p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold text-white">Experience</h2>
                {isEditing && (
                  <button onClick={addExperience} className="text-indigo-400 hover:text-indigo-300">
                    + Add Experience
                  </button>
                )}
              </div>
              <div className="space-y-6">
                {editedProfile.experience?.map((exp, index) => (
                  <div key={index} className="border border-gray-800 rounded-lg p-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                      <input
                        type="text"
                        value={exp.company}
                        onChange={(e) => updateExperience(index, 'company', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="Company"
                      />
                      <input
                        type="text"
                        value={exp.title}
                        onChange={(e) => updateExperience(index, 'title', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="Job Title"
                      />
                      <input
                        type="text"
                        value={exp.startDate}
                        onChange={(e) => updateExperience(index, 'startDate', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="Start Date (YYYY-MM)"
                      />
                      <input
                        type="text"
                        value={exp.endDate || ''}
                        onChange={(e) => updateExperience(index, 'endDate', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="End Date (or leave empty if current)"
                      />
                    </div>
                    <div>
                      <label className="block text-sm text-gray-400 mb-1">Achievements (one per line)</label>
                      <textarea
                        value={exp.achievements.join('\n')}
                        onChange={(e) => updateExperience(index, 'achievements', e.target.value.split('\n').filter(Boolean))}
                        disabled={!isEditing}
                        rows={3}
                        className="w-full px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60 resize-none"
                        placeholder="Led a team of 5 developers..."
                      />
                    </div>
                    {isEditing && (
                      <button
                        onClick={() => removeExperience(index)}
                        className="mt-2 text-red-400 hover:text-red-300 text-sm"
                      >
                        Remove Experience
                      </button>
                    )}
                  </div>
                ))}
                {(!editedProfile.experience || editedProfile.experience.length === 0) && (
                  <p className="text-gray-500 text-center py-4">No experience added yet</p>
                )}
              </div>
            </div>

            {/* Education */}
            <div className="bg-gray-900 rounded-xl p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-xl font-semibold text-white">Education</h2>
                {isEditing && (
                  <button onClick={addEducation} className="text-indigo-400 hover:text-indigo-300">
                    + Add Education
                  </button>
                )}
              </div>
              <div className="space-y-4">
                {editedProfile.education?.map((edu, index) => (
                  <div key={index} className="border border-gray-800 rounded-lg p-4">
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <input
                        type="text"
                        value={edu.institution}
                        onChange={(e) => updateEducation(index, 'institution', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="University/Institution"
                      />
                      <input
                        type="text"
                        value={edu.degree}
                        onChange={(e) => updateEducation(index, 'degree', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="Degree (B.S., M.S., etc.)"
                      />
                      <input
                        type="text"
                        value={edu.field}
                        onChange={(e) => updateEducation(index, 'field', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="Field of Study"
                      />
                      <input
                        type="text"
                        value={edu.endDate}
                        onChange={(e) => updateEducation(index, 'endDate', e.target.value)}
                        disabled={!isEditing}
                        className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm focus:border-indigo-500 disabled:opacity-60"
                        placeholder="Graduation Year"
                      />
                    </div>
                    {isEditing && (
                      <button
                        onClick={() => removeEducation(index)}
                        className="mt-2 text-red-400 hover:text-red-300 text-sm"
                      >
                        Remove Education
                      </button>
                    )}
                  </div>
                ))}
                {(!editedProfile.education || editedProfile.education.length === 0) && (
                  <p className="text-gray-500 text-center py-4">No education added yet</p>
                )}
              </div>
            </div>
          </div>
        )}

        {/* ATS Analysis Tab */}
        {activeTab === 'ats' && (
          <div className="space-y-6">
            <div className="bg-gray-900 rounded-xl p-6">
              <div className="flex items-center justify-between mb-6">
                <div>
                  <h2 className="text-xl font-semibold text-white">ATS Optimization Analysis</h2>
                  <p className="text-gray-400 text-sm mt-1">
                    See how well your CV will perform with Applicant Tracking Systems
                  </p>
                </div>
                <button
                  onClick={handleAnalyze}
                  disabled={analyzeMutation.isPending || !profile}
                  className="px-4 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
                >
                  {analyzeMutation.isPending ? 'Analyzing...' : 'Analyze CV'}
                </button>
              </div>

              {analyzeMutation.data && (
                <div className="space-y-6">
                  {/* Score */}
                  <div className="flex items-center gap-6">
                    <div className="text-center">
                      <div className={`text-5xl font-bold ${getATSScoreColor(analyzeMutation.data.atsScore)}`}>
                        {analyzeMutation.data.atsScore}
                      </div>
                      <div className="text-gray-400 text-sm">ATS Score</div>
                    </div>
                    <div className="flex-1 bg-gray-800 rounded-full h-4 overflow-hidden">
                      <div
                        className={`h-full transition-all ${
                          analyzeMutation.data.atsScore >= 80 ? 'bg-emerald-500' :
                          analyzeMutation.data.atsScore >= 60 ? 'bg-amber-500' : 'bg-red-500'
                        }`}
                        style={{ width: `${analyzeMutation.data.atsScore}%` }}
                      />
                    </div>
                  </div>

                  {/* Issues */}
                  {analyzeMutation.data.issues.length > 0 && (
                    <div>
                      <h3 className="text-lg font-semibold text-white mb-3">Issues Found</h3>
                      <div className="space-y-3">
                        {analyzeMutation.data.issues.map((issue, i) => (
                          <div key={i} className="bg-gray-800 rounded-lg p-4">
                            <div className="flex items-start gap-3">
                              <span className={`px-2 py-1 rounded text-xs font-medium ${
                                issue.severity === 'high' ? 'bg-red-500/20 text-red-400' :
                                issue.severity === 'medium' ? 'bg-amber-500/20 text-amber-400' :
                                'bg-blue-500/20 text-blue-400'
                              }`}>
                                {issue.severity.toUpperCase()}
                              </span>
                              <div>
                                <p className="text-white">{issue.issue}</p>
                                <p className="text-gray-400 text-sm mt-1">{issue.suggestion}</p>
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Suggestions */}
                  {analyzeMutation.data.suggestions.length > 0 && (
                    <div>
                      <h3 className="text-lg font-semibold text-white mb-3">Improvement Suggestions</h3>
                      <div className="space-y-2">
                        {analyzeMutation.data.suggestions.map((suggestion, i) => (
                          <div key={i} className="bg-gray-800 rounded-lg p-3 flex items-start gap-3">
                            <span className="text-indigo-400">#{i + 1}</span>
                            <div>
                              <span className="text-gray-400 text-sm">[{suggestion.section}]</span>
                              <p className="text-white">{suggestion.suggestion}</p>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}

              {!analyzeMutation.data && !analyzeMutation.isPending && (
                <div className="text-center py-12 text-gray-500">
                  Click "Analyze CV" to get ATS optimization suggestions
                </div>
              )}
            </div>
          </div>
        )}

        {/* Preview Tab */}
        {activeTab === 'preview' && (
          <div className="space-y-6">
            <div className="bg-gray-900 rounded-xl p-6">
              <div className="flex items-center justify-between mb-6">
                <div>
                  <h2 className="text-xl font-semibold text-white">CV Preview</h2>
                  <p className="text-gray-400 text-sm mt-1">
                    Preview your CV with different templates
                  </p>
                </div>
                <div className="flex gap-3">
                  <select
                    className="px-3 py-2 rounded-lg bg-gray-800 border border-gray-700 text-white text-sm"
                    onChange={(e) => handleGeneratePreview(e.target.value || undefined)}
                  >
                    <option value="">Select Template</option>
                    {templatesData?.templates.map((template) => (
                      <option key={template.id} value={template.id}>
                        {template.name}
                      </option>
                    ))}
                  </select>
                  <button
                    onClick={() => handleGeneratePreview()}
                    disabled={generateMutation.isPending || !profile}
                    className="px-4 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700 disabled:opacity-50"
                  >
                    {generateMutation.isPending ? 'Generating...' : 'Generate Preview'}
                  </button>
                </div>
              </div>

              {generateMutation.data?.html && (
                <div className="bg-white rounded-lg overflow-hidden">
                  <iframe
                    srcDoc={generateMutation.data.html}
                    className="w-full h-[800px] border-0"
                    title="CV Preview"
                  />
                </div>
              )}

              {!generateMutation.data && !generateMutation.isPending && (
                <div className="text-center py-12 text-gray-500">
                  Select a template and click "Generate Preview" to see your CV
                </div>
              )}
            </div>
          </div>
        )}

        {/* Error states */}
        {profileError && (
          <div className="bg-gray-900 rounded-xl p-6 text-center">
            <p className="text-gray-400 mb-4">No profile found. Start by creating your CV profile.</p>
            <button
              onClick={() => setIsEditing(true)}
              className="px-4 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-700"
            >
              Create Profile
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

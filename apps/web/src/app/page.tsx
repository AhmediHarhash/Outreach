'use client';

import { useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { useAuth } from '@/lib/auth';
import Link from 'next/link';

export default function Home() {
  const router = useRouter();
  const { isAuthenticated, isLoading, checkAuth } = useAuth();

  useEffect(() => {
    checkAuth();
  }, [checkAuth]);

  useEffect(() => {
    if (!isLoading && isAuthenticated) {
      router.push('/dashboard');
    }
  }, [isAuthenticated, isLoading, router]);

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-primary"></div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gradient-to-br from-background via-background to-primary/5">
      {/* Navigation */}
      <nav className="fixed top-0 w-full z-50 border-b border-border/40 bg-background/80 backdrop-blur-xl">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16 items-center">
            <div className="flex items-center space-x-2">
              <div className="w-8 h-8 rounded-lg bg-primary flex items-center justify-center">
                <span className="text-primary-foreground font-bold">H</span>
              </div>
              <span className="text-xl font-bold">Hekax</span>
            </div>
            <div className="flex items-center space-x-4">
              <Link
                href="/auth/login"
                className="text-sm text-muted-foreground hover:text-foreground transition-colors"
              >
                Login
              </Link>
              <Link
                href="/auth/register"
                className="px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors"
              >
                Get Started
              </Link>
            </div>
          </div>
        </div>
      </nav>

      {/* Hero */}
      <main className="pt-32 pb-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-5xl mx-auto text-center">
          <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold tracking-tight">
            Close Deals with
            <span className="text-primary block mt-2">AI Superpowers</span>
          </h1>
          <p className="mt-8 text-xl text-muted-foreground max-w-2xl mx-auto">
            Real-time AI assistance during your sales calls, interviews, and negotiations.
            Never miss an objection. Always have the perfect response.
          </p>
          <div className="mt-12 flex flex-col sm:flex-row gap-4 justify-center">
            <Link
              href="/auth/register"
              className="px-8 py-4 rounded-xl bg-primary text-primary-foreground font-semibold text-lg hover:bg-primary/90 transition-all hover:scale-105"
            >
              Start Free Trial
            </Link>
            <Link
              href="#features"
              className="px-8 py-4 rounded-xl border border-border text-foreground font-semibold text-lg hover:bg-accent transition-colors"
            >
              See How It Works
            </Link>
          </div>
        </div>

        {/* Features */}
        <section id="features" className="mt-32 max-w-6xl mx-auto">
          <h2 className="text-3xl font-bold text-center mb-16">The Hekax Ecosystem</h2>
          <div className="grid md:grid-cols-3 gap-8">
            <FeatureCard
              icon="🎯"
              title="Voice Copilot"
              description="Real-time AI suggestions during calls. Flash bullets for quick wins, deep analysis for complex situations."
            />
            <FeatureCard
              icon="🔍"
              title="Lead Hunter"
              description="Find high-ticket B2B leads with Apollo integration. Auto-research companies before your calls."
            />
            <FeatureCard
              icon="📊"
              title="Call Analytics"
              description="AI-powered post-call analysis. Know what worked, what didn't, and how to improve."
            />
          </div>
        </section>

        {/* CTA */}
        <section className="mt-32 max-w-4xl mx-auto text-center">
          <div className="p-12 rounded-3xl bg-gradient-to-r from-primary/10 to-primary/5 border border-primary/20">
            <h2 className="text-3xl font-bold mb-4">Ready to Close More Deals?</h2>
            <p className="text-muted-foreground mb-8">
              Join top performers using AI to win every conversation.
            </p>
            <Link
              href="/auth/register"
              className="inline-block px-8 py-4 rounded-xl bg-primary text-primary-foreground font-semibold hover:bg-primary/90 transition-colors"
            >
              Get Started Free
            </Link>
          </div>
        </section>
      </main>

      {/* Footer */}
      <footer className="border-t border-border py-12 mt-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center">
            <div className="flex items-center space-x-2">
              <div className="w-6 h-6 rounded bg-primary flex items-center justify-center">
                <span className="text-primary-foreground text-sm font-bold">H</span>
              </div>
              <span className="font-semibold">Hekax</span>
            </div>
            <p className="text-sm text-muted-foreground">
              Built for closers. Powered by AI.
            </p>
          </div>
        </div>
      </footer>
    </div>
  );
}

function FeatureCard({
  icon,
  title,
  description,
}: {
  icon: string;
  title: string;
  description: string;
}) {
  return (
    <div className="p-8 rounded-2xl border border-border bg-card hover:border-primary/50 transition-colors">
      <div className="text-4xl mb-4">{icon}</div>
      <h3 className="text-xl font-semibold mb-2">{title}</h3>
      <p className="text-muted-foreground">{description}</p>
    </div>
  );
}
